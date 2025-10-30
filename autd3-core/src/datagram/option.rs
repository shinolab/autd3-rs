use core::time::Duration;

use crate::common::DEFAULT_TIMEOUT;

/// The option of the [`Datagram`].
///
/// [`Datagram`]: super::Datagram
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatagramOption {
    /// The default timeout of the datagram.
    pub timeout: Duration,
    /// The default threshold of the parallel processing.
    pub parallel_threshold: usize,
}

impl Default for DatagramOption {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            parallel_threshold: usize::MAX,
        }
    }
}

impl DatagramOption {
    /// Merges two [`DatagramOption`]s.
    pub fn merge(self, other: DatagramOption) -> Self {
        Self {
            timeout: self.timeout.max(other.timeout),
            parallel_threshold: self.parallel_threshold.min(other.parallel_threshold),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datagram_option_merge() {
        let opt1 = DatagramOption {
            timeout: Duration::from_secs(1),
            parallel_threshold: 10,
        };
        let opt2 = DatagramOption {
            timeout: Duration::from_secs(2),
            parallel_threshold: 5,
        };
        let opt3 = opt1.merge(opt2);
        assert_eq!(opt3.timeout, Duration::from_secs(2));
        assert_eq!(opt3.parallel_threshold, 5);
    }
}
