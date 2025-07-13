use std::mem::MaybeUninit;

use crate::{
    error::AUTDDriverError,
    firmware::driver::{Operation, Version},
};

use autd3_core::{
    datagram::{DatagramOption, DeviceFilter},
    derive::{Datagram, FirmwareLimits},
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
    fn dyn_generate(
        &mut self,
        device: &Device,
        version: Version,
    ) -> Option<(BoxedOperation, BoxedOperation)>;
}

pub struct DynOperationGenerator {
    pub(crate) g: Box<dyn DOperationGenerator>,
}

pub trait DDatagram: std::fmt::Debug {
    fn dyn_operation_generator(
        &mut self,
        geometry: &Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Box<dyn DOperationGenerator>, AUTDDriverError>;
    #[must_use]
    fn dyn_option(&self) -> DatagramOption;
    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<'geo, 'dev, 'tr, T: Datagram<'geo, 'dev, 'tr>> DDatagram for MaybeUninit<T>
where
    T::G: DOperationGenerator + 'static,
    AUTDDriverError: From<T::Error>,
{
    #[allow(clippy::missing_transmute_annotations)]
    fn dyn_operation_generator(
        &mut self,
        geometry: &Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Box<dyn DOperationGenerator>, AUTDDriverError> {
        let mut tmp = MaybeUninit::<T>::uninit();
        std::mem::swap(&mut tmp, self);
        let d = unsafe { tmp.assume_init() };
        Ok(Box::new(d.operation_generator(
            unsafe { std::mem::transmute(geometry) },
            env,
            filter,
            limits,
        )?))
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
    pub fn new<
        'geo,
        'dev,
        'tr,
        E,
        G: DOperationGenerator + 'static,
        D: Datagram<'geo, 'dev, 'tr, G = G, Error = E> + 'static,
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

impl<'geo, 'dev, 'tr> Datagram<'geo, 'dev, 'tr> for BoxedDatagram {
    type G = DynOperationGenerator;
    type Error = AUTDDriverError;

    fn operation_generator(
        self,
        geometry: &'geo Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        let Self { mut d } = self;
        Ok(DynOperationGenerator {
            g: d.dyn_operation_generator(geometry, env, filter, limits)?,
        })
    }

    fn option(&self) -> DatagramOption {
        self.d.dyn_option()
    }
}

impl<'dev, O: Operation<'dev>> DOperation for O
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

impl<'dev> Operation<'dev> for BoxedOperation {
    type Error = AUTDDriverError;

    fn required_size(&self, device: &'dev Device) -> usize {
        self.inner.required_size(device)
    }

    fn pack(&mut self, device: &'dev Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        self.inner.pack(device, tx)
    }

    fn is_done(&self) -> bool {
        self.inner.is_done()
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
