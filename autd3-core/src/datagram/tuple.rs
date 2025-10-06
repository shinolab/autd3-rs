use crate::{environment::Environment, firmware::FirmwareLimits, geometry::Geometry};

use super::{Datagram, DatagramOption, DeviceMask};

#[derive(Debug, PartialEq)]
#[doc(hidden)]
pub struct CombinedOperationGenerator<O1, O2> {
    pub o1: O1,
    pub o2: O2,
}

#[derive(Debug, PartialEq)]
#[doc(hidden)]
pub enum CombinedError<E1, E2> {
    E1(E1),
    E2(E2),
}

// GRCOV_EXCL_START
impl<E1, E2> core::error::Error for CombinedError<E1, E2>
where
    E1: core::error::Error + core::fmt::Display + 'static,
    E2: core::error::Error + core::fmt::Display + 'static,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::E1(e) => Some(e),
            Self::E2(e) => Some(e),
        }
    }
}

impl<E1, E2> core::fmt::Display for CombinedError<E1, E2>
where
    E1: core::fmt::Display,
    E2: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::E1(e) => write!(f, "E1: {}", e),
            Self::E2(e) => write!(f, "E2: {}", e),
        }
    }
}
// GRCOV_EXCL_STOP

impl<'a, G1, G2, D1, D2, E1, E2> Datagram<'a> for (D1, D2)
where
    D1: Datagram<'a, G = G1, Error = E1>,
    D2: Datagram<'a, G = G2, Error = E2>,
{
    type G = CombinedOperationGenerator<D1::G, D2::G>;
    type Error = CombinedError<E1, E2>;

    fn operation_generator(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
        limits: &FirmwareLimits,
    ) -> Result<Self::G, Self::Error> {
        match (
            self.0.operation_generator(geometry, env, filter, limits),
            self.1.operation_generator(geometry, env, filter, limits),
        ) {
            (Ok(g1), Ok(g2)) => Ok(CombinedOperationGenerator { o1: g1, o2: g2 }),
            (Err(e1), _) => Err(Self::Error::E1(e1)),
            (_, Err(e2)) => Err(Self::Error::E2(e2)),
        }
    }

    fn option(&self) -> DatagramOption {
        self.0.option().merge(self.1.option())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::time::Duration;

    #[derive(Debug)]
    pub struct TestDatagram {
        pub option: DatagramOption,
        pub result: Result<(), ()>,
    }

    impl Datagram<'_> for TestDatagram {
        type G = ();
        type Error = ();

        fn operation_generator(
            self,
            _: &Geometry,
            _: &Environment,
            _: &DeviceMask,
            _: &FirmwareLimits,
        ) -> Result<Self::G, Self::Error> {
            self.result
        }

        fn option(&self) -> DatagramOption {
            self.option
        }
    }

    #[rstest::rstest]
    #[case(Ok(CombinedOperationGenerator { o1: (), o2: () }), Ok(()), Ok(()))]
    #[case(Err(CombinedError::E1(())), Err(()), Ok(()))]
    #[case(Err(CombinedError::E2(())), Ok(()), Err(()))]
    #[test]
    fn operation_generator(
        #[case] expect: Result<CombinedOperationGenerator<(), ()>, CombinedError<(), ()>>,
        #[case] result1: Result<(), ()>,
        #[case] result2: Result<(), ()>,
    ) {
        assert_eq!(
            expect,
            (
                TestDatagram {
                    option: DatagramOption::default(),
                    result: result1,
                },
                TestDatagram {
                    option: DatagramOption::default(),
                    result: result2,
                }
            )
                .operation_generator(
                    &Geometry::new(Default::default()),
                    &Environment::new(),
                    &DeviceMask::AllEnabled,
                    &FirmwareLimits::unused()
                )
        );
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
                TestDatagram {
                    option: DatagramOption {
                        timeout: timeout1,
                        parallel_threshold: 0,
                    },
                    result: Ok(()),
                },
                TestDatagram {
                    option: DatagramOption {
                        timeout: timeout2,
                        parallel_threshold: 0,
                    },
                    result: Ok(()),
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
                TestDatagram {
                    option: DatagramOption {
                        timeout: Duration::ZERO,
                        parallel_threshold: threshold1,
                    },
                    result: Ok(()),
                },
                TestDatagram {
                    option: DatagramOption {
                        timeout: Duration::ZERO,
                        parallel_threshold: threshold2,
                    },
                    result: Ok(()),
                }
            )
                .option()
                .parallel_threshold
        );
    }
}
