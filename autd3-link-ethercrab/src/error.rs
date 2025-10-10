use std::time::Duration;

use autd3_core::link::LinkError;

#[derive(Debug)]
pub enum EtherCrabError {
    PduStorageError,
    DeviceNotFound,
    NotResponding,
    IoError(std::io::Error),
    EtherCrab(ethercrab::error::Error),
    DeviceNumberMismatch(usize, usize),
    SyncTimeout(Duration),
    Pcap(pcap::Error),
}

impl std::fmt::Display for EtherCrabError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EtherCrabError::PduStorageError => write!(f, "Can only split once"),
            EtherCrabError::DeviceNotFound => write!(f, "No AUTD3 device found"),
            EtherCrabError::NotResponding => write!(f, "One ore more devices are not responding"),
            EtherCrabError::IoError(e) => write!(f, "{}", e),
            EtherCrabError::EtherCrab(e) => write!(f, "{}", e),
            EtherCrabError::DeviceNumberMismatch(expected, found) => write!(
                f,
                "Number of devices specified ({}) does not match the number found ({})",
                expected, found
            ),
            EtherCrabError::SyncTimeout(max_deviation) => {
                write!(
                    f,
                    "Failed to synchronize devices (Max deviation: {:?})",
                    max_deviation
                )
            }
            EtherCrabError::Pcap(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for EtherCrabError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EtherCrabError::IoError(e) => Some(e),
            EtherCrabError::EtherCrab(e) => Some(e),
            EtherCrabError::Pcap(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for EtherCrabError {
    fn from(e: std::io::Error) -> Self {
        EtherCrabError::IoError(e)
    }
}

impl From<ethercrab::error::Error> for EtherCrabError {
    fn from(e: ethercrab::error::Error) -> Self {
        EtherCrabError::EtherCrab(e)
    }
}

impl From<pcap::Error> for EtherCrabError {
    fn from(e: pcap::Error) -> Self {
        EtherCrabError::Pcap(e)
    }
}

impl From<EtherCrabError> for LinkError {
    fn from(val: EtherCrabError) -> LinkError {
        LinkError::new(val.to_string())
    }
}
