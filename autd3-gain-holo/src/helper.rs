use std::{collections::HashMap, sync::Arc};

use autd3_driver::{
    derive::{rad, GainCalcResult, Phase},
    firmware::fpga::Drive,
    geometry::Geometry,
};
use bitvec::{order::Lsb0, vec::BitVec};
use nalgebra::ComplexField;

use crate::EmissionConstraint;

pub(crate) trait IntoDrive {
    fn into_phase(self) -> Phase;
    fn into_intensity(self) -> f64;
}

impl IntoDrive for f64 {
    fn into_intensity(self) -> f64 {
        1.
    }

    fn into_phase(self) -> Phase {
        Phase::from(self * rad)
    }
}

impl IntoDrive for crate::Complex {
    fn into_intensity(self) -> f64 {
        self.abs()
    }

    fn into_phase(self) -> Phase {
        Phase::from(self)
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
    max_coefficient: f64,
    constraint: EmissionConstraint,
    filter: Option<HashMap<usize, BitVec<usize, Lsb0>>>,
) -> GainCalcResult
where
    T: IntoDrive + Copy + Send + Sync + 'static,
{
    let x = std::sync::Arc::new(std::sync::RwLock::new(q));
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
                    let x = x.read().unwrap()[idx];
                    let phase = x.into_phase();
                    let intensity = constraint.convert(x.into_intensity(), max_coefficient);
                    Drive::new(phase, intensity)
                } else {
                    return Drive::null();
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
                let x = x.read().unwrap()[base_idx + tr.idx()];
                let phase = x.into_phase();
                let intensity = constraint.convert(x.into_intensity(), max_coefficient);
                Drive::new(phase, intensity)
            })
        }))
    }
}
