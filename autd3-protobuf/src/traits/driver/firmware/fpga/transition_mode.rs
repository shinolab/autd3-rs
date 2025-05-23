use autd3_driver::ethercat::DcSysTime;

use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl From<autd3_driver::firmware::fpga::TransitionMode> for TransitionMode {
    fn from(value: autd3_driver::firmware::fpga::TransitionMode) -> Self {
        Self {
            mode: Some(match value {
                autd3_driver::firmware::fpga::TransitionMode::SyncIdx => {
                    transition_mode::Mode::SyncIdx(transition_mode::SyncIdx {})
                }
                autd3_driver::firmware::fpga::TransitionMode::SysTime(value) => {
                    transition_mode::Mode::SysTime(transition_mode::SysTime {
                        value: value.sys_time(),
                    })
                }
                autd3_driver::firmware::fpga::TransitionMode::GPIO(value) => {
                    transition_mode::Mode::Gpio(transition_mode::Gpio { value: value as _ })
                }
                autd3_driver::firmware::fpga::TransitionMode::Ext => {
                    transition_mode::Mode::Ext(transition_mode::Ext {})
                }
                autd3_driver::firmware::fpga::TransitionMode::Immediate => {
                    transition_mode::Mode::Immediate(transition_mode::Immediate {})
                }
            }),
        }
    }
}

impl FromMessage<TransitionMode> for autd3_driver::firmware::fpga::TransitionMode {
    fn from_msg(msg: TransitionMode) -> Result<Self, AUTDProtoBufError> {
        Ok(match msg.mode.ok_or(AUTDProtoBufError::DataParseError)? {
            transition_mode::Mode::SyncIdx(transition_mode::SyncIdx {}) => {
                autd3_driver::firmware::fpga::TransitionMode::SyncIdx
            }
            transition_mode::Mode::SysTime(transition_mode::SysTime { value }) => {
                autd3_driver::firmware::fpga::TransitionMode::SysTime(
                    DcSysTime::ZERO + std::time::Duration::from_nanos(value),
                )
            }
            transition_mode::Mode::Gpio(transition_mode::Gpio { value }) => {
                autd3_driver::firmware::fpga::TransitionMode::GPIO(
                    autd3_driver::firmware::fpga::GPIOIn::from(GpioIn::try_from(value)?),
                )
            }
            transition_mode::Mode::Ext(transition_mode::Ext {}) => {
                autd3_driver::firmware::fpga::TransitionMode::Ext
            }
            transition_mode::Mode::Immediate(transition_mode::Immediate {}) => {
                autd3_driver::firmware::fpga::TransitionMode::Immediate
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(autd3_driver::firmware::fpga::TransitionMode::SyncIdx)]
    #[case(
        autd3_driver::firmware::fpga::TransitionMode::SysTime(
            DcSysTime::ZERO
                + std::time::Duration::from_nanos(1)
        ),
    )]
    #[case(autd3_driver::firmware::fpga::TransitionMode::GPIO(
        autd3_driver::firmware::fpga::GPIOIn::I0
    ))]
    #[case(autd3_driver::firmware::fpga::TransitionMode::Ext)]
    fn test_transition_mode(#[case] expect: autd3_driver::firmware::fpga::TransitionMode) {
        let msg = expect.into();
        assert_eq!(
            expect,
            autd3_driver::firmware::fpga::TransitionMode::from_msg(msg).unwrap()
        );
    }
}
