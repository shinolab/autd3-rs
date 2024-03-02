use std::{collections::HashMap, sync::Arc};

use rand::Rng;

use crate::{
    constraint::EmissionConstraint, helper::generate_result, impl_holo, Amplitude, Complex,
    LinAlgBackend, Trans,
};

use autd3_driver::{derive::*, geometry::Vector3};

/// Gain to produce multiple foci by solving Semi-Denfinite Programming
///
/// Reference
/// * Inoue, Seki, Yasutoshi Makino, and Hiroyuki Shinoda. "Active touch perception produced by airborne ultrasonic haptic hologram." 2015 IEEE World Haptics Conference (WHC). IEEE, 2015.
#[derive(Gain, Builder)]
#[no_const]
pub struct SDP<B: LinAlgBackend + 'static> {
    foci: Vec<Vector3>,
    amps: Vec<Amplitude>,
    #[getset]
    alpha: float,
    #[getset]
    lambda: float,
    #[getset]
    repeat: usize,
    constraint: EmissionConstraint,
    backend: Arc<B>,
}

impl_holo!(B, SDP<B>);

impl<B: LinAlgBackend + 'static> SDP<B> {
    pub const fn new(backend: Arc<B>) -> Self {
        Self {
            foci: vec![],
            amps: vec![],
            alpha: 1e-3,
            lambda: 0.9,
            repeat: 100,
            backend,
            constraint: EmissionConstraint::DontCare,
        }
    }
}

impl<B: LinAlgBackend> Gain for SDP<B> {
    #[allow(non_snake_case)]
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        let G = self
            .backend
            .generate_propagation_matrix(geometry, &self.foci, &filter)?;

        let m = self.foci.len();
        let n = self.backend.cols_c(&G)?;

        let zeros = self.backend.alloc_zeros_cv(m)?;
        let ones = self.backend.from_slice_cv(&vec![1.; m])?;

        let P = {
            let mut P = self.backend.alloc_zeros_cm(m, m)?;
            let amps = self.backend.from_slice_cv(self.amps_as_slice())?;
            self.backend.create_diagonal_c(&amps, &mut P)?;
            P
        };

        let G_inv = {
            let mut G_inv = self.backend.alloc_zeros_cm(n, m)?;
            let mut u_ = self.backend.alloc_cm(m, m)?;
            let mut s = self.backend.alloc_cm(n, m)?;
            let mut vt = self.backend.alloc_cm(n, n)?;
            let mut buf = self.backend.alloc_zeros_cm(n, m)?;
            let G_ = self.backend.clone_cm(&G)?;
            self.backend.pseudo_inverse_svd(
                G_, self.alpha, &mut u_, &mut s, &mut vt, &mut buf, &mut G_inv,
            )?;
            G_inv
        };

        let M = {
            let mut M = self.backend.alloc_cm(m, m)?;

            // M = I
            self.backend.create_diagonal_c(&ones, &mut M)?;

            // M = I - GG^{-1}
            self.backend.gemm_c(
                Trans::NoTrans,
                Trans::NoTrans,
                Complex::new(-1., 0.),
                &G,
                &G_inv,
                Complex::new(1., 0.),
                &mut M,
            )?;

            // tmp = P (I - GG^{-1})
            let mut tmp = self.backend.alloc_zeros_cm(m, m)?;
            self.backend.gemm_c(
                Trans::NoTrans,
                Trans::NoTrans,
                Complex::new(1., 0.),
                &P,
                &M,
                Complex::new(0., 0.),
                &mut tmp,
            )?;

            // M = P (I - GG^{-1}) P
            self.backend.gemm_c(
                Trans::NoTrans,
                Trans::NoTrans,
                Complex::new(1., 0.),
                &tmp,
                &P,
                Complex::new(0., 0.),
                &mut M,
            )?;
            M
        };

