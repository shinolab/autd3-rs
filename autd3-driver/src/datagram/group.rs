use std::{collections::HashMap, fmt::Debug, hash::Hash, time::Duration};

use crate::datagram::*;

use autd3_core::{
    datagram::DatagramOption,
    derive::{Inspectable, InspectionResult},
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
///         0 => Some("clear"),
///         2 => Some("force-fan"),
///         _ => None,
///     },
///     datagram_map: HashMap::from([
///         ("clear", BoxedDatagram::new(Clear::default())),
///         ("force-fan", BoxedDatagram::new(ForceFan { f: |_| false })),
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

    fn generate_filter(key_map: F, geometry: &Geometry) -> HashMap<K, DeviceFilter> {
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

    fn generate(&mut self, dev: &Device) -> Option<(Self::O1, Self::O2)> {
        self.operations[dev.idx()].take()
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

    fn operation_generator(
        self,
        geometry: &Geometry,
        _: &DeviceFilter,
    ) -> Result<Self::G, Self::Error> {
        let Self {
            key_map,
            mut datagram_map,
        } = self;

        let filters = Self::generate_filter(key_map, geometry);

        let mut operations: Vec<_> = geometry.iter().map(|_| None).collect();
        filters
            .into_iter()
            .try_for_each(|(k, filter)| -> Result<(), AUTDDriverError> {
                {
                    let datagram = datagram_map
                        .remove(&k)
                        .ok_or(AUTDDriverError::UnknownKey(format!("{:?}", k)))?;

                    let mut generator = datagram
                        .operation_generator(geometry, &filter)
                        .map_err(AUTDDriverError::from)?;

                    operations
                        .iter_mut()
                        .zip(geometry.iter())
                        .filter(|(_, dev)| filter.is_enabled(dev))
                        .for_each(|(op, dev)| {
                            tracing::debug!("Generate operation for device {}", dev.idx());
                            *op = generator.generate(dev);
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

impl<K, D, F> Inspectable for Group<K, D, F>
where
    K: Hash + Eq + Debug,
    D: Datagram + Inspectable,
    F: Fn(&Device) -> Option<K>,
    D::G: OperationGenerator,
    AUTDDriverError: From<<D as Datagram>::Error>,
{
    type Result = D::Result;

    fn inspect(
        self,
        geometry: &Geometry,
        _: &DeviceFilter,
    ) -> Result<InspectionResult<Self::Result>, AUTDDriverError> {
        let Self {
            key_map,
            mut datagram_map,
        } = self;

        let filters = Self::generate_filter(key_map, geometry);

        let results = filters
            .into_iter()
            .map(
                |(k, filter)| -> Result<Vec<Option<Self::Result>>, AUTDDriverError> {
                    {
                        let datagram = datagram_map
                            .remove(&k)
                            .ok_or(AUTDDriverError::UnknownKey(format!("{:?}", k)))?;

                        let r = datagram
                            .inspect(geometry, &filter)
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

        fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
            Some((NullOp, NullOp))
        }
    }

    #[test]
    fn test_group_option() -> anyhow::Result<()> {
        #[derive(Debug)]
        pub struct TestDatagram {
            pub option: DatagramOption,
        }

        impl Datagram for TestDatagram {
            type G = NullOperationGenerator;
            type Error = Infallible;

            // GRCOV_EXCL_START
            fn operation_generator(
                self,
                _: &Geometry,
                _: &DeviceFilter,
            ) -> Result<Self::G, Self::Error> {
                Ok(NullOperationGenerator)
            }
            // GRCOV_EXCL_STOP

            fn option(&self) -> DatagramOption {
                self.option
            }
        }

        let option1 = DatagramOption {
            timeout: Duration::from_secs(1),
            parallel_threshold: 10,
        };
        let option2 = DatagramOption {
            timeout: Duration::from_secs(2),
            parallel_threshold: 5,
        };

        assert_eq!(
            option1.merge(option2),
            Group::new(
                |dev| Some(dev.idx()),
                HashMap::from([
                    (0, TestDatagram { option: option1 }),
                    (1, TestDatagram { option: option2 }),
                ]),
            )
            .option()
        );

        Ok(())
    }

    #[test]
    fn test_group() -> anyhow::Result<()> {
        #[derive(Debug)]
        pub struct TestDatagram {
            pub test: Arc<Mutex<Vec<bool>>>,
        }

        impl Datagram for TestDatagram {
            type G = NullOperationGenerator;
            type Error = Infallible;

            fn operation_generator(
                self,
                geometry: &Geometry,
                filter: &DeviceFilter,
            ) -> Result<Self::G, Self::Error> {
                geometry.iter().for_each(|dev| {
                    self.test.lock().unwrap()[dev.idx()] = filter.is_enabled(dev);
                });
                Ok(NullOperationGenerator)
            }
        }

        let geometry = create_geometry(3, 1);

        let test = Arc::new(Mutex::new(vec![false; 3]));
        Group::new(
            |dev| match dev.idx() {
                0 | 2 => Some(()),
                _ => None,
            },
            HashMap::from([((), TestDatagram { test: test.clone() })]),
        )
        .operation_generator(&geometry, &DeviceFilter::all_enabled())?;

        assert!(test.lock().unwrap()[0]);
        assert!(!test.lock().unwrap()[1]);
        assert!(test.lock().unwrap()[2]);

        Ok(())
    }

    #[test]
    fn unknown_key() -> anyhow::Result<()> {
        let geometry = create_geometry(2, 1);

        assert_eq!(
            Some(AUTDDriverError::UnknownKey("1".to_owned())),
            Group::new(|dev| Some(dev.idx()), HashMap::from([(0, Clear {})]))
                .operation_generator(&geometry, &DeviceFilter::all_enabled())
                .err()
        );

        Ok(())
    }

    #[test]
    fn unused_key() -> anyhow::Result<()> {
        let geometry = create_geometry(2, 1);
        assert_eq!(
            Some(AUTDDriverError::UnusedKey("2".to_owned())),
            Group::new(
                |dev| Some(dev.idx()),
                HashMap::from([(0, Clear {}), (1, Clear {}), (2, Clear {})])
            )
            .operation_generator(&geometry, &DeviceFilter::all_enabled())
            .err()
        );

        Ok(())
    }

    #[test]
    fn inspect() -> anyhow::Result<()> {
        #[derive(Debug)]
        pub struct TestDatagram {}

        impl Datagram for TestDatagram {
            type G = NullOperationGenerator;
            type Error = Infallible;

            // GRCOV_EXCL_START
            fn operation_generator(
                self,
                _: &Geometry,
                _: &DeviceFilter,
            ) -> Result<Self::G, Self::Error> {
                Ok(NullOperationGenerator)
            }
            // GRCOV_EXCL_STOP
        }

        impl Inspectable for TestDatagram {
            type Result = ();

            fn inspect(
                self,
                geometry: &Geometry,
                filter: &DeviceFilter,
            ) -> Result<InspectionResult<Self::Result>, Self::Error> {
                Ok(InspectionResult::new(geometry, filter, |_| ()))
            }
        }

        let geometry = create_geometry(4, 1);
        let r = Group::new(
            |dev| match dev.idx() {
                1 => None,
                _ => Some(()),
            },
            HashMap::from([((), TestDatagram {})]),
        )
        .inspect(&geometry, &DeviceFilter::all_enabled())?;

        assert!(r[0].is_some());
        assert!(r[1].is_none());
        assert!(r[2].is_some());
        assert!(r[3].is_some());

        Ok(())
    }
}
