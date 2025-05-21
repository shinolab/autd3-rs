use std::mem::MaybeUninit;

use crate::{
    datagram::Datagram,
    error::AUTDDriverError,
    firmware::operation::{BoxedOperation, OperationGenerator},
};
use autd3_core::{
    datagram::{DatagramOption, Operation},
    geometry::{Device, Geometry},
};

pub trait DOperationGenerator {
    #[must_use]
    fn dyn_generate(&mut self, device: &Device) -> (BoxedOperation, BoxedOperation);
}

#[cfg(feature = "lightweight")]
type DynDOperationGenerator = Box<dyn DOperationGenerator + Send>;
#[cfg(not(feature = "lightweight"))]
type DynDOperationGenerator = Box<dyn DOperationGenerator>;

pub struct DynOperationGenerator {
    g: DynDOperationGenerator,
}

impl OperationGenerator for DynOperationGenerator {
    type O1 = BoxedOperation;
    type O2 = BoxedOperation;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        self.g.dyn_generate(device)
    }
}

impl<G: OperationGenerator> DOperationGenerator for G
where
    G::O1: 'static,
    G::O2: 'static,
    AUTDDriverError: From<<G::O1 as Operation>::Error> + From<<G::O2 as Operation>::Error>,
{
    fn dyn_generate(&mut self, device: &Device) -> (BoxedOperation, BoxedOperation) {
        let (o1, o2) = self.generate(device);
        (BoxedOperation::new(o1), BoxedOperation::new(o2))
    }
}

pub trait DDatagram: std::fmt::Debug {
    fn dyn_operation_generator(
        &mut self,
        geometry: &Geometry,
    ) -> Result<DynDOperationGenerator, AUTDDriverError>;
    #[must_use]
    fn dyn_option(&self) -> DatagramOption;
    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<
    E,
    #[cfg(feature = "lightweight")] G: DOperationGenerator + Send + 'static,
    #[cfg(not(feature = "lightweight"))] G: DOperationGenerator + 'static,
    T: Datagram<G = G, Error = E>,
> DDatagram for MaybeUninit<T>
where
    AUTDDriverError: From<E>,
{
    fn dyn_operation_generator(
        &mut self,
        geometry: &Geometry,
    ) -> Result<DynDOperationGenerator, AUTDDriverError> {
        let mut tmp = MaybeUninit::<T>::uninit();
        std::mem::swap(&mut tmp, self);
        let d = unsafe { tmp.assume_init() };
        Ok(Box::new(d.operation_generator(geometry)?))
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

#[cfg(feature = "lightweight")]
unsafe impl Send for BoxedDatagram {}
#[cfg(feature = "lightweight")]
unsafe impl Sync for BoxedDatagram {}

impl std::fmt::Debug for BoxedDatagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.d.dyn_fmt(f)
    }
}

impl BoxedDatagram {
    fn new<
        E,
        #[cfg(feature = "lightweight")] G: OperationGenerator + Send + 'static,
        #[cfg(not(feature = "lightweight"))] G: OperationGenerator + 'static,
        D: Datagram<G = G, Error = E> + 'static,
    >(
        d: D,
    ) -> Self
    where
        AUTDDriverError: From<E>,
        AUTDDriverError: From<<G::O1 as Operation>::Error> + From<<G::O2 as Operation>::Error>,
    {
        BoxedDatagram {
            d: Box::new(MaybeUninit::new(d)),
        }
    }
}

impl Datagram for BoxedDatagram {
    type G = DynOperationGenerator;
    type Error = AUTDDriverError;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, Self::Error> {
        let Self { mut d } = self;
        Ok(DynOperationGenerator {
            g: d.dyn_operation_generator(geometry)?,
        })
    }

    fn option(&self) -> DatagramOption {
        self.d.dyn_option()
    }
}

/// Trait to convert [`Datagram`] to [`BoxedDatagram`].
pub trait IntoBoxedDatagram {
    /// Convert [`Datagram`] to [`BoxedDatagram`]
    #[must_use]
    fn into_boxed(self) -> BoxedDatagram;
}

impl<
    E,
    #[cfg(feature = "lightweight")] G: OperationGenerator + Send + 'static,
    #[cfg(not(feature = "lightweight"))] G: OperationGenerator + 'static,
    #[cfg(feature = "lightweight")] D: Datagram<Error = E, G = G> + Send + 'static,
    #[cfg(not(feature = "lightweight"))] D: Datagram<Error = E, G = G> + 'static,
> IntoBoxedDatagram for D
where
    AUTDDriverError: From<E>,
    AUTDDriverError: From<<G::O1 as Operation>::Error> + From<<G::O2 as Operation>::Error>,
{
    fn into_boxed(self) -> BoxedDatagram {
        BoxedDatagram::new(self)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // GRCOV_EXCL_START
    struct TestDatagram;
    struct TestOperationGenerator;

    impl OperationGenerator for TestOperationGenerator {
        type O1 = autd3_core::datagram::NullOp;
        type O2 = autd3_core::datagram::NullOp;

        fn generate(&mut self, _device: &Device) -> (Self::O1, Self::O2) {
            unimplemented!()
        }
    }

    impl Datagram for TestDatagram {
        type G = TestOperationGenerator;
        type Error = AUTDDriverError;

        fn operation_generator(self, _geometry: &Geometry) -> Result<Self::G, Self::Error> {
            Ok(Self::G {})
        }
    }
    // GRCOV_EXCL_STOP

    impl std::fmt::Debug for TestDatagram {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "test")
        }
    }

    #[test]
    fn boxed_fmt() {
        let d = TestDatagram;
        let bd = BoxedDatagram::new(d);
        assert_eq!(format!("{:?}", bd), "test");
    }
}
