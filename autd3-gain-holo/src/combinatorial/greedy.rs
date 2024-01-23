/*
 * File: greedy.rs
 * Project: combinational
 * Created Date: 03/06/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Shun Suzuki. All rights reserved.
 *
 */

use std::collections::HashMap;

use crate::{constraint::EmissionConstraint, impl_holo, Amplitude, Complex};

use autd3_driver::{
    acoustics::{directivity::Sphere, propagate},
    common::{EmitIntensity, Phase},
    defined::PI,
    derive::*,
    geometry::{Geometry, Vector3},
};

use nalgebra::ComplexField;
use rand::seq::SliceRandom;

/// Gain to produce multiple foci with greedy algorithm
///
/// Reference
/// * Suzuki, Shun, et al. "Radiation pressure field reconstruction for ultrasound midair haptics by Greedy algorithm with brute-force search." IEEE Transactions on Haptics 14.4 (2021): 914-921.
#[derive(Gain)]
pub struct Greedy {
    foci: Vec<Vector3>,
    amps: Vec<Amplitude>,
    phase_div: u8,
    constraint: EmissionConstraint,
}

impl_holo!(Greedy);

impl Greedy {
    pub const fn new() -> Self {
        Self {
            foci: vec![],
            amps: vec![],
            phase_div: 16,
            constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
        }
    }

    pub fn with_phase_div(self, phase_div: u8) -> Self {
        Self { phase_div, ..self }
    }

    fn transfer_foci(
        trans: &Transducer,
        sound_speed: float,
        attenuation: float,
        foci: &[Vector3],
        res: &mut [Complex],
    ) {
        res.iter_mut().zip(foci.iter()).for_each(|(r, f)| {
            *r = propagate::<Sphere>(trans, attenuation, sound_speed, f);
        });
    }

    pub const fn phase_div(&self) -> u8 {
        self.phase_div
    }
}

impl Gain for Greedy {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        let phase_candidates = (0..self.phase_div)
            .map(|i| Complex::new(0., 2.0 * PI * i as float / self.phase_div as float).exp())
            .collect::<Vec<_>>();

        let indices = {
            let mut indices: Vec<_> = match filter {
                GainFilter::All => geometry
                    .devices()
                    .flat_map(|dev| dev.iter().map(|tr| (dev.idx(), tr.idx())))
                    .collect(),
                GainFilter::Filter(filter) => geometry
                    .devices()
                    .filter_map(|dev| {
                        filter.get(&dev.idx()).map(|filter| {
                            dev.iter().filter_map(|tr| {
                                if filter[tr.idx()] {
                                    Some((dev.idx(), tr.idx()))
                                } else {
                                    None
                                }
                            })
                        })
                    })
                    .flatten()
                    .collect(),
            };
            indices.shuffle(&mut rand::thread_rng());
            indices
        };

        let mut res: HashMap<usize, Vec<Drive>> = geometry
            .devices()
            .map(|dev| (dev.idx(), vec![Drive::null(); dev.num_transducers()]))
            .collect();
        let mut cache = vec![Complex::new(0., 0.); self.foci.len()];
        indices.iter().for_each(|&(dev_idx, idx)| {
            let mut tmp = vec![Complex::new(0., 0.); self.foci.len()];
            Self::transfer_foci(
                &geometry[dev_idx][idx],
                geometry[dev_idx].sound_speed,
                geometry[dev_idx].attenuation,
                &self.foci,
                &mut tmp,
            );
            let (min_idx, _) = phase_candidates.iter().enumerate().fold(
                (0usize, float::INFINITY),
                |acc, (idx, &phase)| {
                    let v = cache.iter().enumerate().fold(0., |acc, (j, c)| {
                        acc + (self.amps[j].value - (tmp[j] * phase + c).abs()).abs()
                    });
                    if v < acc.1 {
                        (idx, v)
                    } else {
                        acc
                    }
                },
            );
            let phase = phase_candidates[min_idx];
            cache.iter_mut().zip(tmp.iter()).for_each(|(c, a)| {
                *c += a * phase;
            });
            let r = &mut res.get_mut(&dev_idx).unwrap()[idx];
            r.intensity = self.constraint.convert(1.0, 1.0);
            r.phase = Phase::from_rad(phase.argument() + PI);
        });
        Ok(res)
    }
}

impl Default for Greedy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{super::super::Pascal, *};
    use autd3_driver::{autd3_device::AUTD3, geometry::IntoDevice};

    #[test]
    fn test_greedy_all() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

        let g = Greedy::new()
            .with_phase_div(32)
            .add_focus(Vector3::zeros(), 1. * Pascal)
            .add_foci_from_iter([(Vector3::zeros(), 1. * Pascal)]);

        assert_eq!(g.phase_div(), 32);
        assert_eq!(
            g.constraint(),
            EmissionConstraint::Uniform(EmitIntensity::MAX)
        );
        assert!(g
            .foci()
            .all(|(&p, &a)| p == Vector3::zeros() && a == 1. * Pascal));

        let _ = g.calc(&geometry, GainFilter::All);
        let _ = g.operation();
    }

    #[test]
    fn test_greedy_filtered() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

        let g = Greedy::default()
            .add_focus(Vector3::new(10., 10., 100.), 5e3 * Pascal)
            .add_foci_from_iter([(Vector3::new(-10., 10., 100.), 5e3 * Pascal)])
            .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)));

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        assert_eq!(
            g.calc(&geometry, GainFilter::Filter(&filter))
                .map(|res| res[&0].iter().filter(|&&d| d != Drive::null()).count()),
            Ok(100),
        )
    }
}
