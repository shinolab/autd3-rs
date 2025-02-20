use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use crate::{
    Amplitude, Complex, LinAlgBackend, Trans,
    constraint::EmissionConstraint,
    helper::{HoloCalculatorGenerator, generate_result},
};

use autd3_core::{acoustics::directivity::Directivity, derive::*, geometry::Point3};
use derive_more::Debug;
use derive_new::new;
use zerocopy::{FromBytes, IntoBytes};

/// The option of [`GS`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GSOption<D: Directivity> {
    /// The number of iterations.
    pub repeat: NonZeroUsize,
    /// The transducers' emission constraint.
    pub constraint: EmissionConstraint,
    #[debug(ignore)]
    #[doc(hidden)]
    pub __phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity> Default for GSOption<D> {
    fn default() -> Self {
        Self {
            repeat: NonZeroUsize::new(100).unwrap(),
            constraint: EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX),
            __phantom: std::marker::PhantomData,
        }
    }
}

/// Gerchberg-Saxton algorithm
///
/// See [Marzo, et al., 2019](https://www.pnas.org/doi/full/10.1073/pnas.1813047115) for more details.
#[derive(Gain, Debug, new)]
pub struct GS<D: Directivity, B: LinAlgBackend<D>> {
    /// The focal positions and amplitudes.
    pub foci: Vec<(Point3, Amplitude)>,
    /// The opinion of the Gain.
    pub option: GSOption<D>,
    /// The backend of calculation.
    #[debug("{}", tynm::type_name::<B>())]
    pub backend: Arc<B>,
}

impl<D: Directivity, B: LinAlgBackend<D>> Gain for GS<D, B> {
    type G = HoloCalculatorGenerator<Complex>;

    // GRCOV_EXCL_START
    fn init(self) -> Result<Self::G, GainError> {
        unimplemented!()
    }
    // GRCOV_EXCL_STOP

    fn init_full(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
        _: bool,
    ) -> Result<Self::G, GainError> {
        let (foci, amps): (Vec<_>, Vec<_>) = self.foci.into_iter().unzip();

        let g = self
            .backend
            .generate_propagation_matrix(geometry, &foci, filter)?;

        let m = foci.len();
        let n = self.backend.cols_c(&g)?;
        let ones = vec![1.; n];

        let b = self.backend.gen_back_prop(n, m, &g)?;

        let mut q = self.backend.from_slice_cv(&ones)?;

        let q0 = self.backend.from_slice_cv(&ones)?;

        let amps = self
            .backend
            .from_slice_cv(<[f32]>::ref_from_bytes(amps.as_bytes()).unwrap())?;
        let mut p = self.backend.alloc_zeros_cv(m)?;
        (0..self.option.repeat.get()).try_for_each(|_| -> Result<(), GainError> {
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

        let mut abs = self.backend.alloc_v(n)?;
        self.backend.norm_squared_cv(&q, &mut abs)?;
        let max_coefficient = self.backend.max_v(&abs)?.sqrt();
        let q = self.backend.to_host_cv(q)?;
        generate_result(geometry, q, max_coefficient, self.option.constraint, filter)
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::gain::{Drive, GainCalculator, GainCalculatorGenerator};

    use crate::tests::create_geometry;

    use super::{super::super::NalgebraBackend, super::super::Pa, *};

    #[test]
    fn test_gs_all() {
        let geometry = create_geometry(1, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = GS {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            backend,
            option: GSOption {
                repeat: NonZeroUsize::new(5).unwrap(),
                constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
                ..Default::default()
            },
        };

        assert_eq!(
            g.init_full(&geometry, None, false).map(|mut res| {
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
    fn test_gs_filtered() {
        let geometry = create_geometry(2, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = GS {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            backend,
            option: GSOption {
                repeat: NonZeroUsize::new(5).unwrap(),
                constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
                ..Default::default()
            },
        };

        let filter = geometry
            .iter()
            .take(1)
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        let mut g = g.init_full(&geometry, Some(&filter), false).unwrap();
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
