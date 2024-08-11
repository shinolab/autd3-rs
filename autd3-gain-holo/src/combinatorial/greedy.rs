use std::{collections::HashMap, num::NonZeroU8};

use crate::{constraint::EmissionConstraint, helper::holo_trace, Amplitude, Complex};

use autd3_driver::{
    acoustics::{directivity::Directivity, propagate},
    defined::PI,
    derive::*,
    geometry::Vector3,
};

use bit_vec::BitVec;
use nalgebra::ComplexField;
use rand::seq::SliceRandom;

#[derive(Gain, Builder)]
pub struct Greedy<D: Directivity> {
    #[get(ref)]
    foci: Vec<Vector3>,
    #[get(ref)]
    amps: Vec<Amplitude>,
    #[get]
    #[set]
    phase_div: NonZeroU8,
    #[get]
    #[set]
    constraint: EmissionConstraint,
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity> Greedy<D> {
    pub fn new(iter: impl IntoIterator<Item = (Vector3, Amplitude)>) -> Self {
        let (foci, amps) = iter.into_iter().unzip();
        Self {
            foci,
            amps,
            phase_div: unsafe { NonZeroU8::new_unchecked(16) },
            constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
            _phantom: std::marker::PhantomData,
        }
    }

    fn transfer_foci(
        trans: &Transducer,
        wavenumber: f32,
        dir: &Vector3,
        foci: &[Vector3],
        res: &mut [Complex],
    ) {
        res.iter_mut().zip(foci.iter()).for_each(|(r, f)| {
            *r = propagate::<D>(trans, wavenumber, dir, f);
        });
    }
}

impl<D: Directivity> Greedy<D> {
    fn calc_impl(
        &self,
        geometry: &Geometry,
        filter: Option<HashMap<usize, BitVec<u32>>>,
    ) -> GainCalcResult {
        let phase_candidates = (0..self.phase_div.get())
            .map(|i| Complex::new(0., 2.0 * PI * i as f32 / self.phase_div.get() as f32).exp())
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
                geometry[dev_idx].axial_direction(),
                &self.foci,
                &mut tmp,
            );
            let (min_idx, _) = phase_candidates.iter().enumerate().fold(
                (0usize, f32::INFINITY),
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

impl<D: Directivity> Gain for Greedy<D> {
    fn calc(&self, geometry: &Geometry) -> GainCalcResult {
        self.calc_impl(geometry, None)
    }

    fn calc_with_filter(
        &self,
        geometry: &Geometry,
        filter: HashMap<usize, BitVec<u32>>,
    ) -> GainCalcResult {
        self.calc_impl(geometry, Some(filter))
    }

    #[tracing::instrument(level = "debug", skip(self, _geometry), fields(?self.phase_div, ?self.constraint))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
        holo_trace(&self.foci, &self.amps);
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use super::{super::super::Pa, *};
    use autd3_driver::{acoustics::directivity::Sphere, autd3_device::AUTD3, geometry::IntoDevice};

    #[test]
    fn test_greedy_all() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

        let g = Greedy::<Sphere>::new([(Vector3::zeros(), 1. * Pa), (Vector3::zeros(), 1. * Pa)])
            .with_phase_div(NonZeroU8::MIN);

        assert_eq!(g.phase_div(), NonZeroU8::MIN);
        assert_eq!(
            g.constraint(),
            EmissionConstraint::Uniform(EmitIntensity::MAX)
        );

        assert_eq!(
            g.calc(&geometry).map(|res| {
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
    fn test_greedy_filtered() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

        let g = Greedy::<Sphere>::new([
            (Vector3::new(10., 10., 100.), 5e3 * Pa),
            (Vector3::new(-10., 10., 100.), 5e3 * Pa),
        ])
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
