use crate::{
    datagram::*,
    derive::DEFAULT_TIMEOUT,
    firmware::fpga::{DebugType, GPIOOut},
    geometry::Device,
};

/// Datagram for configure debug_output_idx
pub struct DebugSettings<F: Fn(&Device, GPIOOut) -> DebugType> {
    f: F,
}

impl<F: Fn(&Device, GPIOOut) -> DebugType> DebugSettings<F> {
    /// constructor
    pub const fn new(f: F) -> Self {
        Self { f }
    }

    // GRCOV_EXCL_START
    pub fn f(&self) -> &F {
        &self.f
    }
    // GRCOV_EXCL_STOP
}

impl<F: Fn(&Device, GPIOOut) -> DebugType> Datagram for DebugSettings<F> {
    type O1 = crate::firmware::operation::DebugSettingOp<F>;
    type O2 = crate::firmware::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation(self) -> (Self::O1, Self::O2) {
        (Self::O1::new(self.f), Self::O2::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::firmware::operation::{DebugSettingOp, NullOp};

    use super::*;

    // GRCOV_EXCL_START
    fn f(_: &Device, _: GPIOOut) -> DebugType {
        DebugType::None
    }
    // GRCOV_EXCL_STOP

    #[test]
    fn test_timeout() {
        let d = DebugSettings::new(f);
        let timeout = d.timeout();
        assert!(timeout.is_some());
        assert!(timeout.unwrap() > Duration::ZERO);
    }

    #[test]
    fn test_operation() {
        let d = DebugSettings::new(f);
        let _: (DebugSettingOp<_>, NullOp) = d.operation();
    }
}
