use std::{collections::HashMap, fmt::Debug, hash::Hash, time::Duration};

use crate::datagram::*;

use autd3_core::{datagram::DatagramOption, gain::BitVec};
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
/// use autd3_driver::datagram::IntoBoxedDatagram;
///
/// Group {
///     key_map: |dev| match dev.idx() {
///         0 => Some("clear"),
///         2 => Some("force-fan"),
///         _ => None,
///     },
///     datagram_map: HashMap::from([
///         ("clear", Clear::default().into_boxed()),
///         ("force-fan", ForceFan { f: |_| false }.into_boxed()),
///     ]),
/// };
/// ```
#[derive(Default, DeriveDebug)]
pub struct Group<K, D, F>
where
    K: Hash + Eq + Debug,
    D: Datagram,
    F: Fn(&Device) -> Option<K>,
    D::G: OperationGenerator,
    AUTDDriverError: From<<D as Datagram>::Error>,
{
    /// Mapping function from device to group key.
    #[debug(ignore)]
    pub key_map: F,
    /// Map from group key to [`Datagram`].
    #[debug(ignore)]
    pub datagram_map: HashMap<K, D>,
}

impl<K, D, F> Group<K, D, F>
where
    K: Hash + Eq + Debug,
    D: Datagram,
    F: Fn(&Device) -> Option<K>,
    D::G: OperationGenerator,
    AUTDDriverError: From<<D as Datagram>::Error>,
{
    /// Creates a new [`Group`].
    #[must_use]
    pub const fn new(key_map: F, datagram_map: HashMap<K, D>) -> Self {
        Self {
            key_map,
            datagram_map,
        }
    }
}

pub struct GroupOpGenerator<D>
where
    D: Datagram,
    D::G: OperationGenerator,
{
    #[allow(clippy::type_complexity)]
    operations: Vec<
        Option<(
            <D::G as OperationGenerator>::O1,
            <D::G as OperationGenerator>::O2,
        )>,
    >,
}

impl<D> OperationGenerator for GroupOpGenerator<D>
where
    D: Datagram,
    D::G: OperationGenerator,
{
    type O1 = <D::G as OperationGenerator>::O1;
    type O2 = <D::G as OperationGenerator>::O2;

    fn generate(&mut self, dev: &Device) -> (Self::O1, Self::O2) {
        self.operations[dev.idx()]
            .take()
            .expect("Group operation for is not generated")
    }
}

impl<K, D, F> Datagram for Group<K, D, F>
where
    K: Hash + Eq + Debug,
    D: Datagram,
    F: Fn(&Device) -> Option<K>,
    D::G: OperationGenerator,
    AUTDDriverError: From<<D as Datagram>::Error>,
{
    type G = GroupOpGenerator<D>;
    type Error = AUTDDriverError;

    fn operation_generator(self, geometry: &mut Geometry) -> Result<Self::G, Self::Error> {
        let Self {
            key_map,
            mut datagram_map,
        } = self;

        let filters = {
            let num_devices = geometry.iter().len();
            let mut filters: HashMap<K, BitVec> = HashMap::new();
            geometry.devices().for_each(|dev| {
                if let Some(key) = key_map(dev) {
                    if let Some(v) = filters.get_mut(&key) {
                        v.set(dev.idx(), true);
                    } else {
                        filters.insert(key, BitVec::from_fn(num_devices, |i| i == dev.idx()));
                    }
                }
            });
            filters
        };

        let enable_store = geometry.iter().map(|dev| dev.enable).collect::<Vec<_>>();

        let mut operations: Vec<_> = geometry.devices().map(|_| None).collect();

        filters
            .into_iter()
            .try_for_each(|(k, filter)| -> Result<(), AUTDDriverError> {
                {
                    let datagram = datagram_map
                        .remove(&k)
                        .ok_or(AUTDDriverError::UnknownKey(format!("{:?}", k)))?;

                    // set enable flag for each device
                    // This is not required for the operation except `Gain`s which cannot be calculated independently for each device, such as `autd3-gain-holo`.
                    geometry.devices_mut().for_each(|dev| {
                        dev.enable = filter[dev.idx()];
                    });

                    let mut generator = datagram
                        .operation_generator(geometry)
                        .map_err(AUTDDriverError::from)?;

                    // restore enable flag
                    geometry
                        .iter_mut()
                        .zip(enable_store.iter())
                        .for_each(|(dev, &enable)| {
                            dev.enable = enable;
                        });

                    operations
                        .iter_mut()
                        .zip(geometry.devices())
                        .filter(|(_, dev)| filter[dev.idx()])
                        .for_each(|(op, dev)| {
                            tracing::debug!("Generate operation for device {}", dev.idx());
                            let (op1, op2) = generator.generate(dev);
                            *op = Some((op1, op2));
                        });
                    Ok(())
                }
            })?;

        if !datagram_map.is_empty() {
            return Err(AUTDDriverError::UnusedKey(
                datagram_map.keys().map(|k| format!("{:?}", k)).join(", "),
            ));
        }

        Ok(GroupOpGenerator { operations })
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
