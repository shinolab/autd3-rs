use std::{collections::HashMap, num::NonZeroU8};

use crate::{constraint::EmissionConstraint, Amplitude, Complex};

use autd3_core::{
    acoustics::{directivity::Directivity, propagate},
    defined::PI,
    derive::*,
    geometry::{Point3, UnitVector3},
};

use derive_more::Debug;
use nalgebra::ComplexField;
use rand::seq::SliceRandom;

/// The option of [`Greedy`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GreedyOption<D: Directivity> {
    /// The number of phase divisions.
    pub phase_div: NonZeroU8,
    /// The transducers' emission constraint.
    pub constraint: EmissionConstraint,
    #[doc(hidden)]
    pub __phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity> Default for GreedyOption<D> {
    fn default() -> Self {
        Self {
            phase_div: NonZeroU8::new(16).unwrap(),
            constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
            __phantom: std::marker::PhantomData,
        }
    }
}

/// Greedy algorithm and Brute-force search
///
/// [`Greedy`] is based on the method of optimizing by brute-force search and greedy algorithm by discretizing the phase.
/// See [Suzuki, et al., 2021](https://ieeexplore.ieee.org/document/9419757) for more details.
#[derive(Gain, Debug)]
pub struct Greedy<D: Directivity> {
    /// The focal positions and amplitudes.
    pub foci: Vec<(Point3, Amplitude)>,
    /// The opinion of the Gain.
    pub option: GreedyOption<D>,
}

impl<D: Directivity> Greedy<D> {
    fn transfer_foci(
        trans: &Transducer,
        wavenumber: f32,
        dir: &UnitVector3,
        foci: &[Point3],
        res: &mut [Complex],
    ) {
        res.iter_mut().zip(foci.iter()).for_each(|(r, f)| {
            *r = propagate::<D>(trans, wavenumber, dir, f);
        });
    }
}

pub struct Context {
    g: Vec<Drive>,
}

impl GainContext for Context {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.g[tr.idx()]
    }
}

pub struct ContextGenerator {
    g: HashMap<usize, Vec<Drive>>,
}

impl GainContextGenerator for ContextGenerator {
    type Context = Context;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Context {
            g: self.g.remove(&device.idx()).unwrap(),
        }
    }
}

impl<D: Directivity> Gain for Greedy<D> {
    type G = ContextGenerator;

    // GRCOV_EXCL_START
    fn init(self) -> Result<Self::G, GainError> {
        unimplemented!()
    }
    // GRCOV_EXCL_STOP

    fn init_full(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
        _option: &DatagramOption,
    ) -> Result<Self::G, GainError> {
        let (foci, amps): (Vec<_>, Vec<_>) = self.foci.into_iter().unzip();

        let phase_candidates = (0..self.option.phase_div.get())
            .map(|i| {
                Complex::new(0., 2.0 * PI * i as f32 / self.option.phase_div.get() as f32).exp()
            })
            .collect::<Vec<_>>();

        let indices = {
            let mut indices: Vec<_> = if let Some(filter) = filter {
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

        let mut g: HashMap<_, _> = geometry
            .devices()
            .map(|dev| (dev.idx(), vec![Drive::NULL; dev.num_transducers()]))
            .collect();
        let mut cache = vec![Complex::new(0., 0.); foci.len()];
        let mut tmp = vec![Complex::new(0., 0.); foci.len()];
        indices.iter().for_each(|&(dev_idx, idx)| {
            Self::transfer_foci(
                &geometry[dev_idx][idx],
                geometry[dev_idx].wavenumber(),
                geometry[dev_idx].axial_direction(),
                &foci,
                &mut tmp,
            );
            let (phase, _) =
                phase_candidates
                    .iter()
                    .fold((Complex::ZERO, f32::INFINITY), |acc, &phase| {
                        let v = cache
                            .iter()
                            .zip(amps.iter())
                            .zip(tmp.iter())
                            .fold(0., |acc, ((c, a), f)| {
                                acc + (a.value - (f * phase + c).abs()).abs()
                            });
                        if v < acc.1 {
                            (phase, v)
                        } else {
                            acc
                        }
                    });
            cache.iter_mut().zip(tmp.iter()).for_each(|(c, a)| {
                *c += a * phase;
            });
            g.get_mut(&dev_idx).unwrap()[idx] = Drive {
                phase: Phase::from(phase),
                intensity: self.option.constraint.convert(1.0, 1.0),
            };
        });

        Ok(ContextGenerator { g })
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::acoustics::directivity::Sphere;

    use crate::tests::create_geometry;

    use super::{super::super::Pa, *};

    #[test]
    fn test_greedy_all() {
        let geometry = create_geometry(1, 1);

        let g = Greedy::<Sphere> {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: GreedyOption::default(),
        };
        assert_eq!(
            g.init_full(&geometry, None, &DatagramOption::default())
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
    fn test_greedy_all_disabled() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2, 1);
        geometry[0].enable = false;

        let g = Greedy::<Sphere> {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: GreedyOption::default(),
        };

        let mut g = g.init_full(&geometry, None, &DatagramOption::default())?;
        let f = g.generate(&geometry[1]);
        assert_eq!(
            geometry[1]
                .iter()
                .filter(|tr| f.calc(tr) != Drive::NULL)
                .count(),
            geometry[1].num_transducers()
        );

        Ok(())
    }

    #[test]
    fn test_greedy_filtered() {
        let geometry = create_geometry(1, 1);

        let g = Greedy::<Sphere> {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: GreedyOption::default(),
        };

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        assert_eq!(
            g.init_full(&geometry, Some(&filter), &DatagramOption::default())
                .map(|mut res| {
                    let f = res.generate(&geometry[0]);
                    geometry[0]
                        .iter()
                        .filter(|tr| f.calc(tr) != Drive::NULL)
                        .count()
                }),
            Ok(100),
        )
    }

    #[test]
    fn test_greedy_filtered_disabled() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2, 1);
        geometry[0].enable = false;

        let g = Greedy::<Sphere> {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: GreedyOption::default(),
        };

        let filter = geometry
            .devices()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();

        let mut g = g.init_full(&geometry, Some(&filter), &DatagramOption::default())?;
        let f = g.generate(&geometry[1]);
        assert_eq!(
            geometry[1]
                .iter()
                .filter(|tr| f.calc(tr) != Drive::NULL)
                .count(),
            100
        );

        Ok(())
    }
}
