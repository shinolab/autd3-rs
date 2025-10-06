use std::mem::MaybeUninit;

use crate::{
    error::AUTDDriverError,
    firmware::operation::{Operation, OperationGenerator},
};

use autd3_core::{
    datagram::{DatagramOption, DeviceMask},
    derive::Datagram,
    environment::Environment,
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

#[doc(hidden)]
pub struct DynOperationGenerator {
    pub(crate) g: Box<dyn DOperationGenerator>,
}

pub trait DDatagram {
    fn dyn_operation_generator(
        &mut self,
        geometry: &Geometry,
        env: &Environment,
        filter: &DeviceMask,
    ) -> Result<Box<dyn DOperationGenerator>, AUTDDriverError>;
    #[must_use]
    fn dyn_option(&self) -> DatagramOption;
}

impl<'a, T: Datagram<'a>> DDatagram for MaybeUninit<T>
where
    T::G: DOperationGenerator + 'static,
    AUTDDriverError: From<T::Error>,
{
    #[allow(clippy::missing_transmute_annotations)]
    fn dyn_operation_generator(
        &mut self,
        geometry: &Geometry,
        env: &Environment,
        filter: &DeviceMask,
    ) -> Result<Box<dyn DOperationGenerator>, AUTDDriverError> {
        let mut tmp = MaybeUninit::<T>::uninit();
        std::mem::swap(&mut tmp, self);
        let d = unsafe { tmp.assume_init() };
        Ok(Box::new(d.operation_generator(
            unsafe { std::mem::transmute(geometry) },
            env,
            filter,
        )?))
    }

    fn dyn_option(&self) -> DatagramOption {
        unsafe { self.assume_init_ref() }.option()
    }
}

/// Boxed [`Datagram`].
pub struct BoxedDatagram {
    d: Box<dyn DDatagram>,
}

impl BoxedDatagram {
    /// Creates a new [`BoxedDatagram`].
    pub fn new<
        'a,
        E,
        G: DOperationGenerator + 'static,
        D: Datagram<'a, G = G, Error = E> + 'static,
    >(
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

impl<'a> Datagram<'a> for BoxedDatagram {
    type G = DynOperationGenerator;
    type Error = AUTDDriverError;

    fn operation_generator(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
    ) -> Result<Self::G, Self::Error> {
        let Self { mut d } = self;
        Ok(DynOperationGenerator {
            g: d.dyn_operation_generator(geometry, env, filter)?,
        })
    }

    fn option(&self) -> DatagramOption {
        self.d.dyn_option()
    }
}

impl<'a, O: Operation<'a>> DOperation for O
where
    AUTDDriverError: From<O::Error>,
{
    #[allow(clippy::missing_transmute_annotations)]
    fn required_size(&self, device: &Device) -> usize {
        O::required_size(self, unsafe { std::mem::transmute(device) })
    }

    #[allow(clippy::missing_transmute_annotations)]
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        Ok(O::pack(self, unsafe { std::mem::transmute(device) }, tx)?)
    }

    fn is_done(&self) -> bool {
        O::is_done(self)
    }
}

impl<'a> Operation<'a> for BoxedOperation {
    type Error = AUTDDriverError;

    fn required_size(&self, device: &'a Device) -> usize {
        self.inner.required_size(device)
    }

    fn pack(&mut self, device: &'a Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        self.inner.pack(device, tx)
    }

    fn is_done(&self) -> bool {
        self.inner.is_done()
    }
}

impl<'a> OperationGenerator<'a> for DynOperationGenerator {
    type O1 = BoxedOperation;
    type O2 = BoxedOperation;

    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
        self.g.dyn_generate(device)
    }
}

impl<'a, G: OperationGenerator<'a>> DOperationGenerator for G
where
    G::O1: 'static,
    G::O2: 'static,
    AUTDDriverError: From<<G::O1 as Operation<'a>>::Error> + From<<G::O2 as Operation<'a>>::Error>,
{
    #[allow(clippy::missing_transmute_annotations)]
    fn dyn_generate(&mut self, device: &Device) -> Option<(BoxedOperation, BoxedOperation)> {
        self.generate(unsafe { std::mem::transmute(device) })
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

    use rand::Rng;

    use autd3_core::geometry::{Point3, Transducer, UnitQuaternion};

    #[derive(Clone, Copy)]
    pub struct TestOp {
        pub req_size: usize,
        pub pack_size: usize,
        pub done: bool,
    }

    impl Operation<'_> for TestOp {
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

        let device = Device::new(
            UnitQuaternion::identity(),
            vec![Transducer::new(Point3::origin())],
        );

        let mut boxed_op = BoxedOperation {
            inner: Box::new(op),
        };
        {
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
    }
}
