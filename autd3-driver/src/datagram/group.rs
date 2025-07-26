use std::{collections::HashMap, fmt::Debug, hash::Hash, time::Duration};

use crate::error::AUTDDriverError;

use autd3_core::{
    datagram::{Datagram, DatagramOption, DeviceFilter, Inspectable, InspectionResult},
    environment::Environment,
    firmware::FirmwareLimits,
    geometry::{Device, Geometry},
};
use derive_more::Debug as DeriveDebug;
use itertools::Itertools;

/// [`Datagram`] that divide the devices into groups by given function and send different data to each group.
///
/// If the key is `None`, nothing is done for the devices corresponding to the key.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// # use autd3_driver::datagram::*;
///
/// Group {
///     key_map: |dev| match dev.idx() {
///         0 => Some("silencer"),
///         1 => Some("disable"),
///         _ => None,
///     },
///     datagram_map: HashMap::from([
///         ("silencer", Silencer::default()),
///         ("disable", Silencer::disable()),
///     ]),
/// };
/// ```
#[derive(Default, DeriveDebug)]
pub struct Group<K, D, F>
where
    F: Fn(&Device) -> Option<K>,
{
    /// Mapping function from device to group key.
    #[debug(ignore)]
    pub key_map: F,
    /// Map from group key to [`Datagram`].
    #[debug(ignore)]
    pub datagram_map: HashMap<K, D>,
}

impl<'a, K, D, F> Group<K, D, F>
where
    K: Hash + Eq + Debug,
    D: Datagram<'a>,
    F: Fn(&Device) -> Option<K>,
    AUTDDriverError: From<<D as Datagram<'a>>::Error>,
{
    /// Creates a new [`Group`].
    #[must_use]
    pub const fn new(key_map: F, datagram_map: HashMap<K, D>) -> Self {
        Self {
            key_map,
            datagram_map,
        }
    }

    fn generate_filter(key_map: &F, geometry: &Geometry) -> HashMap<K, DeviceFilter> {
        let mut filters: HashMap<K, DeviceFilter> = HashMap::new();
        geometry.iter().for_each(|dev| {
            if let Some(key) = key_map(dev) {
                if let Some(v) = filters.get_mut(&key) {
                    v.set_enable(dev.idx());
                } else {
                    filters.insert(
                        key,
                        DeviceFilter::from_fn(geometry, |dev_| dev_.idx() == dev.idx()),
                    );
                }
            }
        });
        filters
    }
}

pub struct GroupOpGenerator<K, F, G> {
    pub(crate) key_map: F,
    pub(crate) generators: HashMap<K, G>,
}

impl<'a, K, D, F> Datagram<'a> for Group<K, D, F>
where
    K: Hash + Eq + Debug,
    D: Datagram<'a>,
    F: Fn(&Device) -> Option<K>,
    AUTDDriverError: From<<D as Datagram<'a>>::Error>,
{
    type G = GroupOpGenerator<K, F, D::G>;
    type Error = AUTDDriverError;

    fn operation_generator(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        _: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        let Self {
            key_map,
            mut datagram_map,
        } = self;

        let filters = Self::generate_filter(&key_map, geometry);

        let generators = filters
            .into_iter()
            .map(|(k, filter)| {
                let datagram = datagram_map
                    .remove(&k)
                    .ok_or(AUTDDriverError::UnknownKey(format!("{k:?}")))?;
                Ok((
                    k,
                    datagram
                        .operation_generator(geometry, env, &filter, limits)
                        .map_err(AUTDDriverError::from)?,
                ))
            })
            .collect::<Result<_, AUTDDriverError>>()?;

        if !datagram_map.is_empty() {
            return Err(AUTDDriverError::UnusedKey(
                datagram_map.keys().map(|k| format!("{k:?}")).join(", "),
            ));
        }

        Ok(GroupOpGenerator {
            key_map,
            generators,
        })
    }

    fn option(&self) -> DatagramOption {
        self.datagram_map.values().map(|d| d.option()).fold(
            DatagramOption {
                timeout: Duration::ZERO,
                parallel_threshold: usize::MAX,
            },
            DatagramOption::merge,
        )
    }
}

impl<'a, K, D, F> Inspectable<'a> for Group<K, D, F>
where
    K: Hash + Eq + Debug,
    D: Datagram<'a> + Inspectable<'a>,
    F: Fn(&Device) -> Option<K>,
    AUTDDriverError: From<<D as Datagram<'a>>::Error>,
{
    type Result = D::Result;

    fn inspect(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        _: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<InspectionResult<Self::Result>, AUTDDriverError> {
        let Self {
            key_map,
            mut datagram_map,
        } = self;

        let filters = Self::generate_filter(&key_map, geometry);

        let results = filters
            .into_iter()
            .map(
                |(k, filter)| -> Result<Vec<Option<Self::Result>>, AUTDDriverError> {
                    {
                        let datagram = datagram_map
                            .remove(&k)
                            .ok_or(AUTDDriverError::UnknownKey(format!("{k:?}")))?;

                        let r = datagram
                            .inspect(geometry, env, &filter, limits)
                            .map_err(AUTDDriverError::from)?;

                        Ok(r.result)
                    }
                },
            )
            .collect::<Result<Vec<_>, _>>()?;

        Ok(InspectionResult {
            result: results
                .into_iter()
                .reduce(|a, b| a.into_iter().zip(b).map(|(a, b)| a.or(b)).collect())
                .unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::datagram::Clear;

    #[test]
    fn unknown_key() -> anyhow::Result<()> {
        let geometry = crate::autd3_device::tests::create_geometry(2);

        assert_eq!(
            Some(AUTDDriverError::UnknownKey("1".to_owned())),
            Group::new(|dev| Some(dev.idx()), HashMap::from([(0, Clear {})]))
                .operation_generator(
                    &geometry,
                    &Environment::default(),
                    &DeviceFilter::all_enabled(),
                    &FirmwareLimits::unused()
                )
                .err()
        );

        Ok(())
    }

    #[test]
    fn unused_key() -> anyhow::Result<()> {
        let geometry = crate::autd3_device::tests::create_geometry(2);

        assert_eq!(
            Some(AUTDDriverError::UnusedKey("2".to_owned())),
            Group::new(
                |dev| Some(dev.idx()),
                HashMap::from([(0, Clear {}), (1, Clear {}), (2, Clear {})])
            )
            .operation_generator(
                &geometry,
                &Environment::default(),
                &DeviceFilter::all_enabled(),
                &FirmwareLimits::unused()
            )
            .err()
        );

        Ok(())
    }
}
