use std::{collections::HashMap, num::NonZeroU8};

use crate::{Amplitude, Complex, constraint::EmissionConstraint};

use autd3_core::{
    acoustics::{
        directivity::{Directivity, Sphere},
        propagate,
    },
    defined::PI,
    derive::*,
    geometry::{Point3, UnitVector3},
};

use derive_more::Debug;
use nalgebra::ComplexField;
use rand::prelude::*;

/// The trait for the objective function of [`Greedy`].
pub trait GreedyObjectiveFn: std::fmt::Debug + Clone + Copy + PartialEq {
    /// The objective function for the greedy algorithm.
    fn objective_func(c: Complex, a: Amplitude) -> f32;
}

/// The objective function for [`Greedy`] that minimizes the absolute value of the difference
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbsGreedyObjectiveFn;

impl GreedyObjectiveFn for AbsGreedyObjectiveFn {
    fn objective_func(c: Complex, a: Amplitude) -> f32 {
        (a.value - c.abs()).abs()
    }
}

/// The option of [`Greedy`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GreedyOption<D: Directivity, F: GreedyObjectiveFn> {
    /// The quantization levels of the phase.
    pub phase_quantization_levels: NonZeroU8,
    /// The transducers' emission constraint.
    pub constraint: EmissionConstraint,
    /// The objective function.
    pub objective_func: F,
    #[doc(hidden)]
    pub __phantom: std::marker::PhantomData<D>,
}

impl Default for GreedyOption<Sphere, AbsGreedyObjectiveFn> {
    fn default() -> Self {
        Self {
            phase_quantization_levels: NonZeroU8::new(16).unwrap(),
            constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
            objective_func: AbsGreedyObjectiveFn,
            __phantom: std::marker::PhantomData,
        }
    }
}

/// Greedy algorithm and Brute-force search
///
/// [`Greedy`] is based on the method of optimizing by brute-force search and greedy algorithm by discretizing the phase.
/// See [Suzuki, et al., 2021](https://ieeexplore.ieee.org/document/9419757) for more details.
#[derive(Gain, Debug)]
pub struct Greedy<D: Directivity, F: GreedyObjectiveFn> {
    /// The focal positions and amplitudes.
    pub foci: Vec<(Point3, Amplitude)>,
    /// The opinion of the Gain.
    pub option: GreedyOption<D, F>,
}

impl<D: Directivity, F: GreedyObjectiveFn> Greedy<D, F> {
    /// Create a new [`Greedy`].
    #[must_use]
    pub fn new(
        foci: impl IntoIterator<Item = (Point3, Amplitude)>,
        option: GreedyOption<D, F>,
    ) -> Self {
        Self {
            foci: foci.into_iter().collect(),
            option,
        }
    }
}

impl<D: Directivity, F: GreedyObjectiveFn> Greedy<D, F> {
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

pub struct Impl {
    g: Vec<Drive>,
}

impl GainCalculator for Impl {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.g[tr.idx()]
    }
}

pub struct Generator {
    g: HashMap<usize, Vec<Drive>>,
}

impl GainCalculatorGenerator for Generator {
    type Calculator = Impl;

    fn generate(&mut self, device: &Device) -> Self::Calculator {
        Impl {
            g: self.g.remove(&device.idx()).unwrap(),
        }
    }
}

impl<D: Directivity, F: GreedyObjectiveFn> Gain for Greedy<D, F> {
    type G = Generator;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
    ) -> Result<Self::G, GainError> {
        let (foci, amps): (Vec<_>, Vec<_>) = self.foci.into_iter().unzip();

        let phase_candidates = (0..self.option.phase_quantization_levels.get())
            .map(|i| {
                Complex::new(
                    0.,
                    2.0 * PI * i as f32 / self.option.phase_quantization_levels.get() as f32,
                )
                .exp()
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
            indices.shuffle(&mut rand::rng());
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
                                acc + F::objective_func(f * phase + c, *a)
                            });
                        if v < acc.1 { (phase, v) } else { acc }
                    });
            cache.iter_mut().zip(tmp.iter()).for_each(|(c, a)| {
                *c += a * phase;
            });
            g.get_mut(&dev_idx).unwrap()[idx] = Drive {
                phase: Phase::from(phase),
                intensity: self.option.constraint.convert(1.0, 1.0),
            };
        });

        Ok(Generator { g })
    }
}

#[cfg(test)]
mod tests {

    use crate::tests::create_geometry;

    use super::{super::super::Pa, *};

    #[test]
    fn test_greedy_all() {
        let geometry = create_geometry(1, 1);

        let g = Greedy::new(
            vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            GreedyOption::default(),
        );
        assert_eq!(
            g.init(&geometry, None).map(|mut res| {
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

        let g = Greedy {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: GreedyOption::default(),
        };

        let mut g = g.init(&geometry, None)?;
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

        let g = Greedy {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: GreedyOption::default(),
        };

        let filter = geometry
            .iter()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();
        assert_eq!(
            g.init(&geometry, Some(&filter)).map(|mut res| {
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

        let g = Greedy {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: GreedyOption::default(),
        };

        let filter = geometry
            .devices()
            .map(|dev| (dev.idx(), dev.iter().map(|tr| tr.idx() < 100).collect()))
            .collect::<HashMap<_, _>>();

        let mut g = g.init(&geometry, Some(&filter))?;
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
