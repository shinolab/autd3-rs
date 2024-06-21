use autd3_driver::{
    ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE},
    firmware::fpga::GPIOIn,
};

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::firmware::fpga::TransitionMode {
    type Message = TransitionMode;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            mode: Some(match *self {
                autd3::derive::TransitionMode::SyncIdx => {
                    transition_mode::Mode::SyncIdx(TransitionModeSyncIdx {})
                }
                autd3::derive::TransitionMode::SysTime(value) => {
                    transition_mode::Mode::SysTime(TransitionModeSysTime {
                        value: value.sys_time(),
                    })
                }
                autd3::derive::TransitionMode::GPIO(value) => {
                    transition_mode::Mode::Gpio(TransitionModeGpio { value: value as _ })
                }
                autd3::derive::TransitionMode::Ext => {
                    transition_mode::Mode::Ext(TransitionModeExt {})
                }
                autd3::derive::TransitionMode::Immediate => {
                    transition_mode::Mode::Immediate(TransitionModeImmediate {})
                }
                _ => unimplemented!(),
            }),
        }
    }
}

impl FromMessage<TransitionMode> for autd3_driver::firmware::fpga::TransitionMode {
    fn from_msg(msg: &TransitionMode) -> Result<Self, AUTDProtoBufError> {
        let mode = msg.mode.as_ref().ok_or(AUTDProtoBufError::DataParseError)?;
        Ok(match *mode {
            transition_mode::Mode::SyncIdx(TransitionModeSyncIdx {}) => {
                autd3_driver::firmware::fpga::TransitionMode::SyncIdx
            }
            transition_mode::Mode::SysTime(TransitionModeSysTime { value }) => {
                autd3_driver::firmware::fpga::TransitionMode::SysTime(
                    DcSysTime::from_utc(
                        ECAT_DC_SYS_TIME_BASE + std::time::Duration::from_nanos(value),
                    )
                    .unwrap(),
                )
            }
            transition_mode::Mode::Gpio(TransitionModeGpio { value }) => {
                autd3_driver::firmware::fpga::TransitionMode::GPIO(
                    autd3_driver::firmware::fpga::GPIOIn::from(match value {
                        0 => GPIOIn::I0,
                        1 => GPIOIn::I1,
                        2 => GPIOIn::I2,
                        3 => GPIOIn::I3,
                        _ => unimplemented!(),
                    }),
                )
            }
            transition_mode::Mode::Ext(TransitionModeExt {}) => {
                autd3_driver::firmware::fpga::TransitionMode::Ext
            }
            transition_mode::Mode::Immediate(TransitionModeImmediate {}) => {
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
            DcSysTime::from_utc(ECAT_DC_SYS_TIME_BASE).unwrap()
                + std::time::Duration::from_nanos(1)
        ),
    )]
    #[case(autd3_driver::firmware::fpga::TransitionMode::GPIO(
        autd3_driver::firmware::fpga::GPIOIn::I0
    ))]
    #[case(autd3_driver::firmware::fpga::TransitionMode::Ext)]
    fn test_transition_mode(#[case] expect: autd3_driver::firmware::fpga::TransitionMode) {
        let msg = expect.to_msg(None);
        assert_eq!(
            expect,
            autd3_driver::firmware::fpga::TransitionMode::from_msg(&msg).unwrap()
        );
    }
}
