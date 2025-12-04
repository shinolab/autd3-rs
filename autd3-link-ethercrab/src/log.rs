#![allow(clippy::single_component_path_imports)]

#[macro_use]
mod _log {
    macro_rules! info {
        ($($arg:tt)+) => {
            #[cfg(feature = "tracing")]
            tracing::info!($($arg)+);
        };
    }

    macro_rules! warn_ {
        ($($arg:tt)+) => {
            #[cfg(feature = "tracing")]
            tracing::warn!($($arg)+);
        };
    }

    macro_rules! error {
        ($($arg:tt)+) => {
            #[cfg(feature = "tracing")]
            tracing::error!($($arg)+);
        };
    }

    macro_rules! debug {
        ($($arg:tt)+) => {
            #[cfg(feature = "tracing")]
            tracing::debug!($($arg)+);
        };
    }

    macro_rules! trace {
        ($($arg:tt)+) => {
            #[cfg(feature = "tracing")]
            tracing::trace!($($arg)+);
        };
    }
}

pub(crate) use debug;
pub(crate) use error;
pub(crate) use info;
pub(crate) use trace;
pub(crate) use warn_ as warn;
