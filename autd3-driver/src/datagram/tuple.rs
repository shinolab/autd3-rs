use std::time::Duration;

use crate::{
    derive::Geometry,
    error::AUTDInternalError,
    firmware::operation::{NullOp, OperationGenerator},
    geometry::Device,
};

use super::Datagram;

pub struct CombinedOperationGenerator<O1, O2>
where
    O1: OperationGenerator,
    O2: OperationGenerator,
{
    o1: O1,
    o2: O2,
}

impl<O1, O2> OperationGenerator for CombinedOperationGenerator<O1, O2>
where
    O1: OperationGenerator,
    O2: OperationGenerator,
{
    type O1 = O1::O1;
    type O2 = O2::O1;

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        let (o1, _) = self.o1.generate(device);
        let (o2, _) = self.o2.generate(device);
        (o1, o2)
    }
}

impl<G1, G2, D1, D2> Datagram for (D1, D2)
where
    D1: Datagram<G = G1>,
    D2: Datagram<G = G2>,
    G1: OperationGenerator<O2 = NullOp>,
    G2: OperationGenerator<O2 = NullOp>,
{
    type G = CombinedOperationGenerator<D1::G, D2::G>;

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(CombinedOperationGenerator {
            o1: self.0.operation_generator(geometry)?,
            o2: self.1.operation_generator(geometry)?,
        })
    }

    fn timeout(&self) -> Option<Duration> {
        match (self.0.timeout(), self.1.timeout()) {
            (Some(t1), Some(t2)) => Some(t1.max(t2)),
            (a, b) => a.or(b),
        }
    }

    fn parallel_threshold(&self) -> Option<usize> {
        match (self.0.parallel_threshold(), self.1.parallel_threshold()) {
            (Some(t1), Some(t2)) => Some(t1.min(t2)),
            (a, b) => a.or(b),
        }
    }
}