use crate::{
    datagram::*,
    derive::{Device, Transducer},
    firmware::fpga::Phase,
};

#[derive(Debug, Clone, Copy)]
pub struct ConfigurePhaseFilter<F: Fn(&Device, &Transducer) -> Phase> {
    f: F,
}

impl<F: Fn(&Device, &Transducer) -> Phase> ConfigurePhaseFilter<F> {
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

impl<F: Fn(&Device, &Transducer) -> Phase> Datagram for ConfigurePhaseFilter<F> {
    type O1 = crate::firmware::operation::ConfigurePhaseFilterOp<F>;
    type O2 = crate::firmware::operation::NullOp;

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new(self.f), Self::O2::default()))
    }

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // GRCOV_EXCL_START
    fn f(_: &Device, _: &Transducer) -> Phase {
        Phase::new(0)
    }
    // GRCOV_EXCL_STOP

    #[test]
    fn test() {
        let datagram = ConfigurePhaseFilter::additive(f);
        assert_eq!(Some(Duration::from_millis(200)), datagram.timeout());
        let r = datagram.operation();
        assert!(r.is_ok());
        let _ = r.unwrap();
    }
}
