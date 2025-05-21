use std::convert::Infallible;

#[derive(Debug)]
pub(crate) struct NullDatagram;

pub(crate) struct NullOperationGenerator;

impl autd3_driver::firmware::operation::OperationGenerator for NullOperationGenerator {
    type O1 = autd3_core::datagram::NullOp;
    type O2 = autd3_core::datagram::NullOp;

    fn generate(&mut self, _: &autd3_core::derive::Device) -> (Self::O1, Self::O2) {
        (autd3_core::datagram::NullOp, autd3_core::datagram::NullOp)
    }
}

impl autd3_core::datagram::Datagram for NullDatagram {
    type G = NullOperationGenerator;
    type Error = Infallible;

    fn operation_generator(
        self,
        _: &autd3_core::geometry::Geometry,
    ) -> Result<Self::G, Self::Error> {
        Ok(NullOperationGenerator)
    }
}
