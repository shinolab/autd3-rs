use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use crate::{
    constraint::EmissionConstraint,
    helper::{generate_result, HoloContextGenerator},
    Amplitude, Complex, LinAlgBackend, Trans,
};

use autd3_driver::{
    acoustics::directivity::Directivity, derive::*, firmware::fpga::EmitIntensity,
    geometry::Vector3,
};
use bit_vec::BitVec;
use derive_more::Debug;
use zerocopy::{FromBytes, IntoBytes};

#[derive(Gain, Builder, Debug)]
pub struct GSPAT<D: Directivity, B: LinAlgBackend<D>> {
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

impl<D: Directivity, B: LinAlgBackend<D>> GSPAT<D, B> {
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

impl<D: Directivity, B: LinAlgBackend<D>> Gain for GSPAT<D, B> {
    type G = HoloContextGenerator<Complex>;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDInternalError> {
        let g = self
            .backend
            .generate_propagation_matrix(geometry, &self.foci, filter)?;

        let m = self.foci.len();
        let n = self.backend.cols_c(&g)?;

        let mut q = self.backend.alloc_zeros_cv(n)?;

        let amps = self
            .backend
            .from_slice_cv(<[f32]>::ref_from_bytes(self.amps.as_bytes()).unwrap())?;

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
        (0..self.repeat.get()).try_for_each(|_| -> Result<(), AUTDInternalError> {
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
        generate_result(geometry, q, max_coefficient, self.constraint, filter)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::super::NalgebraBackend, super::super::Pa, *};
    use autd3_driver::{autd3_device::AUTD3, firmware::fpga::Drive, geometry::IntoDevice};

    #[test]
    fn test_gspat_all() {
        let geometry: Geometry =
            Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)], 4);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = GSPAT::new(
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
    fn test_gspat_filtered() {
        let geometry: Geometry =
            Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)], 4);
        let backend = std::sync::Arc::new(NalgebraBackend::default());

        let g = GSPAT::new(
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
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
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
}
