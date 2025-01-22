use thiserror::Error;

use crate::geometry::Geometry;

use super::{Datagram, DatagramOption};

#[doc(hidden)]
pub struct CombinedOperationGenerator<O1, O2> {
    pub o1: O1,
    pub o2: O2,
}

#[derive(Error, Debug, PartialEq)]
#[doc(hidden)]
pub enum CombinedError<E1, E2>
where
    E1: std::error::Error,
    E2: std::error::Error,
{
    #[error("{0}")]
    E1(E1),
    #[error("{0}")]
    E2(E2),
}

impl<G1, G2, D1, D2, E1, E2> Datagram for (D1, D2)
where
    D1: Datagram<G = G1, Error = E1>,
    D2: Datagram<G = G2, Error = E2>,
    E1: std::error::Error,
    E2: std::error::Error,
{
    type G = CombinedOperationGenerator<D1::G, D2::G>;
    type Error = CombinedError<E1, E2>;

    fn operation_generator(
        self,
        geometry: &Geometry,
        option: &DatagramOption,
    ) -> Result<Self::G, Self::Error> {
        match (
            self.0.operation_generator(geometry, option),
            self.1.operation_generator(geometry, option),
        ) {
            (Ok(g1), Ok(g2)) => Ok(CombinedOperationGenerator { o1: g1, o2: g2 }),
            (Err(e1), _) => Err(Self::Error::E1(e1)),
            (_, Err(e2)) => Err(Self::Error::E2(e2)),
        }
    }

    fn option(&self) -> DatagramOption {
        DatagramOption {
            timeout: self.0.option().timeout.max(self.1.option().timeout),
            parallel_threshold: self
                .0
                .option()
                .parallel_threshold
                .min(self.1.option().parallel_threshold),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    #[derive(Debug)]
    pub struct NullDatagram {
        pub option: DatagramOption,
    }

    impl Datagram for NullDatagram {
        type G = ();
        type Error = std::convert::Infallible;

        fn operation_generator(
            self,
            _: &Geometry,
            _: &DatagramOption,
        ) -> Result<Self::G, Self::Error> {
            Ok(())
        }

        fn option(&self) -> DatagramOption {
            self.option
        }
    }

    #[rstest::rstest]
    #[case(
        Duration::from_millis(200),
        Duration::from_millis(100),
        Duration::from_millis(200)
    )]
    #[case(
        Duration::from_millis(200),
        Duration::from_millis(200),
        Duration::from_millis(100)
    )]
    #[test]
    fn timeout(#[case] expect: Duration, #[case] timeout1: Duration, #[case] timeout2: Duration) {
        assert_eq!(
            expect,
            (
                NullDatagram {
                    option: DatagramOption {
                        timeout: timeout1,
                        parallel_threshold: 0,
                    },
                },
                NullDatagram {
                    option: DatagramOption {
                        timeout: timeout2,
                        parallel_threshold: 0,
                    },
                }
            )
                .option()
                .timeout
        );
    }

    #[rstest::rstest]
    #[case(100, 100, 200)]
    #[case(100, 200, 100)]
    #[test]
    fn parallel_threshold(
        #[case] expect: usize,
        #[case] threshold1: usize,
        #[case] threshold2: usize,
    ) {
        assert_eq!(
            expect,
            (
                NullDatagram {
                    option: DatagramOption {
                        timeout: Duration::ZERO,
                        parallel_threshold: threshold1,
                    },
                },
                NullDatagram {
                    option: DatagramOption {
                        timeout: Duration::ZERO,
                        parallel_threshold: threshold2,
                    },
                }
            )
                .option()
                .parallel_threshold
        );
    }
}
