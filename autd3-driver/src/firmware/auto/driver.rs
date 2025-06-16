use autd3_core::{link::Link, sleep::Sleep};

use super::Version;
use crate::{
    error::AUTDDriverError,
    firmware::driver::{Driver, FixedSchedule, Sender, TimerStrategy},
};

use getset::CopyGetters;

/// A driver with firmware version auto-detection.
#[derive(CopyGetters)]
pub struct Auto {
    #[getset(get_copy = "pub")]
    /// The estimated firmware version.
    pub(crate) version: Version,
}

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T>
    for super::transmission::Sender<'a, L, S, T>
{
    fn initialize_devices(self) -> Result<(), crate::error::AUTDDriverError> {
        match self.version {
            Version::V10 => super::super::v10::V10
                .sender(
                    self.msg_id,
                    self.link,
                    self.geometry,
                    self.sent_flags,
                    self.rx,
                    self.option,
                    self.timer_strategy,
                )
                .initialize_devices(),
            Version::V11 => super::super::v11::V11
                .sender(
                    self.msg_id,
                    self.link,
                    self.geometry,
                    self.sent_flags,
                    self.rx,
                    self.option,
                    self.timer_strategy,
                )
                .initialize_devices(),
            Version::V12 => super::super::v12::V12
                .sender(
                    self.msg_id,
                    self.link,
                    self.geometry,
                    self.sent_flags,
                    self.rx,
                    self.option,
                    self.timer_strategy,
                )
                .initialize_devices(),
        }
    }

    fn firmware_version(
        self,
    ) -> Result<Vec<crate::firmware::version::FirmwareVersion>, crate::error::AUTDDriverError> {
        match self.version {
            Version::V10 => super::super::v10::V10
                .sender(
                    self.msg_id,
                    self.link,
                    self.geometry,
                    self.sent_flags,
                    self.rx,
                    self.option,
                    self.timer_strategy,
                )
                .firmware_version(),
            Version::V11 => super::super::v11::V11
                .sender(
                    self.msg_id,
                    self.link,
                    self.geometry,
                    self.sent_flags,
                    self.rx,
                    self.option,
                    self.timer_strategy,
                )
                .firmware_version(),
            Version::V12 => super::super::v12::V12
                .sender(
                    self.msg_id,
                    self.link,
                    self.geometry,
                    self.sent_flags,
                    self.rx,
                    self.option,
                    self.timer_strategy,
                )
                .firmware_version(),
        }
    }

    fn close(self) -> Result<(), crate::error::AUTDDriverError> {
        match self.version {
            Version::V10 => super::super::v10::V10
                .sender(
                    self.msg_id,
                    self.link,
                    self.geometry,
                    self.sent_flags,
                    self.rx,
                    self.option,
                    self.timer_strategy,
                )
                .close(),
            Version::V11 => super::super::v11::V11
                .sender(
                    self.msg_id,
                    self.link,
                    self.geometry,
                    self.sent_flags,
                    self.rx,
                    self.option,
                    self.timer_strategy,
                )
                .close(),
            Version::V12 => super::super::v12::V12
                .sender(
                    self.msg_id,
                    self.link,
                    self.geometry,
                    self.sent_flags,
                    self.rx,
                    self.option,
                    self.timer_strategy,
                )
                .close(),
        }
    }
}

impl Driver for Auto {
    type Sender<'a, L, S, T>
        = super::transmission::Sender<'a, L, S, T>
    where
        L: Link + 'a,
        S: Sleep,
        T: TimerStrategy<S>;
    type FPGAState = super::super::latest::fpga::FPGAState;

    fn new() -> Self {
        Self {
            version: Version::V12,
        }
    }

