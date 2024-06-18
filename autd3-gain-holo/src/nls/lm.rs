use std::{collections::HashMap, sync::Arc};

use crate::{
    constraint::EmissionConstraint, helper::generate_result, Amplitude, Complex, HoloError,
    LinAlgBackend, Trans,
};

use autd3_driver::{acoustics::directivity::Directivity, derive::*, geometry::Vector3};
use bitvec::{order::Lsb0, vec::BitVec};

#[derive(Gain, Builder)]
#[no_const]
pub struct LM<D: Directivity + 'static, B: LinAlgBackend<D> + 'static> {
    #[get]
    foci: Vec<Vector3>,
    #[get]
    amps: Vec<Amplitude>,
    #[getset]
    eps_1: f32,
    #[getset]
    eps_2: f32,
    #[getset]
    tau: f32,
    #[getset]
    k_max: usize,
    #[getset]
    initial: Vec<f32>,
    #[getset]
    constraint: EmissionConstraint,
    backend: Arc<B>,
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity, B: LinAlgBackend<D>> LM<D, B> {
    pub fn new(backend: Arc<B>, iter: impl IntoIterator<Item = (Vector3, Amplitude)>) -> Self {
        let (foci, amps) = iter.into_iter().unzip();
        Self {
            foci,
            amps,
            eps_1: 1e-8,
            eps_2: 1e-8,
            tau: 1e-3,
            k_max: 5,
            initial: vec![],
            backend,
            constraint: EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<D: Directivity, B: LinAlgBackend<D>> LM<D, B> {
    fn make_t(
        &self,
        zero: &B::VectorX,
        x: &B::VectorX,
        t: &mut B::VectorXc,
    ) -> Result<(), HoloError> {
        self.backend.make_complex2_v(zero, x, t)?;
        self.backend.scale_assign_cv(Complex::new(-1., 0.), t)?;
        self.backend.exp_assign_cv(t)
    }

    #[allow(clippy::too_many_arguments)]
    fn calc_jtj_jtf(
        &self,
        t: &B::VectorXc,
        bhb: &B::MatrixXc,
        tth: &mut B::MatrixXc,
        bhb_tth: &mut B::MatrixXc,
        bhb_tth_i: &mut B::MatrixX,
        jtj: &mut B::MatrixX,
        jtf: &mut B::VectorX,
    ) -> Result<(), HoloError> {
        self.backend.gevv_c(
            Trans::NoTrans,
            Trans::ConjTrans,
            Complex::new(1., 0.),
            t,
            t,
            Complex::new(0., 0.),
            tth,
        )?;
        self.backend.hadamard_product_cm(bhb, tth, bhb_tth)?;

        self.backend.real_cm(bhb_tth, jtj)?;
        self.backend.imag_cm(bhb_tth, bhb_tth_i)?;

        self.backend.reduce_col(bhb_tth_i, jtf)
    }

    fn calc_fx(
        &self,
        zero: &B::VectorX,
        x: &B::VectorX,
        bhb: &B::MatrixXc,
        tmp: &mut B::VectorXc,
        t: &mut B::VectorXc,
    ) -> Result<f32, HoloError> {
        self.backend.make_complex2_v(zero, x, t)?;
        self.backend.exp_assign_cv(t)?;
        self.backend.gemv_c(
            Trans::NoTrans,
            Complex::new(1., 0.),
            bhb,
            t,
            Complex::new(0., 0.),
            tmp,
        )?;
        Ok(self.backend.dot_c(t, tmp)?.re)
    }
}

impl<D: Directivity, B: LinAlgBackend<D>> LM<D, B> {
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::uninit_vec)]
    fn calc_impl(
        &self,
        geometry: &Geometry,
        filter: Option<HashMap<usize, BitVec<usize, Lsb0>>>,
    ) -> GainCalcResult {
        let g = self
            .backend
            .generate_propagation_matrix(geometry, &self.foci, &filter)?;

        let n = self.backend.cols_c(&g)?;
        let m = self.foci.len();

        let n_param = m + n;

        let bhb = {
            let mut bhb = self.backend.alloc_zeros_cm(n_param, n_param)?;

            let mut amps = self.backend.from_slice_cv(unsafe {
                std::slice::from_raw_parts(self.amps.as_ptr() as *const f32, self.amps.len())
            })?;

            let mut p = self.backend.alloc_cm(m, m)?;
            self.backend
                .scale_assign_cv(Complex::new(-1., 0.), &mut amps)?;
            self.backend.create_diagonal_c(&amps, &mut p)?;

            let mut b = self.backend.alloc_cm(m, n_param)?;
            self.backend.concat_col_cm(&g, &p, &mut b)?;

            self.backend.gemm_c(
                Trans::ConjTrans,
                Trans::NoTrans,
                Complex::new(1., 0.),
                &b,
                &b,
                Complex::new(0., 0.),
                &mut bhb,
            )?;
            bhb
        };

        let mut x = self.backend.alloc_zeros_v(n_param)?;
        self.backend.copy_from_slice_v(&self.initial, &mut x)?;

        let mut nu = 2.0;

        let zero = self.backend.alloc_zeros_v(n_param)?;

        let mut t = self.backend.alloc_cv(n_param)?;
        self.make_t(&zero, &x, &mut t)?;

        let mut tth = self.backend.alloc_cm(n_param, n_param)?;
        let mut bhb_tth = self.backend.alloc_cm(n_param, n_param)?;
        let mut bhb_tth_i = self.backend.alloc_m(n_param, n_param)?;
        let mut a = self.backend.alloc_m(n_param, n_param)?;
        let mut g = self.backend.alloc_v(n_param)?;
        self.calc_jtj_jtf(
            &t,
            &bhb,
            &mut tth,
            &mut bhb_tth,
            &mut bhb_tth_i,
            &mut a,
            &mut g,
        )?;

        let mut a_diag = self.backend.alloc_v(n_param)?;
        self.backend.get_diagonal(&a, &mut a_diag)?;
        let a_max = self.backend.max_v(&a_diag)?;

        let mut mu = self.tau * a_max;

        let mut tmp = self.backend.alloc_zeros_cv(n_param)?;
        let mut fx = self.calc_fx(&zero, &x, &bhb, &mut tmp, &mut t)?;

        let ones = vec![1.0; n_param];
        let ones = self.backend.from_slice_v(&ones)?;
        let mut identity = self.backend.alloc_m(n_param, n_param)?;
        self.backend.create_diagonal(&ones, &mut identity)?;

        let mut h_lm = self.backend.alloc_v(n_param)?;
        let mut x_new = self.backend.alloc_v(n_param)?;
        let mut tmp_mat = self.backend.alloc_m(n_param, n_param)?;
        let mut tmp_vec = self.backend.alloc_v(n_param)?;
        for _ in 0..self.k_max {
            if self.backend.max_v(&g)? <= self.eps_1 {
                break; // GRCOV_EXCL_LINE
            }

            self.backend.copy_to_m(&a, &mut tmp_mat)?;
            self.backend.add_m(mu, &identity, &mut tmp_mat)?;

            self.backend.copy_to_v(&g, &mut h_lm)?;

            self.backend.solve_inplace(&tmp_mat, &mut h_lm)?;

            if self.backend.dot(&h_lm, &h_lm)?.sqrt()
                <= self.eps_2 * (self.backend.dot(&x, &x)?.sqrt() + self.eps_2)
            {
                break; // GRCOV_EXCL_LINE
            }

            self.backend.copy_to_v(&x, &mut x_new)?;
            self.backend.add_v(-1., &h_lm, &mut x_new)?;

            let fx_new = self.calc_fx(&zero, &x_new, &bhb, &mut tmp, &mut t)?;

            self.backend.copy_to_v(&g, &mut tmp_vec)?;
            self.backend.add_v(mu, &h_lm, &mut tmp_vec)?;

            let l0_lhlm = self.backend.dot(&h_lm, &tmp_vec)? / 2.;

            let rho = (fx - fx_new) / l0_lhlm;
            fx = fx_new;

            if rho > 0. {
                self.backend.copy_to_v(&x_new, &mut x)?;

                self.make_t(&zero, &x, &mut t)?;

                self.calc_jtj_jtf(
                    &t,
                    &bhb,
                    &mut tth,
                    &mut bhb_tth,
                    &mut bhb_tth_i,
                    &mut a,
                    &mut g,
                )?;

                mu *= f32::max(1. / 3., f32::powi(1. - (2. * rho - 1.), 3));
                nu = 2.;
            } else {
                mu *= nu;
                nu *= 2.;
            }
        }

        let x = self.backend.to_host_v(x)?;
        generate_result(geometry, x, 1.0, self.constraint, filter)
    }
}

impl<D: Directivity, B: LinAlgBackend<D>> Gain for LM<D, B> {
    fn calc(&self, geometry: &Geometry) -> GainCalcResult {
        self.calc_impl(geometry, None)
    }

    fn calc_with_filter(
        &self,
        geometry: &Geometry,
        filter: HashMap<usize, BitVec<usize, Lsb0>>,
    ) -> GainCalcResult {
        self.calc_impl(geometry, Some(filter))
    }

    #[tracing::instrument(level = "debug", skip(self, _geometry), fields(?self.eps_1, ?self.eps_2, ?self.tau, ?self.k_max, ?self.constraint))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
        if tracing::enabled!(tracing::Level::DEBUG) {
            if tracing::enabled!(tracing::Level::TRACE) {
                self.foci
                    .iter()
                    .zip(self.amps.iter())
                    .enumerate()
                    .for_each(|(i, (f, a))| {
                        tracing::debug!("Foci[{}]: {:?}, {}", i, f, a);
                    });
            } else {
                let len = self.foci.len();
                tracing::debug!("Foci[{}]: {:?}, {}", 0, self.foci[0], self.amps[0]);
                if len > 2 {
                    tracing::debug!("︙");
                }
                if len > 1 {
                    tracing::debug!(
                        "Foci[{}]: {:?}, {}",
                        0,
                        self.foci[len - 1],
                        self.amps[len - 1]
                    );
                }
            }
        }
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use super::{super::super::NalgebraBackend, super::super::Pa, *};
    use autd3_driver::{autd3_device::AUTD3, defined::FREQ_40K, geometry::IntoDevice};

    #[test]
    fn test_lm_all() {
        let geometry: Geometry =
            Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)], FREQ_40K);
        let backend = Arc::new(NalgebraBackend::default());

