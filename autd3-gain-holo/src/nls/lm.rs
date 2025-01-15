use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use crate::{
    constraint::EmissionConstraint,
    helper::{generate_result, HoloContextGenerator},
    Amplitude, Complex, HoloError, LinAlgBackend, Trans,
};

use autd3_core::{acoustics::directivity::Directivity, derive::*, geometry::Point3};
use autd3_derive::Builder;
use derive_more::Debug;
use zerocopy::{FromBytes, IntoBytes};

/// Levenberg-Marquardt algorithm
///
/// See [^Levenberg, 1944] and [^Marquardt, 1963] for more details. The implementation is based on [^Madsen, et al., 2004].
///
/// [^Levenberg, 1944]: Levenberg, Kenneth. "A method for the solution of certain non-linear problems in least squares." Quarterly of applied mathematics 2.2 (1944): 164-168.
/// [^Marquardt, 1963]: Marquardt, Donald W. "An algorithm for least-squares estimation of nonlinear parameters." Journal of the society for Industrial and Applied Mathematics 11.2 (1963): 431-441.
/// [^Madsen, et al., 2004]: Madsen, Kaj, Hans Bruun Nielsen, and Ole Tingleff. "Methods for non-linear least squares problems." (2004).
#[derive(Gain, Builder, Debug)]
pub struct LM<D: Directivity, B: LinAlgBackend<D>> {
    #[get(ref)]
    /// The focal positions.
    foci: Vec<Point3>,
    #[get(ref)]
    /// The focal amplitudes.
    amps: Vec<Amplitude>,
    #[get]
    #[set]
    /// The stopping criteria.
    eps_1: f32,
    #[get]
    #[set]
    /// The relative step size.
    eps_2: f32,
    #[get]
    #[set]
    /// The damping parameter.
    tau: f32,
    #[get]
    #[set]
    /// The maximum number of iterations.
    k_max: NonZeroUsize,
    #[get(ref)]
    #[set(no_const)]
    /// Initial values of the transducers' amplitudes.
    initial: Vec<f32>,
    #[get]
    #[set]
    #[set]
    /// The transducers' emission constraint.
    constraint: EmissionConstraint,
    #[debug("{}", tynm::type_name::<B>())]
    backend: Arc<B>,
    #[debug(ignore)]
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity, B: LinAlgBackend<D>> LM<D, B> {
    /// Creates a new [`LM`].
    pub fn new(backend: Arc<B>, iter: impl IntoIterator<Item = (Point3, Amplitude)>) -> Self {
        let (foci, amps) = iter.into_iter().unzip();
        Self {
            foci,
            amps,
            eps_1: 1e-8,
            eps_2: 1e-8,
            tau: 1e-3,
            k_max: NonZeroUsize::new(5).unwrap(),
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

impl<D: Directivity, B: LinAlgBackend<D>> Gain for LM<D, B> {
    type G = HoloContextGenerator<f32>;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
    ) -> Result<Self::G, GainError> {
        let g = self
            .backend
            .generate_propagation_matrix(geometry, &self.foci, filter)?;

        let n = self.backend.cols_c(&g)?;
        let m = self.foci.len();

        let n_param = m + n;

        let bhb = {
            let mut bhb = self.backend.alloc_zeros_cm(n_param, n_param)?;

            let mut amps = self
                .backend
                .from_slice_cv(<[f32]>::ref_from_bytes(self.amps.as_bytes()).unwrap())?;
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
        for _ in 0..self.k_max.get() {
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

#[cfg(test)]
mod tests {
    use autd3_core::gain::{Drive, GainContext, GainContextGenerator};

    use crate::tests::create_geometry;

    use super::{super::super::NalgebraBackend, super::super::Pa, *};

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_lm_all() {
        let geometry = create_geometry(1, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = LM::new(
            backend,
            [(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
        )
        .with_eps_1(1e-3)
        .with_eps_2(1e-4)
        .with_tau(1e-2)
        .with_k_max(NonZeroUsize::new(2).unwrap())
        .with_initial(vec![1.0]);

        assert_eq!(g.eps_1(), 1e-3);
        assert_eq!(g.eps_2(), 1e-4);
        assert_eq!(g.tau(), 1e-2);
        assert_eq!(g.k_max().get(), 2);
        assert_eq!(g.initial(), &[1.0]);
        assert_eq!(
            g.constraint(),
            EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX)
        );

        assert_eq!(
            g.with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)))
                .init(&geometry, None)
                .map(|mut res| {
                    let f = res.generate(&geometry[0]);
                    geometry[0]
                        .iter()
                        .filter(|tr| f.calc(tr) != Drive::NULL)
                        .count()
                }),
            Ok(geometry.num_transducers()),
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_lm_filtered() {
        let geometry = create_geometry(2, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = LM::new(
            backend,
            [
                (Point3::new(10., 10., 100.), 5e3 * Pa),
                (Point3::new(-10., 10., 100.), 5e3 * Pa),
            ],
        )
        .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)));

        let filter = geometry
            .iter()
            .take(1)
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        let mut g = g.init(&geometry, Some(&filter)).unwrap();
        assert_eq!(
            {
                let f = g.generate(&geometry[0]);
                geometry[0]
                    .iter()
                    .filter(|tr| f.calc(tr) != Drive::NULL)
                    .count()
            },
            100,
        );
        assert_eq!(
            {
                let f = g.generate(&geometry[1]);
                geometry[1]
                    .iter()
                    .filter(|tr| f.calc(tr) != Drive::NULL)
                    .count()
            },
            0,
        );
    }
}
