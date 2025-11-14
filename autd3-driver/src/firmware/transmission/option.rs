use std::time::Duration;

use super::parallel_mode::ParallelMode;

/// The option used in [`Sender`].
///
/// [`Sender`]: super::Sender
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SenderOption {
    /// The duration between sending operations.
    pub send_interval: Option<Duration>,
    /// The duration between receiving operations.
    pub receive_interval: Option<Duration>,
    /// Timeout for data transmission check for each frame. If `None`, [`Datagram::option`] is used.
    ///
    /// [`Datagram::option`]: autd3_core::datagram::Datagram::option
    pub timeout: Option<Duration>,
    /// The parallel processing mode.
    pub parallel: ParallelMode,
}

impl Default for SenderOption {
    fn default() -> Self {
        Self {
            send_interval: Some(Duration::from_millis(1)),
            receive_interval: Some(Duration::from_millis(1)),
            timeout: None,
            parallel: ParallelMode::Auto,
        }
    }
}
