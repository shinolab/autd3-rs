use std::collections::HashMap;

use autd3_driver::{
    datagram::*, defined::ControlPoint, derive::*, firmware::cpu::TxDatagram, geometry::Vector3,
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[derive(Gain)]
pub(crate) struct TestGain {
    pub(crate) buf: HashMap<usize, Vec<Drive>>,
}

impl Gain for TestGain {
    fn calc(&self, _: &Geometry) -> GainCalcResult {
        let buf = self.buf.clone();
        Ok(Box::new(move |dev| {
            let buf = buf[&dev.idx()].clone();
            Box::new(move |tr| buf[tr.idx()])
        }))
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

        assert_eq!(Ok(()), send(&mut cpu, g, &geometry, &mut tx));

        assert!(cpu.fpga().is_stm_gain_mode(Segment::S0));
        assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
        assert_eq!(1, cpu.fpga().stm_cycle(Segment::S0));
        assert_eq!(0xFFFFFFFF, cpu.fpga().stm_freq_division(Segment::S0));
        assert_eq!(
            LoopBehavior::infinite(),
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
        let g = TestGain { buf: buf.clone() }.with_segment(Segment::S1, false);

        assert_eq!(Ok(()), send(&mut cpu, g, &geometry, &mut tx));

        assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
        assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
        assert_eq!(1, cpu.fpga().stm_cycle(Segment::S1));
        assert_eq!(0xFFFFFFFF, cpu.fpga().stm_freq_division(Segment::S1));
        assert_eq!(
            LoopBehavior::infinite(),
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
        let d = SwapSegment::gain(Segment::S1);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(Segment::S1, cpu.fpga().req_stm_segment());
    }

    Ok(())
}

#[test]
fn send_gain_invalid_segment_transition() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    // segment 0: FocusSTM
    send(
        &mut cpu,
        FocusSTM::from_sampling_config(
            SamplingConfig::DivisionRaw(0xFFFFFFFF),
            (0..2).map(|_| ControlPoint::new(Vector3::zeros())),
        )
        .with_segment(Segment::S0, Some(TransitionMode::Immediate)),
        &geometry,
        &mut tx,
    )?;

    // segment 1: GainSTM
    send(
        &mut cpu,
        GainSTM::from_sampling_config(
            SamplingConfig::DivisionRaw(0xFFFFFFFF),
            (0..2)
                .map(|_| {
                    geometry
                        .iter()
                        .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::null()).collect()))
                        .collect()
                })
                .map(|buf: HashMap<usize, Vec<Drive>>| TestGain { buf: buf.clone() }),
        )
        .with_segment(Segment::S1, Some(TransitionMode::Immediate)),
        &geometry,
        &mut tx,
    )?;

    {
        let d = SwapSegment::gain(Segment::S0);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, d, &geometry, &mut tx)
        );

        let d = SwapSegment::gain(Segment::S1);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    Ok(())
}
