use std::mem::MaybeUninit;

use super::{Operation, OperationGenerator};
use crate::error::AUTDDriverError;

use autd3_core::{
    datagram::{DatagramOption, DeviceFilter},
    derive::{Datagram, FirmwareLimits},
    geometry::{Device, Geometry},
};

pub trait DOperation: Send + Sync {
    #[must_use]
    fn required_size(&self, device: &Device) -> usize;
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError>;
    #[must_use]
    fn is_done(&self) -> bool;
}

#[doc(hidden)]
pub struct BoxedOperation {
    pub(crate) inner: Box<dyn DOperation>,
}

#[doc(hidden)]
pub trait DOperationGenerator {
    #[must_use]
    fn dyn_generate(&mut self, device: &Device) -> Option<(BoxedOperation, BoxedOperation)>;
}

pub struct DynOperationGenerator {
    pub(crate) g: Box<dyn DOperationGenerator>,
}

pub trait DDatagram: std::fmt::Debug {
    fn dyn_operation_generator(
        &mut self,
        geometry: &Geometry,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Box<dyn DOperationGenerator>, AUTDDriverError>;
    #[must_use]
    fn dyn_option(&self) -> DatagramOption;
    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<E, G: DOperationGenerator + 'static, T: Datagram<G = G, Error = E>> DDatagram
    for MaybeUninit<T>
where
    AUTDDriverError: From<E>,
{
    fn dyn_operation_generator(
        &mut self,
        geometry: &Geometry,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Box<dyn DOperationGenerator>, AUTDDriverError> {
        let mut tmp = MaybeUninit::<T>::uninit();
        std::mem::swap(&mut tmp, self);
        let d = unsafe { tmp.assume_init() };
        Ok(Box::new(d.operation_generator(geometry, filter, limits)?))
    }

    fn dyn_option(&self) -> DatagramOption {
        unsafe { self.assume_init_ref() }.option()
    }

    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { self.assume_init_ref() }.fmt(f)
    }
}

/// Boxed [`Datagram`].
pub struct BoxedDatagram {
    d: Box<dyn DDatagram>,
}

impl std::fmt::Debug for BoxedDatagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.d.dyn_fmt(f)
    }
}

impl BoxedDatagram {
    /// Creates a new [`BoxedDatagram`].
    pub fn new<E, G: DOperationGenerator + 'static, D: Datagram<G = G, Error = E> + 'static>(
        d: D,
    ) -> Self
    where
        AUTDDriverError: From<E>,
    {
        BoxedDatagram {
            d: Box::new(MaybeUninit::new(d)),
        }
    }
}

impl Datagram for BoxedDatagram {
    type G = DynOperationGenerator;
    type Error = AUTDDriverError;

    fn operation_generator(
        self,
        geometry: &Geometry,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        let Self { mut d } = self;
        Ok(DynOperationGenerator {
            g: d.dyn_operation_generator(geometry, filter, limits)?,
        })
    }

    fn option(&self) -> DatagramOption {
        self.d.dyn_option()
    }
}

impl<E, O: Operation<Error = E>> DOperation for O
where
    AUTDDriverError: From<E>,
{
    fn required_size(&self, device: &Device) -> usize {
        O::required_size(self, device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        Ok(O::pack(self, device, tx)?)
    }

    fn is_done(&self) -> bool {
        O::is_done(self)
    }
}

impl Operation for BoxedOperation {
    type Error = AUTDDriverError;

    fn required_size(&self, device: &Device) -> usize {
        self.inner.required_size(device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        self.inner.pack(device, tx)
    }

    fn is_done(&self) -> bool {
        self.inner.is_done()
    }
}

impl OperationGenerator for DynOperationGenerator {
    type O1 = BoxedOperation;
    type O2 = BoxedOperation;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        self.g.dyn_generate(device)
    }
}

impl<G: OperationGenerator> DOperationGenerator for G
where
    G::O1: 'static,
    G::O2: 'static,
    AUTDDriverError: From<<G::O1 as Operation>::Error> + From<<G::O2 as Operation>::Error>,
{
    fn dyn_generate(&mut self, device: &Device) -> Option<(BoxedOperation, BoxedOperation)> {
        self.generate(device).map(|(o1, o2)| {
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

    use rand::Rng;

    use autd3_core::{
        datagram::{Datagram, DatagramOption, DeviceFilter},
        geometry::{Geometry, Point3, Transducer, UnitQuaternion},
    };

    #[derive(Clone, Copy)]
    pub struct TestOp {
        pub req_size: usize,
        pub pack_size: usize,
        pub done: bool,
    }

    impl Operation for TestOp {
        type Error = AUTDDriverError;

        fn required_size(&self, _device: &Device) -> usize {
            self.req_size
        }

        fn pack(&mut self, _device: &Device, _tx: &mut [u8]) -> Result<usize, Self::Error> {
            Ok(self.pack_size)
        }

        fn is_done(&self) -> bool {
            self.done
        }
    }

    #[test]
    fn test_boxed_operation() {
        let mut rng = rand::rng();

        let mut op = TestOp {
            req_size: rng.random::<u32>() as usize,
            pack_size: rng.random::<u32>() as usize,
            done: rng.random(),
        };
        let mut boxed_op = BoxedOperation {
            inner: Box::new(op),
        };

        let device = Device::new(
            UnitQuaternion::identity(),
            vec![Transducer::new(Point3::origin())],
        );

        assert_eq!(
            Operation::required_size(&op, &device),
            Operation::required_size(&boxed_op, &device)
        );
        assert_eq!(
            Operation::pack(&mut op, &device, &mut []),
            Operation::pack(&mut boxed_op, &device, &mut [])
        );
        assert_eq!(Operation::is_done(&op), Operation::is_done(&boxed_op));
    }

    #[derive(Clone, Copy)]
    struct TestDatagram {
        pub option: DatagramOption,
    }

    struct TestOperationGenerator;

    impl OperationGenerator for TestOperationGenerator {
        type O1 = TestOp;
        type O2 = TestOp;

        fn generate(&mut self, _device: &Device) -> Option<(Self::O1, Self::O2)> {
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

    impl Datagram for TestDatagram {
        type G = TestOperationGenerator;
        type Error = AUTDDriverError;

        fn operation_generator(
            self,
            _geometry: &Geometry,
            _: &DeviceFilter,
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
            &DeviceFilter::all_enabled(),
            &FirmwareLimits::unused(),
        )?;
        let (mut op1, mut op2) = g.generate(&geometry[0]).unwrap();
        assert_eq!(1, Operation::required_size(&op1, &geometry[0]));
        assert_eq!(2, Operation::pack(&mut op1, &geometry[0], &mut [])?);
        assert!(!Operation::is_done(&op1));
        assert_eq!(3, Operation::required_size(&op2, &geometry[0]));
        assert_eq!(4, Operation::pack(&mut op2, &geometry[0], &mut [])?);
        assert!(Operation::is_done(&op2));

        Ok(())
    }

    #[test]
    fn boxed_fmt() {
        let d = TestDatagram {
            option: Default::default(),
        };
        let bd = BoxedDatagram::new(d);
        assert_eq!(format!("{:?}", bd), "test");
    }
}
