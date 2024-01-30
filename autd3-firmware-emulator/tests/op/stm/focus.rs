use autd3_driver::{
    cpu::TxDatagram,
    defined::{METER, MILLIMETER},
    derive::Phase,
    fpga::{
        FOCUS_STM_BUF_SIZE_MAX, FOCUS_STM_FIXED_NUM_UNIT, SAMPLING_FREQ_DIV_MAX,
        SAMPLING_FREQ_DIV_MIN, SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT,
    },
    geometry::Vector3,
    operation::{ControlPoint, FocusSTMOp},
};
use autd3_firmware_emulator::CPUEmulator;

use num_integer::Roots;
use rand::*;

use crate::{create_geometry, send};

pub fn gen_random_foci(num: usize) -> Vec<ControlPoint> {
    let mut rng = rand::thread_rng();
    (0..num)
        .map(|_| {
            ControlPoint::new(Vector3::new(
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
            ))
            .with_intensity(rng.gen::<u8>())
        })
        .collect()
}

#[test]
fn test_send_focus_stm() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let freq_div = rng.gen_range(
        SAMPLING_FREQ_DIV_MIN
            * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
            ..=SAMPLING_FREQ_DIV_MAX,
    );
    let foci = gen_random_foci(FOCUS_STM_BUF_SIZE_MAX);
    let mut op = FocusSTMOp::new(foci.clone(), freq_div, None, None);

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert!(cpu.fpga().is_stm_mode());
    assert!(!cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());
    assert_eq!(foci.len(), cpu.fpga().stm_cycle());
    assert_eq!(freq_div, cpu.fpga().stm_frequency_division());
    assert_eq!(
        (geometry[0].sound_speed / METER * 1024.0).round() as u32,
        cpu.fpga().sound_speed()
    );
    foci.iter().enumerate().for_each(|(focus_idx, focus)| {
        cpu.fpga()
            .drives(focus_idx)
            .iter()
            .enumerate()
            .for_each(|(tr_idx, &drive)| {
                let tr = cpu.fpga().local_tr_pos()[tr_idx];
                let tx = ((tr >> 16) & 0xFFFF) as i32;
                let ty = (tr & 0xFFFF) as i16 as i32;
                let tz = 0;
                let fx = (focus.point().x / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                let fy = (focus.point().y / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                let fz = (focus.point().z / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                let d = ((tx - fx).pow(2) + (ty - fy).pow(2) + (tz - fz).pow(2)).sqrt() as u64;
                let q = (d << 18) / cpu.fpga().sound_speed() as u64;
                assert_eq!(Phase::new((q & 0xFF) as u8), drive.phase);
                assert_eq!(focus.intensity(), drive.intensity);
            })
    });
    Ok(())
}

#[test]
fn test_send_focus_stm_with_idx() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let foci = gen_random_foci(2);
    let start_idx = Some(rng.gen_range(0..foci.len()) as u16);
    let finish_idx = Some(rng.gen_range(0..foci.len()) as u16);
    let mut op = FocusSTMOp::new(
        foci,
        SAMPLING_FREQ_DIV_MIN
            * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
        start_idx,
        finish_idx,
    );

    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());

    send(&mut cpu, &mut op, &geometry, &mut tx)?;

    assert_eq!(start_idx, cpu.fpga().stm_start_idx());
    assert_eq!(finish_idx, cpu.fpga().stm_finish_idx());

    Ok(())
}
