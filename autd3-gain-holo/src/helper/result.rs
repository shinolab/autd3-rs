use std::{collections::HashMap, sync::Arc};

use autd3_core::{
    firmware::{Drive, Phase},
    gain::{GainCalculator, GainCalculatorGenerator, GainError, TransducerMask},
    geometry::{Device, Geometry, Transducer},
};
use nalgebra::ComplexField;
use rayon::iter::Either;

use crate::{EmissionConstraint, VectorXc};

#[allow(clippy::type_complexity)]
pub struct HoloCalculator {
    q: Arc<VectorXc>,
    map: Either<Option<Vec<Option<usize>>>, usize>,
    max_coefficient: f32,
    constraint: EmissionConstraint,
}

impl GainCalculator<'_> for HoloCalculator {
    fn calc(&self, tr: &Transducer) -> Drive {
        match &self.map {
            Either::Left(map) => map
                .as_ref()
                .and_then(|map| {
                    map[tr.idx()].map(|idx| {
                        let x = self.q[idx];
                        let phase = Phase::from(x);
                        let intensity = self.constraint.convert(x.abs(), self.max_coefficient);
                        Drive { phase, intensity }
                    })
                })
                .unwrap_or(Drive::NULL),
            Either::Right(base_idx) => {
                let x = self.q[base_idx + tr.idx()];
                let phase = Phase::from(x);
                let intensity = self.constraint.convert(x.abs(), self.max_coefficient);
                Drive { phase, intensity }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct HoloCalculatorGenerator {
    q: Arc<VectorXc>,
    map: Either<HashMap<usize, Vec<Option<usize>>>, HashMap<usize, usize>>,
    max_coefficient: f32,
    constraint: EmissionConstraint,
}

impl GainCalculatorGenerator<'_> for HoloCalculatorGenerator {
    type Calculator = HoloCalculator;

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

pub(crate) fn generate_result(
    geometry: &Geometry,
    q: VectorXc,
    max_coefficient: f32,
    constraint: EmissionConstraint,
    filter: &TransducerMask,
) -> Result<HoloCalculatorGenerator, GainError> {
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
                    .filter(|dev| filter.has_enabled(dev))
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
