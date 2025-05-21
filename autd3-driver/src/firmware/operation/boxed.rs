use autd3_core::{datagram::Operation, geometry::Device};

use crate::error::AUTDDriverError;

trait DOperation: Send + Sync {
    #[must_use]
    fn required_size(&self, device: &Device) -> usize;
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError>;
    #[must_use]
    fn is_done(&self) -> bool;
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

#[doc(hidden)]
pub struct BoxedOperation {
    inner: Box<dyn DOperation>,
}

impl BoxedOperation {
    #[must_use]
    pub fn new<E, O: Operation<Error = E> + 'static>(op: O) -> Self
    where
        AUTDDriverError: From<E>,
    {
        Self {
            inner: Box::new(op),
        }
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

#[cfg(test)]
pub mod tests {
    use autd3_core::{
        derive::Transducer,
        geometry::{Point3, UnitQuaternion},
    };
    use rand::Rng;

    use super::*;

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
        let mut boxed_op = BoxedOperation::new(op);

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
}
