use crate::{
    datagram::*,
    defined::DEFAULT_TIMEOUT,
    derive::{Device, Transducer},
    firmware::fpga::Phase,
};

#[derive(Debug, Clone, Copy)]
pub struct PhaseFilter<P: Into<Phase>, FT: Fn(&Transducer) -> P, F: Fn(&Device) -> FT> {
    f: F,
}

impl<P: Into<Phase>, FT: Fn(&Transducer) -> P, F: Fn(&Device) -> FT> PhaseFilter<P, FT, F> {
    /// constructor
    pub const fn additive(f: F) -> Self {
        Self { f }
    }

    // GRCOV_EXCL_START
    pub const fn f(&self) -> &F {
        &self.f
    }
    // GRCOV_EXCL_STOP
}

impl<P: Into<Phase>, FT: Fn(&Transducer) -> P, F: Fn(&Device) -> FT> Datagram
    for PhaseFilter<P, FT, F>
{
    type O1 = crate::firmware::operation::PhaseFilterOp<P, FT, F>;
    type O2 = crate::firmware::operation::NullOp;

    fn operation(self) -> (Self::O1, Self::O2) {
        (Self::O1::new(self.f), Self::O2::default())
    }

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // GRCOV_EXCL_START
    fn f(_: &Device) -> impl Fn(&Transducer) -> Phase {
        |_| Phase::new(0)
    }
    // GRCOV_EXCL_STOP

    #[test]
    fn test() {
        let datagram = PhaseFilter::additive(f);
        assert_eq!(Some(DEFAULT_TIMEOUT), datagram.timeout());
        let _ = datagram.operation();
    }
}
