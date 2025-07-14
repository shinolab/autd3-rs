use std::{num::NonZeroUsize, sync::Arc};

use crate::{
    Amplitude, Complex, HoloError, LinAlgBackend, Trans,
    constraint::EmissionConstraint,
    helper::{HoloCalculatorGenerator, generate_result},
};

use autd3_core::{acoustics::directivity::Directivity, derive::*, geometry::Point3};
use derive_more::Debug;
use zerocopy::{FromBytes, IntoBytes};

/// The option of [`LM`].
#[derive(Debug, Clone, PartialEq)]
pub struct LMOption<D: Directivity> {
    /// The stopping criteria.
    pub eps_1: f32,
    /// The relative step size.
    pub eps_2: f32,
    /// The damping parameter.
    pub tau: f32,
    /// The maximum number of iterations.
    pub k_max: NonZeroUsize,
    /// Initial values of the transducers' amplitudes.
    pub initial: Vec<f32>,
    /// The transducers' emission constraint.
    pub constraint: EmissionConstraint,
    #[doc(hidden)]
    #[debug(ignore)]
    pub __phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity> Default for LMOption<D> {
    fn default() -> Self {
        Self {
            eps_1: 1e-8,
            eps_2: 1e-8,
            tau: 1e-3,
            k_max: NonZeroUsize::new(5).unwrap(),
            initial: vec![],
            constraint: EmissionConstraint::Clamp(Intensity::MIN, Intensity::MAX),
            __phantom: std::marker::PhantomData,
        }
    }
}

/// Levenberg-Marquardt algorithm
///
/// See [^Levenberg, 1944] and [^Marquardt, 1963] for more details. The implementation is based on [^Madsen, et al., 2004].
///
/// [^Levenberg, 1944]: Levenberg, Kenneth. "A method for the solution of certain non-linear problems in least squares." Quarterly of applied mathematics 2.2 (1944): 164-168.
/// [^Marquardt, 1963]: Marquardt, Donald W. "An algorithm for least-squares estimation of nonlinear parameters." Journal of the society for Industrial and Applied Mathematics 11.2 (1963): 431-441.
/// [^Madsen, et al., 2004]: Madsen, Kaj, Hans Bruun Nielsen, and Ole Tingleff. "Methods for non-linear least squares problems." (2004).
#[derive(Gain, Debug)]
pub struct LM<D: Directivity, B: LinAlgBackend<D>> {
    /// The focal positions and amplitudes.
    pub foci: Vec<(Point3, Amplitude)>,
    /// The opinion of the Gain.
    pub option: LMOption<D>,
    #[debug("{}", tynm::type_name::<B>())]
    /// The backend of linear algebra calculation.
    pub backend: Arc<B>,
}

impl<D: Directivity, B: LinAlgBackend<D>> LM<D, B> {
    /// Create a new [`LM`].
    #[must_use]
    pub fn new(
        foci: impl IntoIterator<Item = (Point3, Amplitude)>,
        option: LMOption<D>,
        backend: Arc<B>,
    ) -> Self {
        Self {
            foci: foci.into_iter().collect(),
            option,
            backend,
        }
    }
}

impl<D: Directivity, B: LinAlgBackend<D>> LM<D, B> {
    fn make_t(
        backend: &B,
        zero: &B::VectorX,
        x: &B::VectorX,
        t: &mut B::VectorXc,
    ) -> Result<(), HoloError> {
        backend.make_complex2_v(zero, x, t)?;
        backend.scale_assign_cv(Complex::new(-1., 0.), t)?;
        backend.exp_assign_cv(t)
    }

    #[allow(clippy::too_many_arguments)]
    fn calc_jtj_jtf(
        backend: &B,
        t: &B::VectorXc,
        bhb: &B::MatrixXc,
        tth: &mut B::MatrixXc,
        bhb_tth: &mut B::MatrixXc,
        bhb_tth_i: &mut B::MatrixX,
        jtj: &mut B::MatrixX,
        jtf: &mut B::VectorX,
    ) -> Result<(), HoloError> {
        backend.gevv_c(
            Trans::NoTrans,
            Trans::ConjTrans,
            Complex::new(1., 0.),
            t,
            t,
            Complex::new(0., 0.),
            tth,
        )?;
        backend.hadamard_product_cm(bhb, tth, bhb_tth)?;
        backend.real_cm(bhb_tth, jtj)?;
        backend.imag_cm(bhb_tth, bhb_tth_i)?;
        backend.reduce_col(bhb_tth_i, jtf)
    }

    fn calc_fx(
        backend: &B,
        zero: &B::VectorX,
        x: &B::VectorX,
        bhb: &B::MatrixXc,
        tmp: &mut B::VectorXc,
        t: &mut B::VectorXc,
    ) -> Result<f32, HoloError> {
        backend.make_complex2_v(zero, x, t)?;
        backend.exp_assign_cv(t)?;
        backend.gemv_c(
            Trans::NoTrans,
            Complex::new(1., 0.),
            bhb,
            t,
            Complex::new(0., 0.),
            tmp,
        )?;
        Ok(backend.dot_c(t, tmp)?.re)
    }
}

impl<D: Directivity, B: LinAlgBackend<D>> Gain<'_> for LM<D, B> {
    type G = HoloCalculatorGenerator<f32>;

