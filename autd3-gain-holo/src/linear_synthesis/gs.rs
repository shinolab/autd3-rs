use std::{collections::HashMap, sync::Arc};

use crate::{
    constraint::EmissionConstraint, helper::generate_result, impl_holo, Amplitude, Complex,
    LinAlgBackend, Trans,
};

use autd3_driver::{acoustics::directivity::Directivity, derive::*, geometry::Vector3};

/// Gain to produce multiple foci with GS algorithm
///
/// Reference
/// * Marzo, Asier, and Bruce W. Drinkwater. "Holographic acoustic tweezers." Proceedings of the National Academy of Sciences 116.1 (2019): 84-89.
#[derive(Gain, Builder)]
#[no_const]
pub struct GS<D: Directivity + 'static, B: LinAlgBackend<D> + 'static> {
    foci: Vec<Vector3>,
    amps: Vec<Amplitude>,
    #[getset]
    repeat: usize,
    constraint: EmissionConstraint,
    backend: Arc<B>,
    _phantom: std::marker::PhantomData<D>,
}

impl_holo!(D, B, GS<D, B>);

impl<D: Directivity + 'static, B: LinAlgBackend<D> + 'static> GS<D, B> {
    pub const fn new(backend: Arc<B>) -> Self {
        Self {
            foci: vec![],
            amps: vec![],
            repeat: 100,
            backend,
            constraint: EmissionConstraint::DontCare,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<D: Directivity, B: LinAlgBackend<D>> Gain for GS<D, B> {
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

        let b = self.backend.gen_back_prop(n, m, &g)?;

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

        let q = self.backend.to_host_cv(q)?;
        let max_coefficient = q.camax().abs();
        generate_result(geometry, q, max_coefficient, &self.constraint, filter)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::super::NalgebraBackend, super::super::Pa, *};
    use autd3_driver::{autd3_device::AUTD3, geometry::IntoDevice};

    #[test]
    fn test_gs_all() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
        let backend = Arc::new(NalgebraBackend::default());

        let g = GS::new(backend)
            .with_repeat(50)
            .add_focus(Vector3::zeros(), 1. * Pa)
            .add_foci_from_iter([(Vector3::zeros(), 1. * Pa)]);

        assert_eq!(g.repeat(), 50);
        assert_eq!(g.constraint(), EmissionConstraint::DontCare);
        assert!(g
            .foci()
            .all(|(&p, &a)| p == Vector3::zeros() && a == 1. * Pa));

        assert_eq!(
            g.with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)))
                .calc(&geometry, GainFilter::All)
                .map(|res| res[&0].iter().filter(|&&d| d != Drive::null()).count()),
            Ok(geometry.num_transducers()),
        );
    }

    #[test]
    fn test_gs_filtered() {
        let geometry: Geometry = Geometry::new(vec![
            AUTD3::new(Vector3::zeros()).into_device(0),
            AUTD3::new(Vector3::zeros()).into_device(1),
        ]);
        let backend = Arc::new(NalgebraBackend::default());

        let g = GS::new(backend)
            .add_focus(Vector3::new(10., 10., 100.), 5e3 * Pa)
            .add_foci_from_iter([(Vector3::new(-10., 10., 100.), 5e3 * Pa)])
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
