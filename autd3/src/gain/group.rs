use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use bitvec::prelude::*;

use autd3_driver::{
    derive::*,
    geometry::{Device, Geometry},
};

#[derive(Gain)]
pub struct Group<K, F>
where
    K: Hash + Eq + Clone + 'static,
    F: Fn(&Device, &Transducer) -> Option<K> + 'static,
{
    f: F,
    gain_map: HashMap<K, Box<dyn Gain>>,
}

impl<K, F> Group<K, F>
where
    K: Hash + Eq + Clone + 'static,
    F: Fn(&Device, &Transducer) -> Option<K> + 'static,
{
    /// Group by transducer
    ///
    /// # Arguments
    ///
    /// `f` - function to get key from transducer (currentry, transducer type annotation is required)
    ///
    /// # Example
    ///
    /// ```
    /// # use autd3::prelude::*;
    /// # let gain : autd3::gain::Group<_, _> =
    /// Group::new(|dev, tr| match tr.idx() {
    ///                 0..=100 => Some("null"),
    ///                 101.. => Some("focus"),
    ///                 _ => None,
    ///             })
    ///             .set("null", Null::new())
    ///             .set("focus", Focus::new(Vector3::new(0.0, 0.0, 150.0)));
    /// ```
    pub fn new(f: F) -> Group<K, F> {
        Group {
            f,
            gain_map: HashMap::new(),
        }
    }

    /// set gain
    ///
    /// # Arguments
    ///
    /// * `key` - key
    /// * `gain` - Gain
    ///
    pub fn set(mut self, key: K, gain: impl Gain + 'static) -> Self {
        self.gain_map.insert(key, Box::new(gain));
        self
    }

    fn get_filters(&self, geometry: &Geometry) -> HashMap<K, HashMap<usize, BitVec<usize, Lsb0>>> {
        let mut filters: HashMap<K, HashMap<usize, BitVec<usize, Lsb0>>> = HashMap::new();
        geometry.devices().for_each(|dev| {
            dev.iter().for_each(|tr| {
                if let Some(key) = (self.f)(dev, tr) {
                    match filters.get_mut(&key) {
                        Some(v) => match v.entry(dev.idx()) {
                            Entry::Occupied(mut e) => {
                                e.get_mut().set(tr.idx(), true);
                            }
                            Entry::Vacant(e) => {
                                let mut filter =
                                    BitVec::<usize, Lsb0>::repeat(false, dev.num_transducers());
                                filter.set(tr.idx(), true);
                                e.insert(filter);
                            }
                        },
                        None => {
                            let mut filter =
                                BitVec::<usize, Lsb0>::repeat(false, dev.num_transducers());
                            filter.set(tr.idx(), true);
                            filters.insert(key.clone(), [(dev.idx(), filter)].into());
                        }
                    }
                }
            })
        });
        filters
    }
}

impl<K, F> Gain for Group<K, F>
where
    K: Hash + Eq + Clone + 'static,
    F: Fn(&Device, &Transducer) -> Option<K> + 'static,
{
    fn calc(
        &self,
        geometry: &Geometry,
        _filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        let filters = self.get_filters(geometry);
        let drives_cache =
            self.gain_map
                .iter()
                .map(|(k, g)| {
                    Ok((
                        k.clone(),
                        g.calc(
                            geometry,
                            GainFilter::Filter(filters.get(k).ok_or(
                                AUTDInternalError::GainError("Unknown group key".to_owned()),
                            )?),
                        )?,
                    ))
                })
                .collect::<Result<HashMap<_, _>, AUTDInternalError>>()?;
        geometry
            .devices()
            .map(|dev| {
                Ok((
                    dev.idx(),
                    dev.iter()
                        .map(|tr| {
                            (self.f)(dev, tr).map_or_else(
                                || Ok(Drive::null()),
                                |key| {
                                    drives_cache
                                        .get(&key)
                                        .ok_or(AUTDInternalError::GainError(
                                            "Unspecified group key".to_owned(),
                                        ))
                                        .map(|g| g[&dev.idx()][tr.idx()])
                                },
                            )
                        })
                        .collect::<Result<Vec<_>, AUTDInternalError>>()?,
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::{
        autd3_device::AUTD3,
        geometry::{IntoDevice, Vector3},
    };

    use super::*;

    use crate::{
        gain::{Null, Plane},
        tests::create_geometry,
    };

    #[test]
    fn test_group() -> anyhow::Result<()> {
        let geometry = create_geometry(4);

        let gain = Group::new(|dev, tr| match (dev.idx(), tr.idx()) {
            (0, 0..=99) => Some("null"),
            (0, 100..=199) => Some("plane"),
            (1, 200..) => Some("plane2"),
            (3, _) => Some("plane"),
            _ => None,
        })
        .set("null", Null::new())
        .set("plane", Plane::new(Vector3::zeros()))
        .set("plane2", Plane::new(Vector3::zeros()).with_intensity(0x1F));

        let drives = gain.calc(&geometry, GainFilter::All)?;
        assert_eq!(4, drives.len());
        assert!(drives.values().all(|d| d.len() == AUTD3::NUM_TRANS_IN_UNIT));
        drives[&0].iter().enumerate().for_each(|(i, &d)| match i {
            i if i <= 99 => {
                assert_eq!(Drive::null(), d);
            }
            i if i <= 199 => {
                assert_eq!(Phase::new(0), d.phase);
                assert_eq!(EmitIntensity::MAX, d.intensity);
            }
            _ => {
                assert_eq!(Drive::null(), d);
            }
        });
        drives[&1].iter().enumerate().for_each(|(i, &d)| match i {
            i if i <= 199 => {
                assert_eq!(Drive::null(), d);
            }
            _ => {
                assert_eq!(Phase::new(0), d.phase);
                assert_eq!(EmitIntensity::new(0x1F), d.intensity);
            }
        });
        drives[&2].iter().for_each(|&d| {
            assert_eq!(Drive::null(), d);
        });
        drives[&3].iter().for_each(|d| {
            assert_eq!(Phase::new(0), d.phase);
            assert_eq!(EmitIntensity::MAX, d.intensity);
        });

        Ok(())
    }

    #[test]
    fn test_group_unknown_key() {
        let geometry = create_geometry(2);

        let gain = Group::new(|_dev, tr| match tr.idx() {
            0..=99 => Some("plane"),
            100..=199 => Some("null"),
            _ => None,
        })
        .set("plane2", Plane::new(Vector3::zeros()));

        assert_eq!(
            Err(AUTDInternalError::GainError("Unknown group key".to_owned())),
            gain.calc(&geometry, GainFilter::All)
        );
    }

    #[test]
    fn test_group_unspecified_key() {
        let geometry = create_geometry(2);

        let gain = Group::new(|_dev, tr| match tr.idx() {
            0..=99 => Some("plane"),
            100..=199 => Some("null"),
            _ => None,
        })
        .set("plane", Plane::new(Vector3::zeros()));

        assert_eq!(
            Err(AUTDInternalError::GainError(
                "Unspecified group key".to_owned()
            )),
            gain.calc(&geometry, GainFilter::All)
        );
    }

    #[test]
    fn test_group_derive() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
        let gain = Group::new(|_, _| -> Option<()> { None });
        let _ = gain.calc(&geometry, GainFilter::All);
        let _ = gain.operation();
    }
}
