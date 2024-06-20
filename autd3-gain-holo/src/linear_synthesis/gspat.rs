use std::{collections::HashMap, sync::Arc};

use crate::{
    constraint::EmissionConstraint, helper::generate_result, Amplitude, Complex, LinAlgBackend,
    Trans,
};

use autd3_driver::{acoustics::directivity::Directivity, derive::*, geometry::Vector3};
use bitvec::{order::Lsb0, vec::BitVec};

#[derive(Gain, Builder)]
pub struct GSPAT<D: Directivity + 'static, B: LinAlgBackend<D> + 'static> {
    #[get]
    foci: Vec<Vector3>,
    #[get]
    amps: Vec<Amplitude>,
    #[getset]
    repeat: usize,
    #[getset]
    constraint: EmissionConstraint,
    backend: Arc<B>,
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity + 'static, B: LinAlgBackend<D> + 'static> GSPAT<D, B> {
    pub fn new(backend: Arc<B>, iter: impl IntoIterator<Item = (Vector3, Amplitude)>) -> Self {
        let (foci, amps) = iter.into_iter().unzip();
        Self {
            foci,
            amps,
            repeat: 100,
            backend,
            constraint: EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<D: Directivity, B: LinAlgBackend<D>> GSPAT<D, B> {
    fn calc_impl(
        &self,
        geometry: &Geometry,
        filter: Option<HashMap<usize, BitVec<usize, Lsb0>>>,
    ) -> GainCalcResult {
        let g = self
            .backend
            .generate_propagation_matrix(geometry, &self.foci, &filter)?;

        let m = self.foci.len();
        let n = self.backend.cols_c(&g)?;

        let mut q = self.backend.alloc_zeros_cv(n)?;

        let amps = self.backend.from_slice_cv(unsafe {
            std::slice::from_raw_parts(self.amps.as_ptr() as *const f32, self.amps.len())
        })?;

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
        (0..self.repeat).try_for_each(|_| -> Result<(), AUTDInternalError> {
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

impl<D: Directivity, B: LinAlgBackend<D>> Gain for GSPAT<D, B> {
    fn calc(&self, geometry: &Geometry) -> GainCalcResult {
        self.calc_impl(geometry, None)
    }

    fn calc_with_filter(
        &self,
        geometry: &Geometry,
        filter: HashMap<usize, BitVec<usize, Lsb0>>,
    ) -> GainCalcResult {
        self.calc_impl(geometry, Some(filter))
    }

    #[tracing::instrument(level = "debug", skip(self, _geometry), fields(?self.repeat, ?self.constraint))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
        if tracing::enabled!(tracing::Level::DEBUG) {
            if tracing::enabled!(tracing::Level::TRACE) {
                self.foci
                    .iter()
                    .zip(self.amps.iter())
                    .enumerate()
                    .for_each(|(i, (f, a))| {
                        tracing::debug!("Foci[{}]: {:?}, {}", i, f, a);
                    });
            } else {
                let len = self.foci.len();
                tracing::debug!("Foci[{}]: {:?}, {}", 0, self.foci[0], self.amps[0]);
                if len > 2 {
                    tracing::debug!("︙");
                }
                if len > 1 {
                    tracing::debug!(
                        "Foci[{}]: {:?}, {}",
                        0,
                        self.foci[len - 1],
                        self.amps[len - 1]
                    );
                }
            }
        }
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use super::{super::super::NalgebraBackend, super::super::Pa, *};
    use autd3_driver::{autd3_device::AUTD3, geometry::IntoDevice};

    #[test]
    fn test_gspat_all() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
        let backend = Arc::new(NalgebraBackend::default());

        let g = GSPAT::new(
            backend,
            [(Vector3::zeros(), 1. * Pa), (Vector3::zeros(), 1. * Pa)],
        )
        .with_repeat(50);

        assert_eq!(g.repeat(), 50);
        assert_eq!(
            g.constraint(),
            EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX)
        );

        assert_eq!(
            g.with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)))
                .calc(&geometry)
                .map(|res| {
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
    fn test_gspat_filtered() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
        let backend = Arc::new(NalgebraBackend::default());

        let g = GSPAT::new(
            backend,
            [
                (Vector3::new(10., 10., 100.), 5e3 * Pa),
                (Vector3::new(-10., 10., 100.), 5e3 * Pa),
            ],
        )
        .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)));

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        assert_eq!(
            g.calc_with_filter(&geometry, filter).map(|res| {
                let f = res(&geometry[0]);
                geometry[0]
                    .iter()
                    .filter(|tr| f(tr) != Drive::null())
                    .count()
            }),
            Ok(100),
        )
    }
}
