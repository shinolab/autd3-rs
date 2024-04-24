use autd3_driver::ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE};

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
        2 => autd3_driver::firmware::fpga::TransitionMode::GPIO,
        3 => autd3_driver::firmware::fpga::TransitionMode::Ext,
        _ => unreachable!(),
    })
}
