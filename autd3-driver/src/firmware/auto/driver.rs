use autd3_core::{environment::Environment, link::Link, sleep::Sleep};

use crate::{
    error::AUTDDriverError,
    firmware::driver::{Driver, FixedSchedule, Sender, TimerStrategy, Version},
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
        match self.inner {
            super::transmission::Inner::V10(inner) => inner.initialize_devices(),
            super::transmission::Inner::V11(inner) => inner.inner.initialize_devices(),
            super::transmission::Inner::V12(inner) => inner.initialize_devices(),
            super::transmission::Inner::V12_1(inner) => inner.inner.initialize_devices(),
        }
    }

    fn firmware_version(
        self,
    ) -> Result<Vec<crate::firmware::version::FirmwareVersion>, crate::error::AUTDDriverError> {
        match self.inner {
            super::transmission::Inner::V10(inner) => inner.firmware_version(),
            super::transmission::Inner::V11(inner) => inner.inner.firmware_version(),
            super::transmission::Inner::V12(inner) => inner.firmware_version(),
            super::transmission::Inner::V12_1(inner) => inner.inner.firmware_version(),
        }
    }

    fn close(self) -> Result<(), crate::error::AUTDDriverError> {
        match self.inner {
            super::transmission::Inner::V10(inner) => inner.close(),
            super::transmission::Inner::V11(inner) => inner.inner.close(),
            super::transmission::Inner::V12(inner) => inner.close(),
            super::transmission::Inner::V12_1(inner) => inner.inner.close(),
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
    type FPGAState = super::super::v12_1::fpga::FPGAState;

    fn new() -> Self {
        Self {
            version: Version::V12_1,
        }
    }

    fn detect_version<'a, L>(
        &mut self,
        msg_id: &'a mut autd3_core::link::MsgId,
        link: &'a mut L,
        geometry: &'a autd3_core::geometry::Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [autd3_core::link::RxMessage],
        env: &'a Environment,
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
            env,
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
        geometry: &'a autd3_core::geometry::Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [autd3_core::link::RxMessage],
        env: &'a Environment,
        option: crate::firmware::driver::SenderOption,
        timer_strategy: T,
    ) -> Self::Sender<'a, L, S, T>
    where
        L: Link + 'a,
        S: Sleep,
        T: TimerStrategy<S>,
    {
        Self::Sender {
            inner: match self.version {
                Version::V10 => {
                    super::transmission::Inner::V10(crate::firmware::v10::transmission::Sender {
                        msg_id,
                        link,
                        geometry,
                        sent_flags,
                        rx,
                        env,
                        option,
                        timer_strategy,
                        _phantom: std::marker::PhantomData,
                    })
                }
                Version::V11 => {
                    super::transmission::Inner::V11(crate::firmware::v11::transmission::Sender {
                        inner: crate::firmware::v10::transmission::Sender {
                            msg_id,
                            link,
                            geometry,
                            sent_flags,
                            rx,
                            env,
                            option,
                            timer_strategy,
                            _phantom: std::marker::PhantomData,
                        },
                    })
                }
                Version::V12 => {
                    super::transmission::Inner::V12(crate::firmware::v12::transmission::Sender {
                        msg_id,
                        link,
                        geometry,
                        sent_flags,
                        rx,
                        env,
                        option,
                        timer_strategy,
                        _phantom: std::marker::PhantomData,
                    })
                }
                Version::V12_1 => super::transmission::Inner::V12_1(
                    crate::firmware::v12_1::transmission::Sender {
                        inner: crate::firmware::v12::transmission::Sender {
                            msg_id,
                            link,
                            geometry,
                            sent_flags,
                            rx,
                            env,
                            option,
                            timer_strategy,
                            _phantom: std::marker::PhantomData,
                        },
                    },
                ),
            },
            version: self.version,
            limits: self.firmware_limits(),
        }
    }

    fn firmware_limits(&self) -> autd3_core::firmware::FirmwareLimits {
        match self.version {
            Version::V10 => super::super::v10::V10.firmware_limits(),
            Version::V11 => super::super::v11::V11.firmware_limits(),
            Version::V12 => super::super::v12::V12.firmware_limits(),
            Version::V12_1 => super::super::v12_1::V12_1.firmware_limits(),
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
        0xA5..=0xA5 => Ok(Version::V12_1),
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
    #[case::v12(
        Ok(Version::V12_1),
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
                major: Major(0xFF),
                minor: Minor(0x00),
            },
            fpga: FPGAVersion {
                major: Major(0xFF),
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
