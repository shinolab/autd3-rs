use std::{collections::HashMap, sync::Arc};

use autd3_core::{
    common::rad,
    gain::{Drive, GainCalculator, GainCalculatorGenerator, GainError, Phase, TransducerFilter},
    geometry::{Device, Geometry, Transducer},
};
use nalgebra::ComplexField;
use rayon::iter::Either;

use crate::EmissionConstraint;

pub trait IntoDrive {
    #[must_use]
    fn into_phase(self) -> Phase;
    #[must_use]
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
pub struct HoloCalculator<T: IntoDrive + Copy + Send + Sync + 'static> {
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

impl<T: IntoDrive + Copy + Send + Sync + 'static> GainCalculator for HoloCalculator<T> {
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
                        Drive { phase, intensity }
                    })
                })
                .unwrap_or(Drive::NULL),
            Either::Right(base_idx) => {
                let x = self.q[base_idx + tr.idx()];
                let phase = x.into_phase();
                let intensity = self
                    .constraint
                    .convert(x.into_intensity(), self.max_coefficient);
                Drive { phase, intensity }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct HoloCalculatorGenerator<T: IntoDrive + Copy + Send + Sync + 'static> {
    q: Arc<
        nalgebra::Matrix<
            T,
            nalgebra::Dyn,
            nalgebra::U1,
            nalgebra::VecStorage<T, nalgebra::Dyn, nalgebra::U1>,
        >,
    >,
    map: Either<HashMap<usize, Vec<Option<usize>>>, HashMap<usize, usize>>,
    max_coefficient: f32,
    constraint: EmissionConstraint,
}

impl<T: IntoDrive + Copy + Send + Sync + 'static> GainCalculatorGenerator
    for HoloCalculatorGenerator<T>
{
    type Calculator = HoloCalculator<T>;

    fn generate(&mut self, device: &Device) -> Self::Calculator {
        match &mut self.map {
            Either::Left(map) => HoloCalculator {
                q: self.q.clone(),
                map: Either::Left(map.remove(&device.idx())),
                max_coefficient: self.max_coefficient,
                constraint: self.constraint,
            },
            Either::Right(map) => HoloCalculator {
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
    filter: &TransducerFilter,
) -> Result<HoloCalculatorGenerator<T>, GainError>
where
    T: IntoDrive + Copy + Send + Sync + 'static,
{
    let q = std::sync::Arc::new(q);
    if filter.is_all_enabled() {
        Ok(HoloCalculatorGenerator {
            q,
            map: Either::Right(
                geometry
                    .iter()
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
    } else {
        Ok(HoloCalculatorGenerator {
            q,
            map: Either::Left(
                geometry
                    .iter()
                    .filter(|dev| filter.is_enabled_device(dev))
                    .scan(0usize, |state, dev| {
                        Some((dev.idx(), {
                            dev.iter()
                                .map(|tr| {
                                    if filter.is_enabled(tr) {
                                        let r = *state;
                                        *state += 1;
                                        Some(r)
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                        }))
                    })
                    .collect(),
            ),
            max_coefficient,
            constraint,
        })
    }
}