        // Block coordinate descent
        let u = {
            let mut U = self.backend.alloc_cm(m, m)?;
            self.backend.create_diagonal_c(&ones, &mut U)?;

            let mut rng = rand::thread_rng();

            let mut x = self.backend.alloc_zeros_cv(m)?;
            let mut Mc = self.backend.alloc_cv(m)?;
            (0..self.repeat).try_for_each(|_| -> Result<(), AUTDInternalError> {
                let i = rng.gen_range(0..m);

                self.backend.get_col_c(&M, i, &mut Mc)?;
                self.backend.set_cv(i, Complex::new(0., 0.), &mut Mc)?;

                self.backend.gemv_c(
                    Trans::NoTrans,
                    Complex::new(1., 0.),
                    &U,
                    &Mc,
                    Complex::new(0., 0.),
                    &mut x,
                )?;

                let gamma = self.backend.dot_c(&x, &Mc)?;
                if gamma.re > 0. {
                    self.backend.scale_assign_cv(
                        Complex::new(-(self.lambda / gamma.re).sqrt(), 0.),
                        &mut x,
                    )?;

                    self.backend.set_col_c(&x, i, 0, i, &mut U)?;
                    self.backend.set_col_c(&x, i, i + 1, m, &mut U)?;
                    self.backend.conj_assign_v(&mut x)?;
                    self.backend.set_row_c(&x, i, 0, i, &mut U)?;
                    self.backend.set_row_c(&x, i, i + 1, m, &mut U)?;
                } else {
                    self.backend.set_col_c(&zeros, i, 0, i, &mut U)?;
                    self.backend.set_col_c(&zeros, i, i + 1, m, &mut U)?;
                    self.backend.set_row_c(&zeros, i, 0, i, &mut U)?;
                    self.backend.set_row_c(&zeros, i, i + 1, m, &mut U)?;
                }
                Ok(())
            })?;

            self.backend.max_eigen_vector_c(U)?
        };

        let mut ut = self.backend.alloc_zeros_cv(m)?;
        self.backend.gemv_c(
            Trans::NoTrans,
            Complex::new(1., 0.),
            &P,
            &u,
            Complex::new(0., 0.),
            &mut ut,
        )?;

        let mut q = self.backend.alloc_zeros_cv(n)?;
        self.backend.gemv_c(
            Trans::NoTrans,
            Complex::new(1., 0.),
            &G_inv,
            &ut,
            Complex::new(0., 0.),
            &mut q,
        )?;

        generate_result(
            geometry,
            self.backend.to_host_cv(q)?,
            &self.constraint,
            filter,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{super::super::NalgebraBackend, super::super::Pascal, *};
    use autd3_driver::{autd3_device::AUTD3, geometry::IntoDevice};

    #[test]
    fn test_sdp_all() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
        let backend = NalgebraBackend::new().unwrap();

        let g = SDP::new(backend)
            .with_alpha(0.1)
            .with_lambda(0.9)
            .with_repeat(10)
            .add_focus(Vector3::new(10., 10., 100.), 5e3 * Pascal)
            .add_foci_from_iter([(Vector3::new(10., 10., 100.), 5e3 * Pascal)]);

        assert_eq!(g.alpha(), 0.1);
        assert_eq!(g.lambda(), 0.9);
        assert_eq!(g.repeat(), 10);
        assert_eq!(g.constraint(), EmissionConstraint::DontCare);
        assert!(g
            .foci()
            .all(|(&p, &a)| p == Vector3::new(10., 10., 100.) && a == 5e3 * Pascal));

        let _ = g.calc(&geometry, GainFilter::All);
        let _ = g.operation_with_segment(Segment::S0, true);
    }

    #[test]
    fn test_sdp_filtered() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
        let backend = NalgebraBackend::new().unwrap();

        let g = SDP::new(backend)
            .add_focus(Vector3::new(10., 10., 100.), 5e3 * Pascal)
            .add_foci_from_iter([(Vector3::new(-10., 10., 100.), 5e3 * Pascal)])
            .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)));

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        assert_eq!(
            g.calc(&geometry, GainFilter::Filter(&filter))
                .map(|res| res[&0].iter().filter(|&&d| d != Drive::null()).count()),
            Ok(100),
        )
    }
}
