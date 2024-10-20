use std::collections::HashMap;

use autd3_driver::{
    datagram::*,
    defined::ControlPoint,
    derive::*,
    firmware::{
        cpu::TxMessage,
        fpga::{Drive, EmitIntensity, Phase},
    },
    geometry::Vector3,
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[derive(Gain, Debug)]
pub(crate) struct TestGain {
    pub(crate) data: HashMap<usize, Vec<Drive>>,
}

pub struct Context {
    data: Vec<Drive>,
}

impl GainContext for Context {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.data[tr.idx()]
    }
}

impl GainContextGenerator for TestGain {
    type Context = Context;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Context {
            data: self.data.remove(&device.idx()).unwrap(),
        }
    }
}

impl Gain for TestGain {
    type G = Self;

    fn init_with_filter(
        self,
        _geometry: &Geometry,
        _filter: Option<HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDInternalError> {
        Ok(self)
    }
}

#[test]
fn send_gain() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

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
        let g = TestGain { data: buf.clone() };

        assert_eq!(Ok(()), send(&mut cpu, g, &geometry, &mut tx));

        assert!(cpu.fpga().is_stm_gain_mode(Segment::S0));
        assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
        assert_eq!(1, cpu.fpga().stm_cycle(Segment::S0));
        assert_eq!(0xFFFF, cpu.fpga().stm_freq_division(Segment::S0));
        assert_eq!(
            LoopBehavior::infinite(),
            cpu.fpga().stm_loop_behavior(Segment::S0)
        );
        buf[&0].iter().zip(cpu.fpga().drives()).for_each(|(&a, b)| {
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
        let g = TestGain { data: buf.clone() }.with_segment(Segment::S1, None);

        assert_eq!(Ok(()), send(&mut cpu, g, &geometry, &mut tx));

        assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
        assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
        assert_eq!(1, cpu.fpga().stm_cycle(Segment::S1));
        assert_eq!(0xFFFF, cpu.fpga().stm_freq_division(Segment::S1));
        assert_eq!(
            LoopBehavior::infinite(),
            cpu.fpga().stm_loop_behavior(Segment::S1)
        );
        buf[&0]
            .iter()
            .zip(cpu.fpga().drives_at(Segment::S1, 0))
            .for_each(|(&a, b)| {
                assert_eq!(a, b);
            });
    }

    {
        let d = SwapSegment::Gain(Segment::S1, TransitionMode::Immediate);

        assert_eq!(Ok(()), send(&mut cpu, d, &geometry, &mut tx));

        assert_eq!(Segment::S1, cpu.fpga().req_stm_segment());
    }

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn send_gain_invalid_segment_transition() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    // segment 0: FociSTM
    send(
        &mut cpu,
        FociSTM::new(
            SamplingConfig::FREQ_MIN,
            (0..2).map(|_| ControlPoint::new(Vector3::zeros())),
        )?
        .with_segment(Segment::S0, Some(TransitionMode::Immediate)),
        &geometry,
        &mut tx,
    )?;

    // segment 1: GainSTM
    send(
        &mut cpu,
        GainSTM::new(
            SamplingConfig::FREQ_MIN,
            (0..2)
                .map(|_| {
                    geometry
                        .iter()
                        .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::NULL).collect()))
                        .collect()
                })
                .map(|buf: HashMap<usize, Vec<Drive>>| TestGain { data: buf.clone() }),
        )?
        .with_segment(Segment::S1, Some(TransitionMode::Immediate)),
        &geometry,
        &mut tx,
    )?;

    {
        let d = SwapSegment::Gain(Segment::S0, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, d, &geometry, &mut tx)
        );

        let d = SwapSegment::Gain(Segment::S1, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSegmentTransition),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    Ok(())
}
