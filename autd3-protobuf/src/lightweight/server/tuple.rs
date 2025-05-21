use autd3_core::{
    datagram::{Datagram, DatagramOption},
    geometry::Geometry,
};
use autd3_driver::{
    datagram::BoxedDatagram,
    error::AUTDDriverError,
    firmware::operation::{BoxedOperation, OperationGenerator},
    geometry::Device,
};

pub struct BoxedDatagramTuple {
    pub d1: BoxedDatagram,
    pub d2: BoxedDatagram,
}

impl std::fmt::Debug for BoxedDatagramTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {:?})", self.d1, self.d2)
    }
}

pub struct OperationGeneratorTuple {
    pub g1: <BoxedDatagram as Datagram>::G,
    pub g2: <BoxedDatagram as Datagram>::G,
}

impl OperationGenerator for OperationGeneratorTuple {
    type O1 = BoxedOperation;
    type O2 = BoxedOperation;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        match (self.g1.generate(device), self.g2.generate(device)) {
            (Some((o1, _)), Some((o2, _))) => {
                Some((BoxedOperation::new(o1), BoxedOperation::new(o2)))
            }
            _ => None,
        }
    }
}

impl Datagram for BoxedDatagramTuple {
    type G = OperationGeneratorTuple;
    type Error = AUTDDriverError;

    fn operation_generator(self, geometry: &mut Geometry) -> Result<Self::G, Self::Error> {
        Ok(OperationGeneratorTuple {
            g1: self.d1.operation_generator(geometry)?,
            g2: self.d2.operation_generator(geometry)?,
        })
    }

    fn option(&self) -> autd3_core::datagram::DatagramOption {
        DatagramOption {
            timeout: self.d1.option().timeout.max(self.d2.option().timeout),
            parallel_threshold: self
                .d1
                .option()
                .parallel_threshold
                .min(self.d2.option().parallel_threshold),
        }
    }
}
