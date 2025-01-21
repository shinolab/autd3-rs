use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use crate::{
    constraint::EmissionConstraint,
    helper::{generate_result, HoloContextGenerator},
    Amplitude, Complex, LinAlgBackend, Trans,
};

use autd3_core::{acoustics::directivity::Directivity, derive::*, geometry::Point3};
use derive_more::Debug;
use zerocopy::{FromBytes, IntoBytes};

#[derive(Debug)]
pub struct GSPATOption<D: Directivity> {
    /// The number of iterations.
    pub repeat: NonZeroUsize,
    /// The transducers' emission constraint.
    pub constraint: EmissionConstraint,
    /// The segment to write the data.
    pub segment: Segment,
    /// The mode when switching the segment.
    pub transition_mode: Option<TransitionMode>,
    #[debug(ignore)]
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity> Default for GSPATOption<D> {
    fn default() -> Self {
        Self {
            repeat: NonZeroUsize::new(100).unwrap(),
            constraint: EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX),
            segment: Segment::S0,
            transition_mode: Some(TransitionMode::Immediate),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Gershberg-Saxon for Phased Arrays of Transducers
///
/// See [Plasencia, et al., 2020](https://dl.acm.org/doi/10.1145/3386569.3392492) for more details.
#[derive(Gain, Debug)]
pub struct GSPAT<D: Directivity, B: LinAlgBackend<D>> {
    /// The focal positions and amplitudes.
    foci: Vec<(Point3, Amplitude)>,
    /// The opinion of the Gain.
    option: GSPATOption<D>,
    /// The backend of calculation.
    #[debug("{}", tynm::type_name::<B>())]
    backend: Arc<B>,
}

impl<D: Directivity, B: LinAlgBackend<D>> Gain for GSPAT<D, B> {
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

        let mut q = self.backend.alloc_zeros_cv(n)?;

        let amps = self
            .backend
            .from_slice_cv(<[f32]>::ref_from_bytes(amps.as_bytes()).unwrap())?;

        let b = self.backend.gen_back_prop(n, m, &g)?;

        let mut r = self.backend.alloc_zeros_cm(m, m)?;
        self.backend.gemm_c(
            Trans::NoTrans,
            Trans::NoTrans,
            Complex::new(1., 0.),
            &g,
            &b,
            Complex::new(0., 0.),
            &mut r,
        )?;

        let mut p = self.backend.clone_cv(&amps)?;
        let mut gamma = self.backend.clone_cv(&amps)?;
        self.backend.gemv_c(
            Trans::NoTrans,
            Complex::new(1., 0.),
            &r,
            &p,
            Complex::new(0., 0.),
            &mut gamma,
        )?;
        (0..self.option.repeat.get()).try_for_each(|_| -> Result<(), GainError> {
            self.backend.scaled_to_cv(&gamma, &amps, &mut p)?;
            self.backend.gemv_c(
                Trans::NoTrans,
                Complex::new(1., 0.),
                &r,
                &p,
                Complex::new(0., 0.),
                &mut gamma,
            )?;
            Ok(())
        })?;

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
    fn test_gspat_all() {
        let geometry = create_geometry(1, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = GSPAT {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            backend,
            option: GSPATOption {
                repeat: NonZeroUsize::new(5).unwrap(),
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
    fn test_gspat_filtered() {
        let geometry = create_geometry(1, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = GSPAT {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            backend,
            option: GSPATOption {
                repeat: NonZeroUsize::new(5).unwrap(),
                constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
                ..Default::default()
            },
        };

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
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
}
