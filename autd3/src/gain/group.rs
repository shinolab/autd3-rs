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
    pub fn set<G: Gain + 'static>(mut self, key: K, gain: G) -> Self {
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
                                let mut filter = BitVec::<usize, Lsb0>::new();
                                filter.resize(dev.num_transducers(), false);
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
        let drives_cache = self
            .gain_map
            .iter()
            .map(|(k, g)| {
                Ok((
                    k.clone(),
                    g.calc(
                        geometry,
                        GainFilter::Filter(if let Some(f) = filters.get(k) {
                            f
                        } else {
                            return Err(AUTDInternalError::GainError(
                                "Unknown group key".to_owned(),
                            ));
                        }),
                    )?,
                ))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        geometry
            .devices()
            .map(|dev| {
                Ok((
                    dev.idx(),
                    dev.iter()
                        .map(|tr| {
                            if let Some(key) = (self.f)(dev, tr) {
                                Ok(if let Some(g) = drives_cache.get(&key) {
                                    g[&dev.idx()][tr.idx()]
                                } else {
                                    return Err(AUTDInternalError::GainError(
                                        "Unspecified group key".to_owned(),
                                    ));
                                })
                            } else {
                                Ok(Drive::null())
                            }
                        })
                        .collect::<Result<Vec<_>, _>>()?,
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

    use crate::gain::{Null, Plane};

    #[test]
    fn test_group() {
        let geometry: Geometry = Geometry::new(vec![
            AUTD3::new(Vector3::zeros()).into_device(0),
            AUTD3::new(Vector3::zeros()).into_device(1),
            AUTD3::new(Vector3::zeros()).into_device(2),
            AUTD3::new(Vector3::zeros()).into_device(3),
        ]);

        let gain = Group::new(|dev, tr| match (dev.idx(), tr.idx()) {
            (0, 0..=99) => Some("null"),
            (0, 100..=199) => Some("plane"),
            (1, 200..) => Some("plane2"),
            _ => None,
        })
        .set("null", Null::new())
        .set("plane", Plane::new(Vector3::zeros()))
        .set("plane2", Plane::new(Vector3::zeros()).with_intensity(0x1F));

        let drives = gain.calc(&geometry, GainFilter::All).unwrap();
        assert_eq!(drives.len(), 4);
        assert!(drives.values().all(|d| d.len() == AUTD3::NUM_TRANS_IN_UNIT));

        drives[&0].iter().enumerate().for_each(|(i, d)| match i {
            i if i <= 99 => {
                assert_eq!(d.phase.value(), 0);
                assert_eq!(d.intensity.value(), 0);
            }
            i if i <= 199 => {
                assert_eq!(d.phase.value(), 0);
                assert_eq!(d.intensity.value(), 0xFF);
            }
            _ => {
                assert_eq!(d.phase.value(), 0);
                assert_eq!(d.intensity.value(), 0);
            }
        });
        drives[&1].iter().enumerate().for_each(|(i, d)| match i {
            i if i <= 199 => {
                assert_eq!(d.phase.value(), 0);
                assert_eq!(d.intensity.value(), 0);
            }
            _ => {
                assert_eq!(d.phase.value(), 0);
                assert_eq!(d.intensity.value(), 0x1F);
            }
        });
        drives[&2].iter().for_each(|d| {
            assert_eq!(d.phase.value(), 0);
            assert_eq!(d.intensity.value(), 0);
        });
        drives[&3].iter().for_each(|d| {
            assert_eq!(d.phase.value(), 0);
            assert_eq!(d.intensity.value(), 0);
        });
    }

    #[test]
    fn test_group_unknown_key() {
        let geometry: Geometry = Geometry::new(vec![
            AUTD3::new(Vector3::zeros()).into_device(0),
            AUTD3::new(Vector3::zeros()).into_device(1),
        ]);

        let gain = Group::new(|_dev, tr| match tr.idx() {
            0..=99 => Some("plane"),
            100..=199 => Some("null"),
            _ => None,
        })
        .set("plane2", Plane::new(Vector3::zeros()));

        assert_eq!(
            gain.calc(&geometry, GainFilter::All).unwrap_err(),
            AUTDInternalError::GainError("Unknown group key".to_owned())
        );
    }

    #[test]
    fn test_group_unspecified_key() {
        let geometry: Geometry = Geometry::new(vec![
            AUTD3::new(Vector3::zeros()).into_device(0),
            AUTD3::new(Vector3::zeros()).into_device(1),
        ]);

        let gain = Group::new(|_dev, tr| match tr.idx() {
            0..=99 => Some("plane"),
            100..=199 => Some("null"),
            _ => None,
        })
        .set("plane", Plane::new(Vector3::zeros()));

        assert_eq!(
            gain.calc(&geometry, GainFilter::All).unwrap_err(),
            AUTDInternalError::GainError("Unspecified group key".to_owned())
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
