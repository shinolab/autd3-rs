use crate::{
    datagram::*,
    defined::DEFAULT_TIMEOUT,
    derive::{Device, Transducer},
    firmware::fpga::Phase,
};

#[derive(Debug, Clone, Copy)]
pub struct ConfigurePhaseFilter<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> {
    f: F,
}

impl<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> ConfigurePhaseFilter<FT, F> {
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

impl<FT: Fn(&Transducer) -> Phase, F: Fn(&Device) -> FT> Datagram for ConfigurePhaseFilter<FT, F> {
    type O1 = crate::firmware::operation::ConfigurePhaseFilterOp<FT, F>;
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
        let datagram = ConfigurePhaseFilter::additive(f);
        assert_eq!(Some(DEFAULT_TIMEOUT), datagram.timeout());
        let _ = datagram.operation();
    }
}
