use std::time::Duration;

use crate::{
    common::Phase,
    datagram::*,
    derive::{Device, Transducer},
    error::AUTDInternalError,
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

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const fn f(&self) -> &F {
        &self.f
    }
}

impl<F: Fn(&Device, &Transducer) -> Phase> Datagram for ConfigurePhaseFilter<F> {
    type O1 = crate::operation::ConfigurePhaseFilterOp<F>;
    type O2 = crate::operation::NullOp;

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

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn f(_: &Device, _: &Transducer) -> Phase {
        Phase::new(0)
    }

    #[test]
    fn test_phase_filter() {
        let datagram = ConfigurePhaseFilter::additive(f);
        assert_eq!(Some(Duration::from_millis(200)), datagram.timeout());
        let r = datagram.operation();
        assert!(r.is_ok());
        let _ = r.unwrap();
    }
}
