use autd3_driver::{
    datagram::*,
    firmware::{cpu::TxDatagram, fpga::DebugType},
};
use autd3_firmware_emulator::{fpga::params::*, CPUEmulator};

use crate::{create_geometry, send};

#[rstest::rstest]
#[test]
#[case([DBG_NONE, DBG_BASE_SIG, DBG_THERMO, DBG_FORCE_FAN], [0, 0, 0, 0], [DebugType::None, DebugType::BaseSignal, DebugType::Thermo, DebugType::ForceFan])]
#[case([DBG_SYNC, DBG_MOD_SEGMENT, DBG_MOD_IDX, DBG_STM_SEGMENT], [0, 0, 0x01, 0], [DebugType::Sync, DebugType::ModSegment, DebugType::ModIdx(0x01), DebugType::StmSegment])]
#[case([DBG_STM_IDX, DBG_IS_STM_MODE, DBG_NONE, DBG_DIRECT], [0x02, 0, 0, 1], [DebugType::StmIdx(0x02), DebugType::IsStmMode, DebugType::None, DebugType::Direct(true)])]
fn send_debug_output_idx(
    #[case] expect_types: [u8; 4],
    #[case] expect_values: [u16; 4],
    #[case] debug_types: [DebugType<'static>; 4],
) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let (mut op, _) = ConfigureDebugSettings::new(|_| debug_types.clone()).operation()?;

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert_eq!(expect_types, cpu.fpga().debug_types());
    assert_eq!(expect_values, cpu.fpga().debug_values());

    Ok(())
}

#[test]
fn send_debug_pwm_out() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let (mut op, _) = ConfigureDebugSettings::new(|dev| {
        [
            DebugType::PwmOut(&dev[0]),
            DebugType::PwmOut(&dev[1]),
            DebugType::PwmOut(&dev[2]),
            DebugType::PwmOut(&dev[3]),
        ]
    })
    .operation()?;

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert_eq!(
        [DBG_PWM_OUT, DBG_PWM_OUT, DBG_PWM_OUT, DBG_PWM_OUT],
        cpu.fpga().debug_types()
    );
    assert_eq!([0, 1, 2, 3], cpu.fpga().debug_values());

    Ok(())
}
