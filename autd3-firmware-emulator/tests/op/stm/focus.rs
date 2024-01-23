use autd3_driver::{
    autd3_device::AUTD3,
    cpu::TxDatagram,
    defined::{METER, MILLIMETER},
    fpga::{
        FOCUS_STM_BUF_SIZE_MAX, FOCUS_STM_FIXED_NUM_UNIT, SAMPLING_FREQ_DIV_MAX,
        SAMPLING_FREQ_DIV_MIN,
    },
    geometry::{Geometry, IntoDevice, Vector3},
    operation::{ControlPoint, FocusSTMOp, NullOp, OperationHandler},
};
use autd3_firmware_emulator::CPUEmulator;

use num_integer::Roots;
use rand::*;

#[test]
fn send_focus_stm() {
    let mut rng = rand::thread_rng();

    let geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());

    let mut tx = TxDatagram::new(1);

    let freq_div = rng.gen_range(SAMPLING_FREQ_DIV_MIN..=SAMPLING_FREQ_DIV_MAX);
    let foci: Vec<_> = (0..FOCUS_STM_BUF_SIZE_MAX)
        .map(|_| {
            ControlPoint::new(Vector3::new(
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
            ))
            .with_intensity(rng.gen::<u8>())
        })
        .collect();
    let start_idx = Some(rng.gen_range(0..foci.len()) as u16);
    let finish_idx = Some(rng.gen_range(0..foci.len()) as u16);
    let mut op = FocusSTMOp::new(foci.clone(), freq_div, start_idx, finish_idx);
    let mut op_null = NullOp::default();

    assert!(!cpu.fpga().is_stm_mode());
    assert!(!cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    loop {
        if OperationHandler::is_finished(&mut op, &mut op_null, &geometry) {
            break;
        }
        OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
        cpu.send(&tx);
        assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);
    }

    assert!(cpu.fpga().is_stm_mode());
    assert!(!cpu.fpga().is_stm_gain_mode());
    assert_eq!(cpu.fpga().stm_start_idx(), start_idx);
    assert_eq!(cpu.fpga().stm_finish_idx(), finish_idx);
    assert_eq!(cpu.fpga().stm_cycle(), foci.len());
    assert_eq!(cpu.fpga().stm_frequency_division(), freq_div);
    assert_eq!(
        cpu.fpga().sound_speed(),
        (geometry[0].sound_speed / METER * 1024.0).round() as _
    );
    foci.iter().enumerate().for_each(|(focus_idx, focus)| {
        cpu.fpga()
            .intensities_and_phases(focus_idx)
            .iter()
            .enumerate()
            .for_each(|(tr_idx, &(intensity, phase))| {
                let tr = cpu.fpga().local_tr_pos()[tr_idx];
                let tx = ((tr >> 16) & 0xFFFF) as i32;
                let ty = (tr & 0xFFFF) as i16 as i32;
                let tz = 0;
                let fx = (focus.point().x / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                let fy = (focus.point().y / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                let fz = (focus.point().z / FOCUS_STM_FIXED_NUM_UNIT).round() as i32;
                let d = ((tx - fx).pow(2) + (ty - fy).pow(2) + (tz - fz).pow(2)).sqrt() as u64;
                let q = (d << 18) / cpu.fpga().sound_speed() as u64;
                assert_eq!(phase, (q & 0xFF) as u8);
                assert_eq!(intensity, focus.intensity().value());
            })
    });

    let foci: Vec<_> = (0..2)
        .map(|_| {
            ControlPoint::new(Vector3::new(
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
                rng.gen_range(-100.0 * MILLIMETER..100.0 * MILLIMETER),
            ))
            .with_intensity(rng.gen::<u8>())
        })
        .collect();
    let start_idx = None;
    let finish_idx = None;
    let mut op = FocusSTMOp::new(foci.clone(), freq_div, start_idx, finish_idx);
    let mut op_null = NullOp::default();

    OperationHandler::init(&mut op, &mut op_null, &geometry).unwrap();
    OperationHandler::pack(&mut op, &mut op_null, &geometry, &mut tx).unwrap();
    cpu.send(&tx);
    assert!(OperationHandler::is_finished(
        &mut op,
        &mut op_null,
        &geometry
    ));
    assert_eq!(cpu.ack(), tx.headers().next().unwrap().msg_id);

    assert!(cpu.fpga().is_stm_mode());
    assert!(!cpu.fpga().is_stm_gain_mode());
    assert!(cpu.fpga().stm_start_idx().is_none());
    assert!(cpu.fpga().stm_finish_idx().is_none());
    assert_eq!(cpu.fpga().stm_cycle(), foci.len());
    assert_eq!(cpu.fpga().stm_frequency_division(), freq_div);
    assert_eq!(
        cpu.fpga().sound_speed(),
        (geometry[0].sound_speed / METER * 1024.0).round() as _
    );
}
