use std::collections::HashMap;

use crate::{constraint::EmissionConstraint, impl_holo, Amplitude, Complex};

use autd3_driver::{
    acoustics::{
        directivity::{Directivity, Sphere},
        propagate,
    },
    defined::PI,
    derive::*,
    geometry::Vector3,
};

use bitvec::{order::Lsb0, vec::BitVec};
use nalgebra::ComplexField;
use rand::seq::SliceRandom;

#[derive(Gain, Builder)]
#[no_const]
pub struct Greedy<D: Directivity + 'static> {
    foci: Vec<Vector3>,
    amps: Vec<Amplitude>,
    #[getset]
    phase_div: u8,
    constraint: EmissionConstraint,
    _phantom: std::marker::PhantomData<D>,
}

impl_holo!(D, Greedy<D>);

impl<D: Directivity + 'static> Greedy<D> {
    pub const fn new() -> Self {
        Self {
            foci: vec![],
            amps: vec![],
            phase_div: 16,
            constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
            _phantom: std::marker::PhantomData,
        }
    }

    fn transfer_foci(
        trans: &Transducer,
        wavenumber: f64,
        attenuation: f64,
        dir: &Vector3,
        foci: &[Vector3],
        res: &mut [Complex],
    ) {
        res.iter_mut().zip(foci.iter()).for_each(|(r, f)| {
            *r = propagate::<D>(trans, attenuation, wavenumber, dir, f);
        });
    }
}

impl<D: Directivity + 'static> Greedy<D> {
    fn calc_impl(
        &self,
        geometry: &Geometry,
        filter: Option<HashMap<usize, BitVec<usize, Lsb0>>>,
    ) -> GainCalcResult {
        let phase_candidates = (0..self.phase_div)
            .map(|i| Complex::new(0., 2.0 * PI * i as f64 / self.phase_div as f64).exp())
            .collect::<Vec<_>>();

        let indices = {
            let mut indices: Vec<_> = if let Some(filter) = &filter {
                geometry
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
                    .collect()
            } else {
                geometry
                    .devices()
                    .flat_map(|dev| dev.iter().map(|tr| (dev.idx(), tr.idx())))
                    .collect()
            };
            indices.shuffle(&mut rand::thread_rng());
            indices
        };

        let res: Vec<_> = geometry
            .devices()
            .map(|dev| {
                std::sync::Arc::new(std::sync::RwLock::new(vec![
                    Drive::null();
                    dev.num_transducers()
                ]))
            })
            .collect();
        let mut cache = vec![Complex::new(0., 0.); self.foci.len()];
        indices.iter().for_each(|&(dev_idx, idx)| {
            let mut tmp = vec![Complex::new(0., 0.); self.foci.len()];
            Self::transfer_foci(
                &geometry[dev_idx][idx],
                geometry[dev_idx].wavenumber(),
                geometry[dev_idx].attenuation,
                geometry[dev_idx].axial_direction(),
                &self.foci,
                &mut tmp,
            );
            let (min_idx, _) = phase_candidates.iter().enumerate().fold(
                (0usize, f64::INFINITY),
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
            res[dev_idx].write().unwrap()[idx] =
                Drive::new(Phase::from(phase), self.constraint.convert(1.0, 1.0));
        });

        Ok(Box::new(move |dev| {
            let d = res[dev.idx()].clone();
            Box::new(move |tr| d.read().unwrap()[tr.idx()])
        }))
    }
}

impl<D: Directivity + 'static> Gain for Greedy<D> {
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
}

impl Default for Greedy<Sphere> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{super::super::Pa, *};
    use autd3_driver::{autd3_device::AUTD3, defined::FREQ_40K, geometry::IntoDevice};

    #[test]
    fn test_greedy_all() {
        let geometry: Geometry =
            Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)], FREQ_40K);

        let g = Greedy::default()
            .with_phase_div(32)
            .add_focus(Vector3::zeros(), 1. * Pa)
            .add_foci_from_iter([(Vector3::zeros(), 1. * Pa)]);

        assert_eq!(g.phase_div(), 32);
        assert_eq!(
            g.constraint(),
            EmissionConstraint::Uniform(EmitIntensity::MAX)
        );
        assert!(g
            .foci()
            .all(|(&p, &a)| p == Vector3::zeros() && a == 1. * Pa));

        assert_eq!(
            g.calc(&geometry)
                .map(|res| res[&0].iter().filter(|&&d| d != Drive::null()).count()),
            Ok(geometry.num_transducers()),
        );
    }

    #[test]
    fn test_greedy_filtered() {
        let geometry: Geometry =
            Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)], FREQ_40K);

        let g = Greedy::default()
            .add_focus(Vector3::new(10., 10., 100.), 5e3 * Pa)
            .add_foci_from_iter([(Vector3::new(-10., 10., 100.), 5e3 * Pa)])
            .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)));

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        assert_eq!(
            g.calc(&geometry, Option<HashMap<usize, BitVec<usize, Lsb0>>>,::Filter(&filter))
                .map(|res| res[&0].iter().filter(|&&d| d != Drive::null()).count()),
            Ok(100),
        )
    }
}