    fn detect_version<'a, L>(
        &mut self,
        msg_id: &'a mut autd3_core::link::MsgId,
        link: &'a mut L,
        geometry: &'a autd3_core::derive::Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [autd3_core::link::RxMessage],
    ) -> Result<(), AUTDDriverError>
    where
        L: autd3_core::link::Link + 'a,
    {
        let mut sender = self.sender(
            msg_id,
            link,
            geometry,
            sent_flags,
            rx,
            crate::firmware::driver::SenderOption {
                timeout: Some(std::time::Duration::from_secs(1)),
                ..Default::default()
            },
            FixedSchedule::default(),
        );
        let _ = sender.send(crate::datagram::ReadsFPGAState::new(|_| false));

        let version_list = sender.firmware_version()?;
        self.version = check_firmware_version(&version_list)?;

        Ok(())
    }

    fn sender<'a, L, S, T>(
        &self,
        msg_id: &'a mut autd3_core::link::MsgId,
        link: &'a mut L,
        geometry: &'a autd3_core::derive::Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [autd3_core::link::RxMessage],
        option: crate::firmware::driver::SenderOption,
        timer_strategy: T,
    ) -> Self::Sender<'a, L, S, T>
    where
        L: Link + 'a,
        S: Sleep,
        T: TimerStrategy<S>,
    {
        Self::Sender {
            msg_id,
            link,
            geometry,
            sent_flags,
            rx,
            option,
            timer_strategy,
            _phantom: std::marker::PhantomData,
            version: self.version,
            limits: self.firmware_limits(),
        }
    }

    fn firmware_limits(&self) -> autd3_core::derive::FirmwareLimits {
        match self.version {
            Version::V10 => super::super::v10::V10.firmware_limits(),
            Version::V11 => super::super::v11::V11.firmware_limits(),
            Version::V12 => super::super::v12::V12.firmware_limits(),
        }
    }
}

fn check_firmware_version(
    version_list: &[crate::firmware::version::FirmwareVersion],
) -> Result<Version, AUTDDriverError> {
    if version_list.is_empty() {
        return Err(AUTDDriverError::FirmwareVersionMismatch);
    }

    let version = version_list[0];
    if version_list
        .iter()
        .skip(1)
        .any(|v| v.cpu.major != version.cpu.major || v.fpga.major != version.fpga.major)
    {
        return Err(AUTDDriverError::FirmwareVersionMismatch);
    }

    match version.cpu.major.0 {
        0xA2..=0xA2 => Ok(Version::V10),
        0xA3..=0xA3 => Ok(Version::V11),
        0xA4..=0xA4 => Ok(Version::V12),
        _ => Err(AUTDDriverError::UnsupportedFirmware),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::firmware::version::{CPUVersion, FPGAVersion, FirmwareVersion, Major, Minor};

    #[rstest::rstest]
    #[case::v10(
        Ok(Version::V10),
        vec![FirmwareVersion {
            idx: 0,
            cpu: CPUVersion {
                major: Major(0xA2),
                minor: Minor(0x00),
            },
            fpga: FPGAVersion {
                major: Major(0xA2),
                minor: Minor(0x00),
                function_bits: 0,
            },
        }]
    )]
    #[case::v11(
        Ok(Version::V11),
        vec![FirmwareVersion {
            idx: 0,
            cpu: CPUVersion {
                major: Major(0xA3),
                minor: Minor(0x00),
            },
            fpga: FPGAVersion {
                major: Major(0xA3),
                minor: Minor(0x00),
                function_bits: 0,
            },
        }]
    )]
    #[case::v12(
        Ok(Version::V12),
        vec![FirmwareVersion {
            idx: 0,
            cpu: CPUVersion {
                major: Major(0xA4),
                minor: Minor(0x00),
            },
            fpga: FPGAVersion {
                major: Major(0xA4),
                minor: Minor(0x00),
                function_bits: 0,
            },
        }]
    )]
    #[case::empty(
        Err(AUTDDriverError::FirmwareVersionMismatch),
        vec![]
    )]
    #[case::mismatch(
        Err(AUTDDriverError::FirmwareVersionMismatch),
        vec![
            FirmwareVersion {
                idx: 0,
                cpu: CPUVersion {
                    major: Major(0xA2),
                    minor: Minor(0x00),
                },
                fpga: FPGAVersion {
                    major: Major(0xA2),
                    minor: Minor(0x00),
                    function_bits: 0,
                },
            },
            FirmwareVersion {
                idx: 1,
                cpu: CPUVersion {
                    major: Major(0xA3),
                    minor: Minor(0x00),
                },
                fpga: FPGAVersion {
                    major: Major(0xA3),
                    minor: Minor(0x00),
                    function_bits: 0,
                },
            }
        ]
    )]
    #[case::unsupported(
        Err(AUTDDriverError::UnsupportedFirmware),
        vec![FirmwareVersion {
            idx: 0,
            cpu: CPUVersion {
                major: Major(0xA5),
                minor: Minor(0x00),
            },
            fpga: FPGAVersion {
                major: Major(0xA5),
                minor: Minor(0x00),
                function_bits: 0,
            },
        }]
    )]
    #[test]
    fn check_firmware_version(
        #[case] expect: Result<Version, AUTDDriverError>,
        #[case] version_list: Vec<crate::firmware::version::FirmwareVersion>,
    ) {
        assert_eq!(expect, super::check_firmware_version(&version_list));
    }
}
