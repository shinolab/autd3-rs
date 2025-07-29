use autd3_core::{
    firmware::GPIOOut,
    link::{MsgId, TxMessage},
};
use autd3_driver::{datagram::*, ethercat::DcSysTime};
use autd3_firmware_emulator::{CPUEmulator, fpga::params::*};

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[rstest::rstest]
#[case([GPIO_O_TYPE_NONE, GPIO_O_TYPE_BASE_SIG, GPIO_O_TYPE_THERMO, GPIO_O_TYPE_FORCE_FAN], [0, 0, 0, 0], [None, Some(GPIOOutputType::BaseSignal), Some(GPIOOutputType::Thermo), Some(GPIOOutputType::ForceFan)])]
#[case([GPIO_O_TYPE_SYNC, GPIO_O_TYPE_MOD_SEGMENT, GPIO_O_TYPE_MOD_IDX, GPIO_O_TYPE_STM_SEGMENT], [0, 0, 0x01, 0], [Some(GPIOOutputType::Sync), Some(GPIOOutputType::ModSegment), Some(GPIOOutputType::ModIdx(0x01)), Some(GPIOOutputType::StmSegment)])]
#[case([GPIO_O_TYPE_STM_IDX, GPIO_O_TYPE_IS_STM_MODE, GPIO_O_TYPE_SYS_TIME_EQ, GPIO_O_TYPE_DIRECT], [0x02, 0, 2, 1], [Some(GPIOOutputType::StmIdx(0x02)), Some(GPIOOutputType::IsStmMode), Some(GPIOOutputType::SysTimeEq(DcSysTime::ZERO + 2 * std::time::Duration::from_micros(25))), Some(GPIOOutputType::Direct(true))])]
#[case([GPIO_O_TYPE_SYNC_DIFF, GPIO_O_TYPE_NONE, GPIO_O_TYPE_NONE, GPIO_O_TYPE_NONE], [0, 0, 0, 0], [Some(GPIOOutputType::SyncDiff), None, None, None])]
fn send_gpio_output(
    #[case] expect_types: [u8; 4],
    #[case] expect_values: [u64; 4],
    #[case] gpio_out_types: [Option<GPIOOutputType<'static>>; 4],
) -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    let d = GPIOOutputs::new(|_, gpio| match gpio {
        GPIOOut::O0 => gpio_out_types[0].clone(),
        GPIOOut::O1 => gpio_out_types[1].clone(),
        GPIOOut::O2 => gpio_out_types[2].clone(),
        GPIOOut::O3 => gpio_out_types[3].clone(),
    });

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert_eq!(expect_types, cpu.fpga().gpio_out_types());
    assert_eq!(expect_values, cpu.fpga().gpio_out_values());

    Ok(())
}

#[test]
fn send_gpio_output_pwm() -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    let d = GPIOOutputs::new(|dev, gpio| match gpio {
        GPIOOut::O0 => Some(GPIOOutputType::PwmOut(&dev[0])),
        GPIOOut::O1 => Some(GPIOOutputType::PwmOut(&dev[1])),
        GPIOOut::O2 => Some(GPIOOutputType::PwmOut(&dev[2])),
        GPIOOut::O3 => Some(GPIOOutputType::PwmOut(&dev[3])),
    });

    assert_eq!(
        Ok(()),
        send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
    );

    assert_eq!(
        [
            GPIO_O_TYPE_PWM_OUT,
            GPIO_O_TYPE_PWM_OUT,
            GPIO_O_TYPE_PWM_OUT,
            GPIO_O_TYPE_PWM_OUT
        ],
        cpu.fpga().gpio_out_types()
    );
    assert_eq!([0, 1, 2, 3], cpu.fpga().gpio_out_values());

    Ok(())
}
