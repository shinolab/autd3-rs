use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use crate::{
    constraint::EmissionConstraint,
    helper::{generate_result, HoloContextGenerator},
    Amplitude, Complex, LinAlgBackend, Trans,
};

use autd3_core::{acoustics::directivity::Directivity, derive::*, geometry::Point3};
use autd3_derive::Builder;
use derive_more::Debug;
use zerocopy::{FromBytes, IntoBytes};

/// Gerchberg-Saxton algorithm
///
/// See [Marzo, et al., 2019](https://www.pnas.org/doi/full/10.1073/pnas.1813047115) for more details.
#[derive(Gain, Builder, Debug)]
pub struct GS<D: Directivity, B: LinAlgBackend<D>> {
    #[get(ref)]
    /// The focal positions.
    foci: Vec<Point3>,
    #[get(ref)]
    /// The focal amplitudes.
    amps: Vec<Amplitude>,
    #[get]
    #[set]
    /// The number of iterations.
    repeat: NonZeroUsize,
    #[get]
    #[set]
    /// The transducers' emission constraint.
    constraint: EmissionConstraint,
    #[debug("{}", tynm::type_name::<B>())]
    backend: Arc<B>,
    #[debug(ignore)]
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity, B: LinAlgBackend<D>> GS<D, B> {
    /// Creates a new [`GS`].
    pub fn new(backend: Arc<B>, iter: impl IntoIterator<Item = (Point3, Amplitude)>) -> Self {
        let (foci, amps) = iter.into_iter().unzip();
        Self {
            foci,
            amps,
            repeat: NonZeroUsize::new(100).unwrap(),
            backend,
            constraint: EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<D: Directivity, B: LinAlgBackend<D>> Gain for GS<D, B> {
    type G = HoloContextGenerator<Complex>;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
    ) -> Result<Self::G, GainError> {
        let g = self
            .backend
            .generate_propagation_matrix(geometry, &self.foci, filter)?;

        let m = self.foci.len();
        let n = self.backend.cols_c(&g)?;
        let ones = vec![1.; n];

        let b = self.backend.gen_back_prop(n, m, &g)?;

        let mut q = self.backend.from_slice_cv(&ones)?;

        let q0 = self.backend.from_slice_cv(&ones)?;

        let amps = self
            .backend
            .from_slice_cv(<[f32]>::ref_from_bytes(self.amps.as_bytes()).unwrap())?;
        let mut p = self.backend.alloc_zeros_cv(m)?;
        (0..self.repeat.get()).try_for_each(|_| -> Result<(), GainError> {
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
        generate_result(geometry, q, max_coefficient, self.constraint, filter)
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::gain::{Drive, GainContext, GainContextGenerator};

    use crate::tests::create_geometry;

    use super::{super::super::NalgebraBackend, super::super::Pa, *};

    #[test]
    fn test_gs_all() {
        let geometry = create_geometry(1, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = GS::new(
            backend,
            [(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
        )
        .with_repeat(NonZeroUsize::new(5).unwrap());

        assert_eq!(g.repeat().get(), 5);
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
    fn test_gs_filtered() {
        let geometry = create_geometry(2, 1);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = GS::new(
            backend,
            [
                (Point3::new(10., 10., 100.), 5e3 * Pa),
                (Point3::new(-10., 10., 100.), 5e3 * Pa),
            ],
        )
        .with_repeat(NonZeroUsize::new(5).unwrap())
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
