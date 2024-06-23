use std::{collections::HashMap, sync::Arc};

use autd3_driver::{
    derive::{rad, tracing, GainCalcResult, Itertools, Phase},
    firmware::fpga::Drive,
    geometry::{Geometry, Vector3},
};
use bitvec::{order::Lsb0, vec::BitVec};
use nalgebra::ComplexField;

use crate::{Amplitude, EmissionConstraint};

pub(crate) trait IntoDrive {
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

pub(crate) fn generate_result<'a, T>(
    geometry: &Geometry,
    q: nalgebra::Matrix<
        T,
        nalgebra::Dyn,
        nalgebra::U1,
        nalgebra::VecStorage<T, nalgebra::Dyn, nalgebra::U1>,
    >,
    max_coefficient: f32,
    constraint: EmissionConstraint,
    filter: Option<HashMap<usize, BitVec<usize, Lsb0>>>,
) -> GainCalcResult<'a>
where
    T: IntoDrive + Copy + Send + Sync + 'static,
{
    let x = std::sync::Arc::new(q);
    if let Some(filter) = filter {
        let transducer_map = geometry
            .iter()
            .scan(0usize, |state, dev| {
                Some(Arc::new(
                    filter
                        .get(&dev.idx())
                        .map(|filter| {
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
                        })
                        .unwrap_or(vec![None; dev.num_transducers()]),
                ))
            })
            .collect::<Vec<_>>();
        Ok(Box::new(move |dev| {
            let x = x.clone();
            let map = transducer_map[dev.idx()].clone();
            Box::new(move |tr| {
                if let Some(idx) = map[tr.idx()] {
                    let x = x[idx];
                    let phase = x.into_phase();
                    let intensity = constraint.convert(x.into_intensity(), max_coefficient);
                    Drive::new(phase, intensity)
                } else {
                    Drive::null()
                }
            })
        }))
    } else {
        let num_transducers = geometry
            .iter()
            .scan(0, |state, dev| {
                let r = *state;
                *state += dev.num_transducers();
                Some(r)
            })
            .collect::<Vec<_>>();
        Ok(Box::new(move |dev| {
            let x = x.clone();
            let base_idx = num_transducers[dev.idx()];
            Box::new(move |tr| {
                let x = x[base_idx + tr.idx()];
                let phase = x.into_phase();
                let intensity = constraint.convert(x.into_intensity(), max_coefficient);
                Drive::new(phase, intensity)
            })
        }))
    }
}

pub(crate) fn holo_trace(foci: &[Vector3], amps: &[Amplitude]) {
    match foci.len() {
        0 => {
            tracing::error!("No foci");
            return;
        }
        1 => {
            tracing::debug!(
                "Foci: [({}, {}, {}), {}]",
                foci[0].x,
                foci[0].y,
                foci[0].z,
                amps[0]
            );
        }
        2 => {
            tracing::debug!(
                "Foci: [({}, {}, {}), {}], [({}, {}, {}), {}]",
                foci[0].x,
                foci[0].y,
                foci[0].z,
                amps[0],
                foci[1].x,
                foci[1].y,
                foci[1].z,
                amps[1]
            );
        }
        _ => {
            if tracing::enabled!(tracing::Level::TRACE) {
                tracing::debug!(
                    "Foci: {}",
                    foci.iter()
                        .zip(amps.iter())
                        .format_with(", ", |elt, f| f(&format_args!(
                            "[({}, {}, {}), {}]",
                            elt.0.x, elt.0.y, elt.0.z, elt.1
                        )))
                );
            } else {
                tracing::debug!(
                    "Foci: [({}, {}, {}), {}], ..., [({}, {}, {}), {}]",
                    foci[0].x,
                    foci[0].y,
                    foci[0].z,
                    amps[0],
                    foci[foci.len() - 1].x,
                    foci[foci.len() - 1].y,
                    foci[foci.len() - 1].z,
                    amps[foci.len() - 1]
                );
            }
        }
    }
}
