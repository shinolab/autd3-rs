use std::time::Duration;

use crate::{
    error::AUTDInternalError,
    firmware::operation::{NullOp, OperationGenerator},
    geometry::{Device, Geometry},
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

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
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
        self.0.timeout().into_iter().chain(self.1.timeout()).max()
    }

    fn parallel_threshold(&self) -> Option<usize> {
        self.0
            .parallel_threshold()
            .into_iter()
            .chain(self.1.parallel_threshold())
            .min()
    }
}

#[cfg(test)]
mod tests {
    use crate::datagram::tests::NullDatagram;

    use super::*;

    #[rstest::rstest]
    #[case(None, None, None)]
    #[case(
        Some(Duration::from_millis(100)),
        Some(Duration::from_millis(100)),
        None
    )]
    #[case(
        Some(Duration::from_millis(100)),
        None,
        Some(Duration::from_millis(100))
    )]
    #[case(
        Some(Duration::from_millis(200)),
        Some(Duration::from_millis(100)),
        Some(Duration::from_millis(200))
    )]
    #[case(
        Some(Duration::from_millis(200)),
        Some(Duration::from_millis(200)),
        Some(Duration::from_millis(100))
    )]
    #[test]
    fn timeout(
        #[case] expect: Option<Duration>,
        #[case] timeout1: Option<Duration>,
        #[case] timeout2: Option<Duration>,
    ) {
        assert_eq!(
            expect,
            (
                NullDatagram {
                    timeout: timeout1,
                    parallel_threshold: None,
                },
                NullDatagram {
                    timeout: timeout2,
                    parallel_threshold: None,
                }
            )
                .timeout()
        );
    }

    #[rstest::rstest]
    #[case(None, None, None)]
    #[case(Some(100), Some(100), None)]
    #[case(Some(100), None, Some(100))]
    #[case(Some(100), Some(100), Some(200))]
    #[case(Some(100), Some(200), Some(100))]
    #[test]
    fn parallel_threshold(
        #[case] expect: Option<usize>,
        #[case] threshold1: Option<usize>,
        #[case] threshold2: Option<usize>,
    ) {
        assert_eq!(
            expect,
            (
                NullDatagram {
                    timeout: None,
                    parallel_threshold: threshold1,
                },
                NullDatagram {
                    timeout: None,
                    parallel_threshold: threshold2,
                }
            )
                .parallel_threshold()
        );
    }
}
