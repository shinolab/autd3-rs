use std::{collections::HashMap, sync::Arc};

use crate::{
    constraint::EmissionConstraint, helper::generate_result, impl_holo, Amplitude, Complex,
    LinAlgBackend, Trans,
};

use autd3_driver::{
    derive::*,
    geometry::{Geometry, Vector3},
};

/// Gain to produce multiple foci with GS algorithm
///
/// Reference
/// * Marzo, Asier, and Bruce W. Drinkwater. "Holographic acoustic tweezers." Proceedings of the National Academy of Sciences 116.1 (2019): 84-89.
#[derive(Gain)]
pub struct GS<B: LinAlgBackend + 'static> {
    foci: Vec<Vector3>,
    amps: Vec<Amplitude>,
    repeat: usize,
    constraint: EmissionConstraint,
    backend: Arc<B>,
}

impl_holo!(B, GS<B>);

impl<B: LinAlgBackend + 'static> GS<B> {
    pub const fn new(backend: Arc<B>) -> Self {
        Self {
            foci: vec![],
            amps: vec![],
            repeat: 100,
            backend,
            constraint: EmissionConstraint::DontCare,
        }
    }

    pub fn with_repeat(self, repeat: usize) -> Self {
        Self { repeat, ..self }
    }

    pub const fn repeat(&self) -> usize {
        self.repeat
    }
}

impl<B: LinAlgBackend> Gain for GS<B> {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        let g = self
            .backend
            .generate_propagation_matrix(geometry, &self.foci, &filter)?;

        let m = self.foci.len();
        let n = self.backend.cols_c(&g)?;
        let ones = vec![1.; n];

        let mut b = self.backend.alloc_cm(n, m)?;
        self.backend.gen_back_prop(n, m, &g, &mut b)?;

        let mut q = self.backend.from_slice_cv(&ones)?;

        let q0 = self.backend.from_slice_cv(&ones)?;

        let amps = self.backend.from_slice_cv(self.amps_as_slice())?;
        let mut p = self.backend.alloc_zeros_cv(m)?;
        (0..self.repeat).try_for_each(|_| -> Result<(), AUTDInternalError> {
            self.backend.scaled_to_assign_cv(&q0, &mut q)?;
            self.backend.gemv_c(
                Trans::NoTrans,
                Complex::new(1., 0.),
                &g,
                &q,
                Complex::new(0., 0.),
                &mut p,
            )?;
            self.backend.scaled_to_assign_cv(&amps, &mut p)?;

            self.backend.gemv_c(
                Trans::NoTrans,
                Complex::new(1., 0.),
                &b,
                &p,
                Complex::new(0., 0.),
                &mut q,
            )?;
            Ok(())
        })?;
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
    fn test_gs_all() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
        let backend = NalgebraBackend::new().unwrap();

        let g = GS::new(backend)
            .with_repeat(50)
            .add_focus(Vector3::zeros(), 1. * Pascal)
            .add_foci_from_iter([(Vector3::zeros(), 1. * Pascal)]);

        assert_eq!(g.repeat(), 50);
        assert_eq!(g.constraint(), EmissionConstraint::DontCare);
        assert!(g
            .foci()
            .all(|(&p, &a)| p == Vector3::zeros() && a == 1. * Pascal));

        let _ = g.calc(&geometry, GainFilter::All);
        let _ = g.operation();
    }

    #[test]
    fn test_gs_filtered() {
        let geometry: Geometry = Geometry::new(vec![
            AUTD3::new(Vector3::zeros()).into_device(0),
            AUTD3::new(Vector3::zeros()).into_device(1),
        ]);
        let backend = NalgebraBackend::new().unwrap();

        let g = GS::new(backend)
            .add_focus(Vector3::new(10., 10., 100.), 5e3 * Pascal)
            .add_foci_from_iter([(Vector3::new(-10., 10., 100.), 5e3 * Pascal)])
            .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)));

        let filter = geometry
            .iter()
            .take(1)
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        assert_eq!(
            g.calc(&geometry, GainFilter::Filter(&filter))
                .map(|res| res[&0].iter().filter(|&&d| d != Drive::null()).count()),
            Ok(100),
        );
        assert_eq!(
            g.calc(&geometry, GainFilter::Filter(&filter))
                .map(|res| res[&1].iter().filter(|&&d| d != Drive::null()).count()),
            Ok(0),
        );
    }
}
