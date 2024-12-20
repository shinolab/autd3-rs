use std::{collections::HashMap, num::NonZeroU8};

use crate::{constraint::EmissionConstraint, Amplitude, Complex};

use autd3_driver::{
    acoustics::{directivity::Directivity, propagate},
    datagram::GainContextGenerator,
    defined::PI,
    derive::*,
    firmware::{
        fpga::{Drive, EmitIntensity, Phase},
        operation::GainContext,
    },
    geometry::{Point3, Transducer, UnitVector3},
};

use bit_vec::BitVec;
use derive_more::Debug;
use nalgebra::ComplexField;
use rand::seq::SliceRandom;

#[derive(Gain, Builder, Debug)]
pub struct Greedy<D: Directivity> {
    #[get(ref)]
    foci: Vec<Point3>,
    #[get(ref)]
    amps: Vec<Amplitude>,
    #[get]
    #[set]
    phase_div: NonZeroU8,
    #[get]
    #[set]
    constraint: EmissionConstraint,
    #[debug(ignore)]
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity> Greedy<D> {
    pub fn new(iter: impl IntoIterator<Item = (Point3, Amplitude)>) -> Self {
        let (foci, amps) = iter.into_iter().unzip();
        Self {
            foci,
            amps,
            phase_div: NonZeroU8::new(16).unwrap(),
            constraint: EmissionConstraint::Uniform(EmitIntensity::MAX),
            _phantom: std::marker::PhantomData,
        }
    }

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

    fn generate(&mut self, device: &autd3_driver::geometry::Device) -> Self::Context {
        Context {
            g: self.g.remove(&device.idx()).unwrap(),
        }
    }
}

impl<D: Directivity> Gain for Greedy<D> {
    type G = ContextGenerator;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDDriverError> {
        let phase_candidates = (0..self.phase_div.get())
            .map(|i| Complex::new(0., 2.0 * PI * i as f32 / self.phase_div.get() as f32).exp())
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
        let mut cache = vec![Complex::new(0., 0.); self.foci.len()];
        let mut tmp = vec![Complex::new(0., 0.); self.foci.len()];
        indices.iter().for_each(|&(dev_idx, idx)| {
            Self::transfer_foci(
                &geometry[dev_idx][idx],
                geometry[dev_idx].wavenumber(),
                geometry[dev_idx].axial_direction(),
                &self.foci,
                &mut tmp,
            );
            let (phase, _) =
                phase_candidates
                    .iter()
                    .fold((Complex::ZERO, f32::INFINITY), |acc, &phase| {
                        let v = cache
                            .iter()
                            .zip(self.amps.iter())
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
            g.get_mut(&dev_idx).unwrap()[idx] =
                Drive::new(Phase::from(phase), self.constraint.convert(1.0, 1.0));
        });

        Ok(ContextGenerator { g })
    }
}

#[cfg(test)]
mod tests {
    use super::{super::super::Pa, *};
    use autd3_driver::{acoustics::directivity::Sphere, autd3_device::AUTD3, geometry::IntoDevice};

    #[test]
    fn test_greedy_all() {
        let geometry: Geometry =
            Geometry::new(vec![AUTD3::new(Point3::origin()).into_device(0)], 4);

        let g = Greedy::<Sphere>::new([(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)])
            .with_phase_div(NonZeroU8::MIN);

        assert_eq!(g.phase_div(), NonZeroU8::MIN);
        assert_eq!(
            g.constraint(),
            EmissionConstraint::Uniform(EmitIntensity::MAX)
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
        let mut geometry = Geometry::new(
            vec![
                AUTD3::new(Point3::origin()).into_device(0),
                AUTD3::new(Point3::origin()).into_device(1),
            ],
            4,
        );
        geometry[0].enable = false;

        let g = Greedy::<Sphere>::new([(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)]);

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
        let geometry: Geometry =
            Geometry::new(vec![AUTD3::new(Point3::origin()).into_device(0)], 4);

        let g = Greedy::<Sphere>::new([
            (Point3::new(10., 10., 100.), 5e3 * Pa),
            (Point3::new(-10., 10., 100.), 5e3 * Pa),
        ])
        .with_constraint(EmissionConstraint::Uniform(EmitIntensity::new(0xFF)));

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
        let mut geometry = Geometry::new(
            vec![
                AUTD3::new(Point3::origin()).into_device(0),
                AUTD3::new(Point3::origin()).into_device(1),
            ],
            4,
        );
        geometry[0].enable = false;

        let g = Greedy::<Sphere>::new([(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)]);

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
