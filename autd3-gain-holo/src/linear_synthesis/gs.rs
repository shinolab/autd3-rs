use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use crate::{
    constraint::EmissionConstraint, helper::generate_result, Amplitude, Complex, LinAlgBackend,
    Trans,
};

use autd3_driver::{
    acoustics::directivity::Directivity, derive::*, firmware::fpga::EmitIntensity,
    geometry::Vector3,
};
use bit_vec::BitVec;
use derive_more::Debug;

#[derive(Gain, Builder, Debug)]
pub struct GS<D: Directivity, B: LinAlgBackend<D>> {
    #[get(ref)]
    foci: Vec<Vector3>,
    #[get(ref)]
    amps: Vec<Amplitude>,
    #[get]
    #[set]
    repeat: NonZeroUsize,
    #[get]
    #[set]
    constraint: EmissionConstraint,
    #[debug("{}", tynm::type_name::<B>())]
    backend: Arc<B>,
    #[debug(ignore)]
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity, B: LinAlgBackend<D>> GS<D, B> {
    pub fn new(backend: Arc<B>, iter: impl IntoIterator<Item = (Vector3, Amplitude)>) -> Self {
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

impl<D: Directivity, B: LinAlgBackend<D>> GS<D, B> {
    fn calc_impl(
        &self,
        geometry: &Geometry,
        filter: Option<HashMap<usize, BitVec<u32>>>,
    ) -> Result<GainCalcFn, AUTDInternalError> {
        let g = self
            .backend
            .generate_propagation_matrix(geometry, &self.foci, &filter)?;

        let m = self.foci.len();
        let n = self.backend.cols_c(&g)?;
        let ones = vec![1.; n];

        let b = self.backend.gen_back_prop(n, m, &g)?;

        let mut q = self.backend.from_slice_cv(&ones)?;

        let q0 = self.backend.from_slice_cv(&ones)?;

        let amps = self.backend.from_slice_cv(unsafe {
            std::slice::from_raw_parts(self.amps.as_ptr() as *const f32, self.amps.len())
        })?;
        let mut p = self.backend.alloc_zeros_cv(m)?;
        (0..self.repeat.get()).try_for_each(|_| -> Result<(), AUTDInternalError> {
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

impl<D: Directivity, B: LinAlgBackend<D>> Gain for GS<D, B> {
    fn calc(&self, geometry: &Geometry) -> Result<GainCalcFn, AUTDInternalError> {
        self.calc_impl(geometry, None)
    }

    fn calc_with_filter(
        &self,
        geometry: &Geometry,
        filter: HashMap<usize, BitVec<u32>>,
    ) -> Result<GainCalcFn, AUTDInternalError> {
        self.calc_impl(geometry, Some(filter))
    }
}

#[cfg(test)]
mod tests {
    use super::{super::super::NalgebraBackend, super::super::Pa, *};
    use autd3_driver::{autd3_device::AUTD3, firmware::fpga::Drive, geometry::IntoDevice};

    #[test]
    fn test_gs_all() {
        let geometry: Geometry =
            Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0, 0)]);
        let backend =
            NalgebraBackend::<autd3_driver::acoustics::directivity::Sphere>::new().unwrap();

        let g = GS::new(
            backend,
            [(Vector3::zeros(), 1. * Pa), (Vector3::zeros(), 1. * Pa)],
        )
        .with_repeat(NonZeroUsize::new(5).unwrap());

        assert_eq!(g.repeat().get(), 5);
        assert_eq!(
            g.constraint(),
            EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX)
        );

        assert_eq!(
            g.with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)))
                .calc(&geometry)
                .map(|mut res| {
                    let f = res(&geometry[0]);
                    geometry[0]
                        .iter()
                        .filter(|tr| f(tr) != Drive::null())
                        .count()
                }),
            Ok(geometry.num_transducers()),
        );
    }

    #[test]
    fn test_gs_filtered() {
        let geometry: Geometry = Geometry::new(vec![
            AUTD3::new(Vector3::zeros()).into_device(0, 0),
            AUTD3::new(Vector3::zeros()).into_device(1, AUTD3::NUM_TRANS_IN_UNIT),
        ]);
        let backend =
            NalgebraBackend::<autd3_driver::acoustics::directivity::Sphere>::new().unwrap();

        let g = GS::new(
            backend,
            [
                (Vector3::new(10., 10., 100.), 5e3 * Pa),
                (Vector3::new(-10., 10., 100.), 5e3 * Pa),
            ],
        )
        .with_repeat(NonZeroUsize::new(5).unwrap())
        .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)));

        let filter = geometry
            .iter()
            .take(1)
            .map(|dev| {
                (
                    dev.idx(),
                    dev.iter().map(|tr| tr.local_idx() < 100).collect(),
                )
            })
            .collect::<HashMap<_, _>>();
        assert_eq!(
            g.calc_with_filter(&geometry, filter.clone())
                .map(|mut res| {
                    let f = res(&geometry[0]);
                    geometry[0]
                        .iter()
                        .filter(|tr| f(tr) != Drive::null())
                        .count()
                }),
            Ok(100),
        );
        assert_eq!(
            g.calc_with_filter(&geometry, filter).map(|mut res| {
                let f = res(&geometry[1]);
                geometry[1]
                    .iter()
                    .filter(|tr| f(tr) != Drive::null())
                    .count()
            }),
            Ok(0),
        );
    }
}
