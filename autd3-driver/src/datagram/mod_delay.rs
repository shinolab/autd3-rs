use crate::{
    datagram::*,
    error::AUTDInternalError,
    geometry::{Device, Transducer},
};

/// Datagram to set modulation delay
pub struct ConfigureModDelay<F: Fn(&Device, &Transducer) -> u16> {
    f: F,
}

impl<F: Fn(&Device, &Transducer) -> u16> ConfigureModDelay<F> {
    pub const fn new(f: F) -> Self {
        Self { f }
    }

    pub const fn f(&self) -> &F {
        &self.f
    }
}

impl<F: Fn(&Device, &Transducer) -> u16> Datagram for ConfigureModDelay<F> {
    type O1 = crate::operation::ConfigureModDelayOp<F>;
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
    fn f(_dev: &Device, _tr: &Transducer) -> u16 {
        0
    }

    #[test]
    fn test_mod_delay_timeout() {
        let delay = ConfigureModDelay::new(f);
        assert_eq!(delay.timeout(), Some(Duration::from_millis(200)));
    }

    #[test]
    fn test_mod_delay_operation() {
        let delay = ConfigureModDelay::new(f);
        let r = delay.operation();
        assert!(r.is_ok());
    }
}
