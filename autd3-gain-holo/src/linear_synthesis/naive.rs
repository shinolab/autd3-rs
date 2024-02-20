use std::{collections::HashMap, sync::Arc};

use crate::{
    constraint::EmissionConstraint, helper::generate_result, impl_holo, Amplitude, Complex,
    LinAlgBackend, Trans,
};

use autd3_driver::{derive::*, geometry::Vector3};

/// Gain to produce multiple foci with naive linear synthesis
#[derive(Gain)]
pub struct Naive<B: LinAlgBackend + 'static> {
    foci: Vec<Vector3>,
    amps: Vec<Amplitude>,
    constraint: EmissionConstraint,
    backend: Arc<B>,
}

impl_holo!(B, Naive<B>);

impl<B: LinAlgBackend + 'static> Naive<B> {
    pub const fn new(backend: Arc<B>) -> Self {
        Self {
            foci: vec![],
            amps: vec![],
            backend,
            constraint: EmissionConstraint::DontCare,
        }
    }
}

impl<B: LinAlgBackend> Gain for Naive<B> {
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

        let mut b = self.backend.alloc_cm(n, m)?;
        self.backend.gen_back_prop(n, m, &g, &mut b)?;

        let p = self.backend.from_slice_cv(self.amps_as_slice())?;
        let mut q = self.backend.alloc_zeros_cv(n)?;
        self.backend.gemv_c(
            Trans::NoTrans,
            Complex::new(1., 0.),
            &b,
            &p,
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
    fn test_naive_all() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
        let backend = NalgebraBackend::new().unwrap();

        let g = Naive::new(backend)
            .add_focus(Vector3::zeros(), 1. * Pascal)
            .add_foci_from_iter([(Vector3::zeros(), 1. * Pascal)]);

        assert_eq!(g.constraint(), EmissionConstraint::DontCare);
        assert!(g
            .foci()
            .all(|(&p, &a)| p == Vector3::zeros() && a == 1. * Pascal));

        let _ = g.calc(&geometry, GainFilter::All);
        let _ = g.operation_with_segment(Segment::S0, true);
    }

    #[test]
    fn test_naive_filtered() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
        let backend = NalgebraBackend::new().unwrap();

        let g = Naive::new(backend)
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
