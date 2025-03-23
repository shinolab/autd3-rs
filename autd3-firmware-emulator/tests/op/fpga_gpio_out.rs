use autd3_driver::{
    datagram::*,
    ethercat::DcSysTime,
    firmware::{
        cpu::TxMessage,
        fpga::{GPIOOut, GPIOOutputType},
    },
};
use autd3_firmware_emulator::{CPUEmulator, fpga::params::*};

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[rstest::rstest]
#[test]
#[case([DBG_NONE, DBG_BASE_SIG, DBG_THERMO, DBG_FORCE_FAN], [0, 0, 0, 0], [GPIOOutputType::None, GPIOOutputType::BaseSignal, GPIOOutputType::Thermo, GPIOOutputType::ForceFan])]
#[case([DBG_SYNC, DBG_MOD_SEGMENT, DBG_MOD_IDX, DBG_STM_SEGMENT], [0, 0, 0x01, 0], [GPIOOutputType::Sync, GPIOOutputType::ModSegment, GPIOOutputType::ModIdx(0x01), GPIOOutputType::StmSegment])]
#[case([DBG_STM_IDX, DBG_IS_STM_MODE, DBG_SYS_TIME_EQ, DBG_DIRECT], [0x02, 0, 2, 1], [GPIOOutputType::StmIdx(0x02), GPIOOutputType::IsStmMode, GPIOOutputType::SysTimeEq(DcSysTime::ZERO + 2 * std::time::Duration::from_micros(25)), GPIOOutputType::Direct(true)])]
fn send_debug_output_idx(
    #[case] expect_types: [u8; 4],
    #[case] expect_values: [u64; 4],
    #[case] debug_types: [GPIOOutputType<'static>; 4],
) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    let d = GPIOOutputs::new(|_, gpio| match gpio {
        GPIOOut::O0 => debug_types[0].clone(),
        GPIOOut::O1 => debug_types[1].clone(),
        GPIOOut::O2 => debug_types[2].clone(),
        GPIOOut::O3 => debug_types[3].clone(),
    });

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(expect_types, cpu.fpga().debug_types());
    assert_eq!(expect_values, cpu.fpga().debug_values());

    Ok(())
}

#[test]
fn send_debug_pwm_out() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    let d = GPIOOutputs::new(|dev, gpio| match gpio {
        GPIOOut::O0 => GPIOOutputType::PwmOut(&dev[0]),
        GPIOOut::O1 => GPIOOutputType::PwmOut(&dev[1]),
        GPIOOut::O2 => GPIOOutputType::PwmOut(&dev[2]),
        GPIOOut::O3 => GPIOOutputType::PwmOut(&dev[3]),
    });

    assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

    assert_eq!(
        [DBG_PWM_OUT, DBG_PWM_OUT, DBG_PWM_OUT, DBG_PWM_OUT],
        cpu.fpga().debug_types()
    );
    assert_eq!([0, 1, 2, 3], cpu.fpga().debug_values());

    Ok(())
}
