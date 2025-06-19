use std::{collections::HashMap, num::NonZeroU8};

use crate::{Amplitude, Complex, constraint::EmissionConstraint};

use autd3_core::{
    acoustics::{
        directivity::{Directivity, Sphere},
        propagate,
    },
    common::PI,
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
            constraint: EmissionConstraint::Uniform(Intensity::MAX),
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

    fn generate_indices(geometry: &Geometry, filter: &TransducerFilter) -> Vec<(usize, usize)> {
        let mut indices: Vec<_> = if filter.is_all_enabled() {
            geometry
                .iter()
                .flat_map(|dev| dev.iter().map(|tr| (dev.idx(), tr.idx())))
                .collect()
        } else {
            geometry
                .iter()
                .flat_map(|dev| {
                    dev.iter()
                        .filter_map(|tr| filter.is_enabled(tr).then_some((dev.idx(), tr.idx())))
                })
                .collect()
        };
        indices.shuffle(&mut rand::rng());
        indices
    }

    fn alloc_result(geometry: &Geometry, filter: &TransducerFilter) -> HashMap<usize, Vec<Drive>> {
        geometry
            .iter()
            .filter(|dev| filter.is_enabled_device(dev))
            .map(|dev| (dev.idx(), vec![Drive::NULL; dev.num_transducers()]))
            .collect()
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

    fn init(self, geometry: &Geometry, filter: &TransducerFilter) -> Result<Self::G, GainError> {
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

        let indices = Self::generate_indices(geometry, filter);

        let mut g = Self::alloc_result(geometry, filter);
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
    fn test_greedy() {
        let geometry = create_geometry(1, 1);

        let g = Greedy::new(
            vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            GreedyOption::default(),
        );
        assert_eq!(
            g.init(&geometry, &TransducerFilter::all_enabled())
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

    #[rstest::rstest]
    #[case(itertools::iproduct!(0..2, 0..249).collect::<Vec<_>>(), TransducerFilter::all_enabled())]
    #[case(itertools::iproduct!(1..2, 0..249).collect::<Vec<_>>(), TransducerFilter::new(HashMap::from([(1, None)])))]
    #[test]
    fn test_greedy_indices(
        #[case] expected: Vec<(usize, usize)>,
        #[case] filter: TransducerFilter,
    ) {
        let geometry = create_geometry(2, 1);

        let mut indices =
            Greedy::<Sphere, AbsGreedyObjectiveFn>::generate_indices(&geometry, &filter);
        indices.sort();
        assert_eq!(expected, indices);
    }

    #[rstest::rstest]
    #[case(HashMap::from([(0, vec![Drive::NULL; 249]), (1, vec![Drive::NULL; 249])]), TransducerFilter::all_enabled())]
    #[case(HashMap::from([(1, vec![Drive::NULL; 249])]), TransducerFilter::new(HashMap::from([(1, None)])))]
    #[test]
    fn test_greedy_alloc_result(
        #[case] expected: HashMap<usize, Vec<Drive>>,
        #[case] filter: TransducerFilter,
    ) {
        let geometry = create_geometry(2, 1);
        assert_eq!(
            expected,
            Greedy::<Sphere, AbsGreedyObjectiveFn>::alloc_result(&geometry, &filter)
        );
    }

    #[test]
    fn test_greedy_filtered() {
        let geometry = create_geometry(1, 1);

        let g = Greedy {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: GreedyOption::default(),
        };

        let filter = TransducerFilter::new(
            geometry
                .iter()
                .map(|dev| {
                    (
                        dev.idx(),
                        Some(dev.iter().map(|tr| tr.idx() < 100).collect()),
                    )
                })
                .collect::<HashMap<_, _>>(),
        );
        assert_eq!(
            g.init(&geometry, &filter).map(|mut res| {
                let f = res.generate(&geometry[0]);
                geometry[0]
                    .iter()
                    .filter(|tr| f.calc(tr) != Drive::NULL)
                    .count()
            }),
            Ok(100),
        )
    }
}
