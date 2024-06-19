use autd3_driver::ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE};

// TODO: Add a Immediate variant
pub fn to_transition_mode(
    mode: Option<i32>,
    value: Option<u64>,
) -> Option<autd3_driver::firmware::fpga::TransitionMode> {
    mode.map(|mode| match mode {
        0 => autd3_driver::firmware::fpga::TransitionMode::SyncIdx,
        1 => autd3_driver::firmware::fpga::TransitionMode::SysTime(
            DcSysTime::from_utc(ECAT_DC_SYS_TIME_BASE).unwrap()
                + std::time::Duration::from_nanos(value.unwrap()),
        ),
        2 => autd3_driver::firmware::fpga::TransitionMode::GPIO(match value.unwrap() {
            0 => autd3_driver::firmware::fpga::GPIOIn::I0,
            1 => autd3_driver::firmware::fpga::GPIOIn::I1,
            2 => autd3_driver::firmware::fpga::GPIOIn::I2,
            3 => autd3_driver::firmware::fpga::GPIOIn::I3,
            _ => unreachable!(),
        }),
        3 => autd3_driver::firmware::fpga::TransitionMode::Ext,
        0xFF => autd3_driver::firmware::fpga::TransitionMode::Immediate,
        _ => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(
        Some(autd3_driver::firmware::fpga::TransitionMode::SyncIdx),
        Some(0),
        Some(0)
    )]
    #[case(
        Some(autd3_driver::firmware::fpga::TransitionMode::SysTime(
            DcSysTime::from_utc(ECAT_DC_SYS_TIME_BASE).unwrap()
                + std::time::Duration::from_nanos(1)
        )),
        Some(1),
        Some(1)
    )]
    #[case(
        Some(autd3_driver::firmware::fpga::TransitionMode::GPIO(
            autd3_driver::firmware::fpga::GPIOIn::I0
        )),
        Some(2),
        Some(0)
    )]
    #[case(
        Some(autd3_driver::firmware::fpga::TransitionMode::Ext),
        Some(3),
        Some(0)
    )]
    fn test_transition_mode(
        #[case] expect: Option<autd3_driver::firmware::fpga::TransitionMode>,
        #[case] mode: Option<i32>,
        #[case] value: Option<u64>,
    ) {
        assert_eq!(expect, to_transition_mode(mode, value));
    }
}
