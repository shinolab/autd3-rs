use std::collections::HashMap;

use autd3_driver::{
    common::EmitIntensity,
    cpu::TxDatagram,
    datagram::*,
    derive::*,
    geometry::{Geometry, Vector3},
    operation::{ControlPoint, FocusSTMOp, GainChangeSegmentOp, GainOp, GainSTMMode, GainSTMOp},
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[derive(Gain)]
pub(crate) struct TestGain {
    pub(crate) buf: HashMap<usize, Vec<Drive>>,
}

impl Gain for TestGain {
    fn calc(
        &self,
        _: &Geometry,
        _: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(self.buf.clone())
    }
}

#[test]
fn send_gain() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let buf: HashMap<usize, Vec<Drive>> = geometry
            .iter()
            .map(|dev| {
                (
                    dev.idx(),
                    dev.iter()
                        .map(|_| Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen())))
                        .collect(),
                )
            })
            .collect();
        let g = TestGain { buf: buf.clone() };

        let (mut op, _) = g.operation_with_segment(Segment::S0, true)?;

        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        assert!(cpu.fpga().is_stm_gain_mode(Segment::S0));
        assert_eq!(Segment::S0, cpu.fpga().current_stm_segment());
        assert_eq!(1, cpu.fpga().stm_cycle(Segment::S0));
        assert_eq!(0xFFFFFFFF, cpu.fpga().stm_frequency_division(Segment::S0));
        assert_eq!(
            LoopBehavior::Infinite,
            cpu.fpga().stm_loop_behavior(Segment::S0)
        );
        buf[&0]
            .iter()
            .zip(cpu.fpga().drives(Segment::S0, 0))
            .for_each(|(&a, b)| {
                assert_eq!(a, b);
            });
    }

    {
        let buf: HashMap<usize, Vec<Drive>> = geometry
            .iter()
            .map(|dev| {
                (
                    dev.idx(),
                    dev.iter()
                        .map(|_| Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen())))
                        .collect(),
                )
            })
            .collect();
        let g = TestGain { buf: buf.clone() };

        let (mut op, _) = g.operation_with_segment(Segment::S1, false)?;

        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
        assert_eq!(Segment::S0, cpu.fpga().current_stm_segment());
        assert_eq!(1, cpu.fpga().stm_cycle(Segment::S1));
        assert_eq!(0xFFFFFFFF, cpu.fpga().stm_frequency_division(Segment::S1));
        assert_eq!(
            LoopBehavior::Infinite,
            cpu.fpga().stm_loop_behavior(Segment::S1)
        );
        buf[&0]
            .iter()
            .zip(cpu.fpga().drives(Segment::S1, 0))
            .for_each(|(&a, b)| {
                assert_eq!(a, b);
            });
    }

    {
        let mut op = GainChangeSegmentOp::new(Segment::S1);

        send(&mut cpu, &mut op, &geometry, &mut tx)?;

        assert_eq!(Segment::S1, cpu.fpga().current_stm_segment());
    }

    Ok(())
}

#[test]
fn send_gain_invalid_segment_transition() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    // segment 0: FocusSTM
    {
        let freq_div = 0xFFFFFFFF;
        let foci = (0..2)
            .map(|_| ControlPoint::new(Vector3::zeros()))
            .collect();
        let loop_behaviour = LoopBehavior::Infinite;
        let segment = Segment::S0;
        let mut op = FocusSTMOp::new(foci, freq_div, loop_behaviour, segment, true);

        send(&mut cpu, &mut op, &geometry, &mut tx)?;
    }

    // segment 1: GainSTM
    {
        let bufs: Vec<HashMap<usize, Vec<Drive>>> = (0..2)
            .map(|_| {
                geometry
                    .iter()
                    .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::null()).collect()))
                    .collect()
            })
            .collect();
        let loop_behavior = LoopBehavior::Infinite;
        let segment = Segment::S1;
        let freq_div = 0xFFFFFFFF;
        let mut op = GainSTMOp::new(
            bufs.iter()
                .map(|buf| TestGain { buf: buf.clone() })
                .collect(),
            GainSTMMode::PhaseIntensityFull,
            freq_div,
            loop_behavior,
            segment,
            true,
        );

        send(&mut cpu, &mut op, &geometry, &mut tx)?;
    }

    {
        let mut op = GainChangeSegmentOp::new(Segment::S0);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );

        let mut op = GainChangeSegmentOp::new(Segment::S1);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    Ok(())
}