    fn init(
        self,
        geometry: &Geometry,
        env: &Environment,
        filter: &TransducerFilter,
    ) -> Result<Self::G, GainError> {
        let (foci, amps): (Vec<_>, Vec<_>) = self.foci.into_iter().unzip();

        let g = self
            .backend
            .generate_propagation_matrix(geometry, env, &foci, filter)?;

        let n = self.backend.cols_c(&g)?;
        let m = foci.len();

        let n_param = m + n;

        let bhb = {
            let mut bhb = self.backend.alloc_zeros_cm(n_param, n_param)?;

            let mut amps = self
                .backend
                .from_slice_cv(<[f32]>::ref_from_bytes(amps.as_bytes()).unwrap())?;
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
        self.backend
            .copy_from_slice_v(&self.option.initial, &mut x)?;

        let mut nu = 2.0;

        let zero = self.backend.alloc_zeros_v(n_param)?;

        let mut t = self.backend.alloc_cv(n_param)?;
        Self::make_t(&self.backend, &zero, &x, &mut t)?;

        let mut tth = self.backend.alloc_cm(n_param, n_param)?;
        let mut bhb_tth = self.backend.alloc_cm(n_param, n_param)?;
        let mut bhb_tth_i = self.backend.alloc_m(n_param, n_param)?;
        let mut a = self.backend.alloc_m(n_param, n_param)?;
        let mut g = self.backend.alloc_v(n_param)?;
        Self::calc_jtj_jtf(
            &self.backend,
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

        let mut mu = self.option.tau * a_max;

        let mut tmp = self.backend.alloc_zeros_cv(n_param)?;
        let mut fx = Self::calc_fx(&self.backend, &zero, &x, &bhb, &mut tmp, &mut t)?;

        let ones = vec![1.0; n_param];
        let ones = self.backend.from_slice_v(&ones)?;
        let mut identity = self.backend.alloc_m(n_param, n_param)?;
        self.backend.create_diagonal(&ones, &mut identity)?;

        let mut h_lm = self.backend.alloc_v(n_param)?;
        let mut x_new = self.backend.alloc_v(n_param)?;
        let mut tmp_mat = self.backend.alloc_m(n_param, n_param)?;
        let mut tmp_vec = self.backend.alloc_v(n_param)?;
        for _ in 0..self.option.k_max.get() {
            if self.backend.max_abs_v(&g)? <= self.option.eps_1 {
                break; // GRCOV_EXCL_LINE
            }

            self.backend.copy_to_m(&a, &mut tmp_mat)?;
            self.backend.add_m(mu, &identity, &mut tmp_mat)?;

            self.backend.copy_to_v(&g, &mut h_lm)?;

            self.backend.solve_inplace(&tmp_mat, &mut h_lm)?;

            if self.backend.dot(&h_lm, &h_lm)?.sqrt()
                <= self.option.eps_2 * (self.backend.dot(&x, &x)?.sqrt() + self.option.eps_2)
            {
                break; // GRCOV_EXCL_LINE
            }

            self.backend.copy_to_v(&x, &mut x_new)?;
            self.backend.add_v(-1., &h_lm, &mut x_new)?;

            let fx_new = Self::calc_fx(&self.backend, &zero, &x_new, &bhb, &mut tmp, &mut t)?;

            self.backend.copy_to_v(&g, &mut tmp_vec)?;
            self.backend.add_v(mu, &h_lm, &mut tmp_vec)?;

            let l0_lhlm = self.backend.dot(&h_lm, &tmp_vec)? / 2.;

            let rho = (fx - fx_new) / l0_lhlm;
            fx = fx_new;

            if rho > 0. {
                self.backend.copy_to_v(&x_new, &mut x)?;

                Self::make_t(&self.backend, &zero, &x, &mut t)?;

                Self::calc_jtj_jtf(
                    &self.backend,
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
        generate_result(geometry, x, 1.0, self.option.constraint, filter)
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::gain::{Drive, GainCalculator, GainCalculatorGenerator};

    use crate::tests::create_geometry;

    use super::{super::super::NalgebraBackend, super::super::Pa, *};

    #[test]
    fn test_lm_all() {
        let geometry = create_geometry(1, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = LM::new(
            vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            LMOption {
                k_max: NonZeroUsize::new(2).unwrap(),
                constraint: EmissionConstraint::Uniform(Intensity::MAX),
                ..Default::default()
            },
            backend,
        );

        assert_eq!(
            g.init(
                &geometry,
                &Environment::new(),
                &TransducerFilter::all_enabled()
            )
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
    fn test_lm_filtered() {
        let geometry = create_geometry(2, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = LM {
            foci: vec![
                (Point3::new(10., 10., 100.), 5e3 * Pa),
                (Point3::new(-10., 10., 100.), 5e3 * Pa),
            ],
            backend,
            option: LMOption {
                k_max: NonZeroUsize::new(2).unwrap(),
                constraint: EmissionConstraint::Uniform(Intensity::MAX),
                ..Default::default()
            },
        };

        let filter = TransducerFilter::from_fn(&geometry, |dev| {
            if dev.idx() == 0 {
                Some(|tr: &Transducer| tr.idx() < 100)
            } else {
                None
            }
        });
        let mut g = g.init(&geometry, &Environment::new(), &filter).unwrap();
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
