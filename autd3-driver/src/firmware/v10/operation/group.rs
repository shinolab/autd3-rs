use std::{fmt::Debug, hash::Hash};

use super::OperationGenerator;
use crate::datagram::GroupOpGenerator;

use autd3_core::geometry::Device;

impl<'a, K, F, G> OperationGenerator<'a> for GroupOpGenerator<K, F, G>
where
    K: Hash + Eq + Debug,
    F: Fn(&Device) -> Option<K>,
    G: OperationGenerator<'a>,
{
    type O1 = <G as OperationGenerator<'a>>::O1;
    type O2 = <G as OperationGenerator<'a>>::O2;

    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
        let key = (self.key_map)(device)?;
        self.generators
            .get_mut(&key)
            .and_then(|g| g.generate(device))
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

    use super::*;
    use crate::{datagram::Group, firmware::driver::NullOp};

    use autd3_core::{
        datagram::{Datagram, DatagramOption, DeviceMask, Inspectable, InspectionResult},
        environment::Environment,
        firmware::FirmwareLimits,
        geometry::Geometry,
    };

    pub struct NullOperationGenerator;

    impl OperationGenerator<'_> for NullOperationGenerator {
        type O1 = NullOp;
        type O2 = NullOp;

        fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
            Some((NullOp, NullOp))
        }
    }

    #[test]
    fn test_group_option() -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Debug)]
        pub struct TestDatagram {
            pub option: DatagramOption,
        }

        impl Datagram<'_> for TestDatagram {
            type G = NullOperationGenerator;
            type Error = Infallible;

            // GRCOV_EXCL_START
            fn operation_generator(
                self,
                _: &Geometry,
                _: &Environment,
                _: &DeviceMask,
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
    fn test_group() -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Debug)]
        pub struct TestDatagram {
            pub test: Arc<Mutex<Vec<bool>>>,
        }

        impl Datagram<'_> for TestDatagram {
            type G = NullOperationGenerator;
            type Error = Infallible;

            fn operation_generator(
                self,
                geometry: &Geometry,
                _: &Environment,
                filter: &DeviceMask,
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
            &Environment::default(),
            &DeviceMask::AllEnabled,
            &FirmwareLimits::unused(),
        )?;

        assert!(test.lock().unwrap()[0]);
        assert!(!test.lock().unwrap()[1]);
        assert!(test.lock().unwrap()[2]);

        Ok(())
    }

    #[test]
    fn inspect() -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Debug)]
        pub struct TestDatagram {}

        impl Datagram<'_> for TestDatagram {
            type G = NullOperationGenerator;
            type Error = Infallible;

            // GRCOV_EXCL_START
            fn operation_generator(
                self,
                _: &Geometry,
                _: &Environment,
                _: &DeviceMask,
                _: &FirmwareLimits,
            ) -> Result<Self::G, Self::Error> {
                Ok(NullOperationGenerator)
            }
            // GRCOV_EXCL_STOP
        }

        impl Inspectable<'_> for TestDatagram {
            type Result = ();

            fn inspect(
                self,
                geometry: &Geometry,
                _: &Environment,
                filter: &DeviceMask,
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
            &Environment::default(),
            &DeviceMask::AllEnabled,
            &FirmwareLimits::unused(),
        )?;

        assert!(r[0].is_some());
        assert!(r[1].is_none());
        assert!(r[2].is_some());
        assert!(r[3].is_some());

        Ok(())
    }
}
