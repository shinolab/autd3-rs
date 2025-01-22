use std::{collections::HashMap, sync::Arc};

use crate::{
    constraint::EmissionConstraint,
    helper::{generate_result, HoloContextGenerator},
    Amplitude, Complex, LinAlgBackend, Trans,
};

use autd3_core::{acoustics::directivity::Directivity, derive::*, geometry::Point3};
use derive_more::Debug;
use zerocopy::{FromBytes, IntoBytes};

/// The option of [`Naive`].
#[derive(Debug)]
pub struct NaiveOption<D: Directivity> {
    /// The transducers' emission constraint.
    pub constraint: EmissionConstraint,
    #[doc(hidden)]
    #[debug(ignore)]
    pub __phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity> Default for NaiveOption<D> {
    fn default() -> Self {
        Self {
            constraint: EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX),
            __phantom: std::marker::PhantomData,
        }
    }
}

/// Naive linear synthesis of simple focal solutions
#[derive(Gain, Debug)]
pub struct Naive<D: Directivity, B: LinAlgBackend<D>> {
    /// The focal positions and amplitudes.
    pub foci: Vec<(Point3, Amplitude)>,
    /// The opinion of the Gain.
    pub option: NaiveOption<D>,
    /// The backend of calculation.
    #[debug("{}", tynm::type_name::<B>())]
    pub backend: Arc<B>,
}

impl<D: Directivity, B: LinAlgBackend<D>> Gain for Naive<D, B> {
    type G = HoloContextGenerator<Complex>;

    fn init(self) -> Result<Self::G, GainError> {
        unimplemented!()
    }

    fn init_full(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
        _option: &DatagramOption,
    ) -> Result<Self::G, GainError> {
        let (foci, amps): (Vec<_>, Vec<_>) = self.foci.into_iter().unzip();

        let g = self
            .backend
            .generate_propagation_matrix(geometry, &foci, filter)?;

        let m = foci.len();
        let n = self.backend.cols_c(&g)?;

        let b = self.backend.gen_back_prop(n, m, &g)?;

        let p = self
            .backend
            .from_slice_cv(<[f32]>::ref_from_bytes(amps.as_bytes()).unwrap())?;
        let mut q = self.backend.alloc_zeros_cv(n)?;
        self.backend.gemv_c(
            Trans::NoTrans,
            Complex::new(1., 0.),
            &b,
            &p,
            Complex::new(0., 0.),
            &mut q,
        )?;

        let mut abs = self.backend.alloc_v(n)?;
        self.backend.norm_squared_cv(&q, &mut abs)?;
        let max_coefficient = self.backend.max_v(&abs)?.sqrt();
        let q = self.backend.to_host_cv(q)?;
        generate_result(geometry, q, max_coefficient, self.option.constraint, filter)
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::gain::{Drive, GainContext, GainContextGenerator};

    use crate::tests::create_geometry;

    use super::{super::super::NalgebraBackend, super::super::Pa, *};

    #[test]
    fn test_naive_all() {
        let geometry = create_geometry(1, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = Naive {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            backend,
            option: NaiveOption {
                constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
                ..Default::default()
            },
        };

        assert_eq!(
            g.init_full(&geometry, None, &DatagramOption::default())
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
    fn test_naive_all_disabled() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2, 1);
        geometry[0].enable = false;
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = Naive {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            backend,
            option: NaiveOption {
                constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
                ..Default::default()
            },
        };

        let mut g = g.init_full(&geometry, None, &DatagramOption::default())?;
        let f = g.generate(&geometry[1]);
        assert_eq!(
            geometry[1]
                .iter()
                .filter(|tr| f.calc(tr) != Drive::NULL)
                .count(),
            geometry[1].num_transducers()
        );

        Ok(())
    }

    #[test]
    fn test_naive_filtered() {
        let geometry = create_geometry(1, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = Naive {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            backend,
            option: NaiveOption {
                constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
                ..Default::default()
            },
        };

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect();
        assert_eq!(
            g.init_full(&geometry, Some(&filter), &DatagramOption::default())
                .map(|mut res| {
                    let f = res.generate(&geometry[0]);
                    geometry[0]
                        .iter()
                        .filter(|tr| f.calc(tr) != Drive::NULL)
                        .count()
                }),
            Ok(100),
        )
    }

    #[test]
    fn test_naive_filtered_disabled() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2, 1);
        geometry[0].enable = false;
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = Naive {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            backend,
            option: NaiveOption {
                constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
                ..Default::default()
            },
        };

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect();
        let mut g = g.init_full(&geometry, Some(&filter), &DatagramOption::default())?;
        let f = g.generate(&geometry[1]);
        assert_eq!(
            geometry[1]
                .iter()
                .filter(|tr| f.calc(tr) != Drive::NULL)
                .count(),
            100
        );

        Ok(())
    }
}
