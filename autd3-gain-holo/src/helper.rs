use std::{collections::HashMap, sync::Arc};

use autd3_driver::{
    defined::rad,
    derive::{GainContext, GainContextGenerator, Transducer},
    error::AUTDInternalError,
    firmware::fpga::{Drive, Phase},
    geometry::Geometry,
};
use bit_vec::BitVec;
use nalgebra::ComplexField;
use rayon::iter::Either;

use crate::EmissionConstraint;

pub trait IntoDrive {
    fn into_phase(self) -> Phase;
    fn into_intensity(self) -> f32;
}

impl IntoDrive for f32 {
    fn into_intensity(self) -> f32 {
        1.
    }

    fn into_phase(self) -> Phase {
        Phase::from(self * rad)
    }
}

impl IntoDrive for crate::Complex {
    fn into_intensity(self) -> f32 {
        self.abs()
    }

    fn into_phase(self) -> Phase {
        Phase::from(self)
    }
}

#[allow(clippy::type_complexity)]
pub struct HoloContext<T: IntoDrive + Copy + Send + Sync + 'static> {
    q: Arc<
        nalgebra::Matrix<
            T,
            nalgebra::Dyn,
            nalgebra::U1,
            nalgebra::VecStorage<T, nalgebra::Dyn, nalgebra::U1>,
        >,
    >,
    map: Either<Option<Vec<Option<usize>>>, usize>,
    max_coefficient: f32,
    constraint: EmissionConstraint,
}

impl<T: IntoDrive + Copy + Send + Sync + 'static> GainContext for HoloContext<T> {
    fn calc(&self, tr: &Transducer) -> Drive {
        match &self.map {
            Either::Left(map) => map
                .as_ref()
                .and_then(|map| {
                    map[tr.idx()].map(|idx| {
                        let x = self.q[idx];
                        let phase = x.into_phase();
                        let intensity = self
                            .constraint
                            .convert(x.into_intensity(), self.max_coefficient);
                        Drive::new(phase, intensity)
                    })
                })
                .unwrap_or(Drive::NULL),
            Either::Right(base_idx) => {
                let x = self.q[base_idx + tr.idx()];
                let phase = x.into_phase();
                let intensity = self
                    .constraint
                    .convert(x.into_intensity(), self.max_coefficient);
                Drive::new(phase, intensity)
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct HoloContextGenerator<T: IntoDrive + Copy + Send + Sync + 'static> {
    q: Arc<
        nalgebra::Matrix<
            T,
            nalgebra::Dyn,
            nalgebra::U1,
            nalgebra::VecStorage<T, nalgebra::Dyn, nalgebra::U1>,
        >,
    >,
    map: Either<HashMap<usize, Option<Vec<Option<usize>>>>, HashMap<usize, usize>>,
    max_coefficient: f32,
    constraint: EmissionConstraint,
}

impl<T: IntoDrive + Copy + Send + Sync + 'static> GainContextGenerator for HoloContextGenerator<T> {
    type Context = HoloContext<T>;

    fn generate(&mut self, device: &autd3_driver::geometry::Device) -> Self::Context {
        match &mut self.map {
            Either::Left(map) => HoloContext {
                q: self.q.clone(),
                map: Either::Left(map.remove(&device.idx()).unwrap()),
                max_coefficient: self.max_coefficient,
                constraint: self.constraint,
            },
            Either::Right(map) => HoloContext {
                q: self.q.clone(),
                map: Either::Right(map[&device.idx()]),
                max_coefficient: self.max_coefficient,
                constraint: self.constraint,
            },
        }
    }
}

pub(crate) fn generate_result<T>(
    geometry: &Geometry,
    q: nalgebra::Matrix<
        T,
        nalgebra::Dyn,
        nalgebra::U1,
        nalgebra::VecStorage<T, nalgebra::Dyn, nalgebra::U1>,
    >,
    max_coefficient: f32,
    constraint: EmissionConstraint,
    filter: Option<HashMap<usize, BitVec<u32>>>,
) -> Result<HoloContextGenerator<T>, AUTDInternalError>
where
    T: IntoDrive + Copy + Send + Sync + 'static,
{
    let q = std::sync::Arc::new(q);
    if let Some(filter) = filter {
        Ok(HoloContextGenerator {
            q,
            map: Either::Left(
                geometry
                    .devices()
                    .scan(0usize, |state, dev| {
                        Some((
                            dev.idx(),
                            filter.get(&dev.idx()).map(|filter| {
                                dev.iter()
                                    .map(|tr| {
                                        if filter[tr.idx()] {
                                            let r = *state;
                                            *state += 1;
                                            Some(r)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            }),
                        ))
                    })
                    .collect(),
            ),
            max_coefficient,
            constraint,
        })
    } else {
        Ok(HoloContextGenerator {
            q,
            map: Either::Right(
                geometry
                    .devices()
                    .scan(0, |state, dev| {
                        let r = *state;
                        *state += dev.num_transducers();
                        Some((dev.idx(), r))
                    })
                    .collect(),
            ),
            max_coefficient,
            constraint,
        })
    }
}
