use std::{fmt::Debug, hash::Hash};

use super::OperationGenerator;
use crate::{datagram::GroupOpGenerator, geometry::Device};

use autd3_core::datagram::Datagram;

impl<K, F, D> OperationGenerator for GroupOpGenerator<K, F, D>
where
    K: Hash + Eq + Debug,
    F: Fn(&Device) -> Option<K>,
    D: Datagram,
    D::G: OperationGenerator,
{
    type O1 = <D::G as OperationGenerator>::O1;
    type O2 = <D::G as OperationGenerator>::O2;

    fn generate(&mut self, dev: &Device) -> Option<(Self::O1, Self::O2)> {
        let key = (self.key_map)(dev)?;
        self.generators.get_mut(&key).and_then(|g| g.generate(dev))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        convert::Infallible,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use super::{super::NullOp, *};
    use crate::datagram::Group;

    use autd3_core::{
        datagram::{DatagramOption, DeviceFilter, FirmwareLimits, Inspectable, InspectionResult},
        geometry::Geometry,
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
                _: &FirmwareLimits,
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
                _: &FirmwareLimits,
            ) -> Result<Self::G, Self::Error> {
                geometry.iter().for_each(|dev| {
                    self.test.lock().unwrap()[dev.idx()] = filter.is_enabled(dev);
                });
                Ok(NullOperationGenerator)
            }
        }

        let geometry = crate::autd3_device::tests::create_geometry(3);

        let test = Arc::new(Mutex::new(vec![false; 3]));
        Group::new(
            |dev| match dev.idx() {
                0 | 2 => Some(()),
                _ => None,
            },
            HashMap::from([((), TestDatagram { test: test.clone() })]),
        )
        .operation_generator(
            &geometry,
            &DeviceFilter::all_enabled(),
            &FirmwareLimits::unused(),
        )?;

        assert!(test.lock().unwrap()[0]);
        assert!(!test.lock().unwrap()[1]);
        assert!(test.lock().unwrap()[2]);

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
                _: &FirmwareLimits,
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
                _: &FirmwareLimits,
            ) -> Result<InspectionResult<Self::Result>, Self::Error> {
                Ok(InspectionResult::new(geometry, filter, |_| ()))
            }
        }

        let geometry = crate::autd3_device::tests::create_geometry(4);
        let r = Group::new(
            |dev| match dev.idx() {
                1 => None,
                _ => Some(()),
            },
            HashMap::from([((), TestDatagram {})]),
        )
        .inspect(
            &geometry,
            &DeviceFilter::all_enabled(),
            &FirmwareLimits::unused(),
        )?;

        assert!(r[0].is_some());
        assert!(r[1].is_none());
        assert!(r[2].is_some());
        assert!(r[3].is_some());

        Ok(())
    }
}
