use std::{collections::HashMap, sync::Arc};

use crate::{
    constraint::EmissionConstraint,
    helper::{generate_result, HoloContextGenerator},
    Amplitude, Complex, LinAlgBackend, Trans,
};

use autd3_driver::{
    acoustics::directivity::Directivity, derive::*, firmware::fpga::EmitIntensity, geometry::Point3,
};
use bit_vec::BitVec;
use derive_more::Debug;
use zerocopy::{FromBytes, IntoBytes};

/// Naive linear synthesis of simple focal solutions
#[derive(Gain, Builder, Debug)]
pub struct Naive<D: Directivity, B: LinAlgBackend<D>> {
    #[get(ref)]
    /// The focal positions.
    foci: Vec<Point3>,
    #[get(ref)]
    /// The focal amplitudes.
    amps: Vec<Amplitude>,
    #[get]
    #[set]
    /// The transducers' emission constraint.
    constraint: EmissionConstraint,
    #[debug("{}", tynm::type_name::<B>())]
    backend: Arc<B>,
    #[debug(ignore)]
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity, B: LinAlgBackend<D>> Naive<D, B> {
    /// Creates a new [`Naive`].
    pub fn new(backend: Arc<B>, iter: impl IntoIterator<Item = (Point3, Amplitude)>) -> Self {
        let (foci, amps) = iter.into_iter().unzip();
        Self {
            foci,
            amps,
            backend,
            constraint: EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<D: Directivity, B: LinAlgBackend<D>> Gain for Naive<D, B> {
    type G = HoloContextGenerator<Complex>;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDDriverError> {
        let g = self
            .backend
            .generate_propagation_matrix(geometry, &self.foci, filter)?;

        let m = self.foci.len();
        let n = self.backend.cols_c(&g)?;

        let b = self.backend.gen_back_prop(n, m, &g)?;

        let p = self
            .backend
            .from_slice_cv(<[f32]>::ref_from_bytes(self.amps.as_bytes()).unwrap())?;
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
        generate_result(geometry, q, max_coefficient, self.constraint, filter)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::super::NalgebraBackend, super::super::Pa, *};
    use autd3_driver::{autd3_device::AUTD3, firmware::fpga::Drive, geometry::IntoDevice};

    #[test]
    fn test_naive_all() {
        let geometry: Geometry =
            Geometry::new(vec![AUTD3::new(Point3::origin()).into_device(0)], 4);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = Naive::new(
            backend,
            [(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
        );

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
    fn test_naive_all_disabled() -> anyhow::Result<()> {
        let mut geometry = Geometry::new(
            vec![
                AUTD3::new(Point3::origin()).into_device(0),
                AUTD3::new(Point3::origin()).into_device(1),
            ],
            4,
        );
        geometry[0].enable = false;
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = Naive::new(
            backend,
            [(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
        );

        let mut g = g
            .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)))
            .init(&geometry, None)?;
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
        let geometry: Geometry =
            Geometry::new(vec![AUTD3::new(Point3::origin()).into_device(0)], 4);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = Naive::new(
            backend,
            [
                (Point3::new(10., 10., 100.), 5e3 * Pa),
                (Point3::new(-10., 10., 100.), 5e3 * Pa),
            ],
        )
        .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)));

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect();
        assert_eq!(
            g.init(&geometry, Some(&filter)).map(|mut res| {
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
        let mut geometry = Geometry::new(
            vec![
                AUTD3::new(Point3::origin()).into_device(0),
                AUTD3::new(Point3::origin()).into_device(1),
            ],
            4,
        );
        geometry[0].enable = false;
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = Naive::new(
            backend,
            [(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
        );

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect();
        let mut g = g
            .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)))
            .init(&geometry, Some(&filter))?;
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