        let g = LM::new(
            backend,
            [(Vector3::zeros(), 1. * Pa), (Vector3::zeros(), 1. * Pa)].into_iter(),
        )
        .with_eps_1(1e-3)
        .with_eps_2(1e-4)
        .with_tau(1e-2)
        .with_k_max(2)
        .with_initial(vec![1.0]);

        assert_eq!(g.eps_1(), 1e-3);
        assert_eq!(g.eps_2(), 1e-4);
        assert_eq!(g.tau(), 1e-2);
        assert_eq!(g.k_max(), 2);
        assert_eq!(g.initial(), &[1.0]);
        assert_eq!(
            g.constraint(),
            EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX)
        );

        assert_eq!(
            g.with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)))
                .calc(&geometry)
                .map(|res| {
                    let f = res(&geometry[0]);
                    geometry[0]
                        .iter()
                        .filter(|tr| f(tr) != Drive::null())
                        .count()
                }),
            Ok(geometry.num_transducers()),
        );
    }

    #[test]
    fn test_lm_filtered() {
        let geometry: Geometry = Geometry::new(
            vec![
                AUTD3::new(Vector3::zeros()).into_device(0),
                AUTD3::new(Vector3::zeros()).into_device(1),
            ],
            FREQ_40K,
        );
        let backend = Arc::new(NalgebraBackend::default());

        let g = LM::new(
            backend,
            [
                (Vector3::new(10., 10., 100.), 5e3 * Pa),
                (Vector3::new(-10., 10., 100.), 5e3 * Pa),
            ]
            .into_iter(),
        )
        .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)));

        let filter = geometry
            .iter()
            .take(1)
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        assert_eq!(
            g.calc_with_filter(&geometry, filter.clone()).map(|res| {
                let f = res(&geometry[0]);
                geometry[0]
                    .iter()
                    .filter(|tr| f(tr) != Drive::null())
                    .count()
            }),
            Ok(100),
        );
        assert_eq!(
            g.calc_with_filter(&geometry, filter).map(|res| {
                let f = res(&geometry[1]);
                geometry[1]
                    .iter()
                    .filter(|tr| f(tr) != Drive::null())
                    .count()
            }),
            Ok(0),
        );
    }
}
