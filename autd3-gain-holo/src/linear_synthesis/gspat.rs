use std::{num::NonZeroUsize, sync::Arc};

use crate::{
    Amplitude, Complex, LinAlgBackend, Trans,
    constraint::EmissionConstraint,
    helper::{HoloCalculatorGenerator, generate_result},
};

use autd3_core::{
    acoustics::directivity::{Directivity, Sphere},
    derive::*,
    geometry::Point3,
};
use derive_more::Debug;
use zerocopy::{FromBytes, IntoBytes};

/// The option of [`GSPAT`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GSPATOption {
    /// The number of iterations.
    pub repeat: NonZeroUsize,
    /// The transducers' emission constraint.
    pub constraint: EmissionConstraint,
}

impl Default for GSPATOption {
    fn default() -> Self {
        Self {
            repeat: NonZeroUsize::new(100).unwrap(),
            constraint: EmissionConstraint::Clamp(Intensity::MIN, Intensity::MAX),
        }
    }
}

/// Gerchberg-Saxon for Phased Arrays of Transducers
///
/// See [Plasencia, et al., 2020](https://dl.acm.org/doi/10.1145/3386569.3392492) for more details.
#[derive(Gain, Debug)]
pub struct GSPAT<D: Directivity, B: LinAlgBackend> {
    /// The focal positions and amplitudes.
    pub foci: Vec<(Point3, Amplitude)>,
    /// The option of the Gain.
    pub option: GSPATOption,
    /// The backend of linear algebra calculation.
    #[debug("{}", tynm::type_name::<B>())]
    pub backend: Arc<B>,
    /// The directivity of the transducers.
    pub directivity: std::marker::PhantomData<D>,
}

impl<B: LinAlgBackend> GSPAT<Sphere, B> {
    /// Create a new [`GSPAT`].
    #[must_use]
    pub fn new(
        foci: impl IntoIterator<Item = (Point3, Amplitude)>,
        option: GSPATOption,
        backend: Arc<B>,
    ) -> Self {
        Self::with_directivity(foci, option, backend)
    }
}

impl<D: Directivity, B: LinAlgBackend> GSPAT<D, B> {
    /// Create a new [`GSPAT`] with directivity.
    #[must_use]
    pub fn with_directivity(
        foci: impl IntoIterator<Item = (Point3, Amplitude)>,
        option: GSPATOption,
        backend: Arc<B>,
    ) -> Self {
        Self {
            foci: foci.into_iter().collect(),
            option,
            backend,
            directivity: std::marker::PhantomData,
        }
    }
}

impl<D: Directivity, B: LinAlgBackend> Gain<'_> for GSPAT<D, B> {
    type G = HoloCalculatorGenerator<Complex>;

    fn init(
        self,
        geometry: &Geometry,
        env: &Environment,
        filter: &TransducerFilter,
    ) -> Result<Self::G, GainError> {
        let (foci, amps): (Vec<_>, Vec<_>) = self.foci.into_iter().unzip();

        let g = self
            .backend
            .generate_propagation_matrix::<D>(geometry, env, &foci, filter)?;

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
    use autd3_core::gain::{Drive, GainCalculator, GainCalculatorGenerator};

    use crate::tests::create_geometry;

    use super::{super::super::NalgebraBackend, super::super::Pa, *};

    #[test]
    fn test_gspat_all() {
        let geometry = create_geometry(1, 1);
        let backend = std::sync::Arc::new(NalgebraBackend);

        let g = GSPAT::new(
            vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            GSPATOption {
                repeat: NonZeroUsize::new(5).unwrap(),
                constraint: EmissionConstraint::Uniform(Intensity::MAX),
            },
            backend,
        );

        assert_eq!(
            g.init(
                &geometry,
                &Environment::new(),
                &TransducerFilter::all_enabled(),
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
    fn test_gspat_filtered() {
        let geometry = create_geometry(2, 1);
        let backend = std::sync::Arc::new(NalgebraBackend);

        let g = GSPAT {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            backend,
            option: GSPATOption {
                repeat: NonZeroUsize::new(5).unwrap(),
                constraint: EmissionConstraint::Uniform(Intensity::MAX),
            },
            directivity: std::marker::PhantomData::<Sphere>,
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
