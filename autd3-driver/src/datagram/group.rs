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

#[cfg(test)]
mod tests {
    use crate::datagram::tests::create_geometry;

    use super::*;

    use std::{
        convert::Infallible,
        sync::{Arc, Mutex},
    };

    pub struct NullOperationGenerator;

    impl OperationGenerator for NullOperationGenerator {
        type O1 = NullOp;
        type O2 = NullOp;

        fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
            (NullOp, NullOp)
        }
    }

    #[test]
    fn test_group_only_for_enabled() -> anyhow::Result<()> {
        #[derive(Debug)]
        pub struct TestDatagram;

        impl Datagram for TestDatagram {
            type G = NullOperationGenerator;
            type Error = Infallible;

            fn operation_generator(self, _: &mut Geometry) -> Result<Self::G, Self::Error> {
                Ok(NullOperationGenerator)
            }
        }

        let mut geometry = create_geometry(2, 1);

        geometry[0].enable = false;

        let check = Arc::new(Mutex::new([false; 2]));
        Group::new(
            |dev| {
                check.lock().unwrap()[dev.idx()] = true;
                Some(())
            },
            HashMap::from([((), TestDatagram)]),
        )
        .operation_generator(&mut geometry)?;

        assert!(!check.lock().unwrap()[0]);
        assert!(check.lock().unwrap()[1]);

        Ok(())
    }

    #[test]
    fn test_group_only_for_set() -> anyhow::Result<()> {
        #[derive(Debug)]
        pub struct TestDatagram {
            pub test: Arc<Mutex<Vec<bool>>>,
        }

        impl Datagram for TestDatagram {
            type G = NullOperationGenerator;
            type Error = Infallible;

            fn operation_generator(self, geometry: &mut Geometry) -> Result<Self::G, Self::Error> {
                geometry.iter().for_each(|dev| {
                    self.test.lock().unwrap()[dev.idx()] = dev.enable;
                });
                Ok(NullOperationGenerator)
            }
        }

        let mut geometry = create_geometry(3, 1);

        let test = Arc::new(Mutex::new(vec![false; 3]));
        Group::new(
            |dev| match dev.idx() {
                0 | 2 => Some(()),
                _ => None,
            },
            HashMap::from([((), TestDatagram { test: test.clone() })]),
        )
        .operation_generator(&mut geometry)?;

        assert!(test.lock().unwrap()[0]);
        assert!(!test.lock().unwrap()[1]);
        assert!(test.lock().unwrap()[2]);

        Ok(())
    }

    #[test]
    fn unknown_key() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2, 1);

        assert_eq!(
            Some(AUTDDriverError::UnknownKey("1".to_owned())),
            Group::new(|dev| Some(dev.idx()), HashMap::from([(0, Clear {})]))
                .operation_generator(&mut geometry)
                .err()
        );

        Ok(())
    }

    #[test]
    fn unused_key() -> anyhow::Result<()> {
        let mut geometry = create_geometry(2, 1);
        assert_eq!(
            Some(AUTDDriverError::UnusedKey("2".to_owned())),
            Group::new(
                |dev| Some(dev.idx()),
                HashMap::from([(0, Clear {}), (1, Clear {}), (2, Clear {})])
            )
            .operation_generator(&mut geometry)
            .err()
        );

        Ok(())
    }
}
