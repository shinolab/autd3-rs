use super::OperationGenerator;
use crate::{
    error::AUTDDriverError,
    firmware::driver::{
        BoxedOperation, DOperationGenerator, DynOperationGenerator, Operation, Version,
    },
};

use autd3_core::geometry::Device;

impl<'a> OperationGenerator<'a> for DynOperationGenerator {
    type O1 = BoxedOperation;
    type O2 = BoxedOperation;

    fn generate(&mut self, device: &'a Device, version: Version) -> Option<(Self::O1, Self::O2)> {
        self.g.dyn_generate(device, version)
    }
}

impl<'a, G: OperationGenerator<'a>> DOperationGenerator for G
where
    G::O1: 'static,
    G::O2: 'static,
    AUTDDriverError: From<<G::O1 as Operation<'a>>::Error> + From<<G::O2 as Operation<'a>>::Error>,
{
    #[allow(clippy::missing_transmute_annotations)]
    fn dyn_generate(
        &mut self,
        device: &Device,
        version: Version,
    ) -> Option<(BoxedOperation, BoxedOperation)> {
        self.generate(unsafe { std::mem::transmute(device) }, version)
            .map(move |(o1, o2)| {
                (
                    BoxedOperation {
                        inner: Box::new(o1),
                    },
                    BoxedOperation {
                        inner: Box::new(o2),
                    },
                )
            })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::firmware::driver::{BoxedDatagram, operation::boxed::tests::TestOp};

    use autd3_core::{
        datagram::{Datagram, DatagramOption, DeviceMask},
        environment::Environment,
        firmware::FirmwareLimits,
        geometry::Geometry,
    };

    #[derive(Clone, Copy)]
    struct TestDatagram {
        pub option: DatagramOption,
    }

    struct TestOperationGenerator;

    impl OperationGenerator<'_> for TestOperationGenerator {
        type O1 = TestOp;
        type O2 = TestOp;

        fn generate(&mut self, _device: &Device, _: Version) -> Option<(Self::O1, Self::O2)> {
            Some((
                Self::O1 {
                    req_size: 1,
                    pack_size: 2,
                    done: false,
                },
                Self::O2 {
                    req_size: 3,
                    pack_size: 4,
                    done: true,
                },
            ))
        }
    }

    impl Datagram<'_> for TestDatagram {
        type G = TestOperationGenerator;
        type Error = AUTDDriverError;

        fn operation_generator(
            self,
            _geometry: &Geometry,
            _: &Environment,
            _: &DeviceMask,
            _: &FirmwareLimits,
        ) -> Result<Self::G, Self::Error> {
            Ok(Self::G {})
        }

        fn option(&self) -> DatagramOption {
            self.option
        }
    }

    impl std::fmt::Debug for TestDatagram {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "test")
        }
    }

    #[test]
    fn boxed_datagram() -> anyhow::Result<()> {
        let geometry = crate::autd3_device::tests::create_geometry(1);

        let d = TestDatagram {
            option: Default::default(),
        };
        let bd = BoxedDatagram::new(d);

        assert_eq!(d.option(), bd.option());

        let mut g = Datagram::operation_generator(
            bd,
            &geometry,
            &Environment::new(),
            &DeviceMask::AllEnabled,
            &FirmwareLimits::unused(),
        )?;
        let (mut op1, mut op2) = g.generate(&geometry[0], Version::V12).unwrap();
        assert_eq!(1, Operation::required_size(&op1, &geometry[0]));
        assert_eq!(2, Operation::pack(&mut op1, &geometry[0], &mut [])?);
        assert!(!Operation::is_done(&op1));
        assert_eq!(3, Operation::required_size(&op2, &geometry[0]));
        assert_eq!(4, Operation::pack(&mut op2, &geometry[0], &mut [])?);
        assert!(Operation::is_done(&op2));

        Ok(())
    }
}
