use crate::{datagram::*, geometry::Device, operation::DebugType};

/// Datagram for configure debug_output_idx
pub struct ConfigureDebugSettings<F: Fn(&Device) -> [DebugType; 4]> {
    f: F,
}

impl<F: Fn(&Device) -> [DebugType; 4]> ConfigureDebugSettings<F> {
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

impl<F: Fn(&Device) -> [DebugType; 4]> Datagram for ConfigureDebugSettings<F> {
    type O1 = crate::operation::DebugSettingOp<F>;
    type O2 = crate::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new(self.f), Self::O2::default()))
    }
}

mod old {
    #![allow(deprecated)]

    use crate::derive::Transducer;

    use super::*;

    /// Datagram for configure debug_output_idx
    #[deprecated(note = "Use DebugSettingOp instead", since = "22.1.0")]
    pub struct ConfigureDebugOutputIdx<F: Fn(&Device) -> Option<&Transducer>> {
        f: F,
    }

    impl<F: Fn(&Device) -> Option<&Transducer>> ConfigureDebugOutputIdx<F> {
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
}

#[allow(deprecated)]
pub use old::ConfigureDebugOutputIdx;
