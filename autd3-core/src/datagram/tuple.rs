use std::time::Duration;

use thiserror::Error;

use crate::geometry::Geometry;

use super::Datagram;

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

    fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, Self::Error> {
        match (
            self.0.operation_generator(geometry),
            self.1.operation_generator(geometry),
        ) {
            (Ok(g1), Ok(g2)) => Ok(CombinedOperationGenerator { o1: g1, o2: g2 }),
            (Err(e1), _) => Err(Self::Error::E1(e1)),
            (_, Err(e2)) => Err(Self::Error::E2(e2)),
        }
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
    use super::*;

    #[derive(Debug)]
    pub struct NullDatagram {
        pub timeout: Option<Duration>,
        pub parallel_threshold: Option<usize>,
    }

    impl Datagram for NullDatagram {
        type G = ();
        type Error = std::convert::Infallible;

        fn operation_generator(self, _: &Geometry) -> Result<Self::G, Self::Error> {
            Ok(())
        }

        fn timeout(&self) -> Option<Duration> {
            self.timeout
        }

        fn parallel_threshold(&self) -> Option<usize> {
            self.parallel_threshold
        }
    }

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
