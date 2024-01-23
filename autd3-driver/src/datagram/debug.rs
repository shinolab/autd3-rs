use std::time::Duration;

use crate::{
    datagram::*,
    error::AUTDInternalError,
    geometry::{Device, Transducer},
};

/// Datagram for configure debug_output_idx
pub struct ConfigureDebugOutputIdx<F: Fn(&Device) -> Option<&Transducer>> {
    f: F,
}

impl<F: Fn(&Device) -> Option<&Transducer>> ConfigureDebugOutputIdx<F> {
    /// constructor
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Fn(&Device) -> Option<&Transducer>> Datagram for ConfigureDebugOutputIdx<F> {
    type O1 = crate::operation::DebugOutIdxOp<F>;
    type O2 = crate::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new(self.f), Self::O2::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn f(dev: &Device) -> Option<&Transducer> {
        Some(&dev[0])
    }

    #[test]
    fn test_debug_output_idx_timeout() {
        let debug_output_idx = ConfigureDebugOutputIdx::new(f);
        let timeout = debug_output_idx.timeout();
        assert!(timeout.is_some());
        assert!(timeout.unwrap() > Duration::ZERO);
    }

    #[test]
    fn test_debug_output_idx_operation() {
        let debug_output_idx = ConfigureDebugOutputIdx::new(f);
        let r = debug_output_idx.operation();
        assert!(r.is_ok());
        let _ = r.unwrap();
    }
}
