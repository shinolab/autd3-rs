use std::collections::HashMap;

use autd3_core::derive::*;
use autd3_driver::{
    datagram::*,
    error::AUTDDriverError,
    firmware::{
        cpu::TxMessage,
        fpga::{Drive, EmitIntensity, Phase},
    },
    geometry::Point3,
};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

use zerocopy::FromZeros;

#[derive(Gain, Debug)]
pub(crate) struct TestGain {
    pub(crate) data: HashMap<usize, Vec<Drive>>,
}

pub struct Impl {
    data: Vec<Drive>,
}

impl GainCalculator for Impl {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.data[tr.idx()]
    }
}

impl GainCalculatorGenerator for TestGain {
    type Calculator = Impl;

    fn generate(&mut self, device: &Device) -> Self::Calculator {
        Impl {
            data: self.data.remove(&device.idx()).unwrap(),
        }
    }
}

impl Gain for TestGain {
    type G = Self;

    fn init(self) -> Result<Self::G, GainError> {
        unimplemented!()
    }

    fn init_full(
        self,
        _: &Geometry,
        _filter: Option<&HashMap<usize, BitVec>>,
        _: bool,
    ) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

#[test]
fn send_gain_unsafe() -> anyhow::Result<()> {
    let mut rng = rand::rng();

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
                        .map(|_| Drive {
                            phase: Phase(rng.random()),
                            intensity: EmitIntensity(rng.random()),
                        })
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
            LoopBehavior::Infinite,
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
                        .map(|_| Drive {
                            phase: Phase(rng.random()),
                            intensity: EmitIntensity(rng.random()),
                        })
                        .collect(),
                )
            })
            .collect();
        let g = WithSegment::new(TestGain { data: buf.clone() }, Segment::S1, None);

        assert_eq!(Ok(()), send(&mut cpu, g, &geometry, &mut tx));

        assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
        assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
        assert_eq!(1, cpu.fpga().stm_cycle(Segment::S1));
        assert_eq!(0xFFFF, cpu.fpga().stm_freq_division(Segment::S1));
        assert_eq!(
            LoopBehavior::Infinite,
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
fn send_gain_invalid_segment_transition() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];

    // segment 0: FociSTM
    send(
        &mut cpu,
        WithSegment {
            inner: FociSTM {
                config: SamplingConfig::new(std::num::NonZeroU16::MAX),
                foci: (0..2)
                    .map(|_| ControlPoint::from(Point3::origin()))
                    .collect::<Vec<_>>(),
            },
            segment: Segment::S0,
            transition_mode: Some(TransitionMode::Immediate),
        },
        &geometry,
        &mut tx,
    )?;

    // segment 1: GainSTM
    send(
        &mut cpu,
        WithSegment {
            inner: GainSTM {
                config: SamplingConfig::new(std::num::NonZeroU16::MAX),
                gains: (0..2)
                    .map(|_| {
                        geometry
                            .iter()
                            .map(|dev| (dev.idx(), dev.iter().map(|_| Drive::NULL).collect()))
                            .collect()
                    })
                    .map(|buf: HashMap<usize, Vec<Drive>>| TestGain { data: buf.clone() })
                    .collect::<Vec<_>>(),
                option: GainSTMOption::default(),
            },
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::Immediate),
        },
        &geometry,
        &mut tx,
    )?;

    {
        let d = SwapSegment::Gain(Segment::S0, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSegmentTransition),
            send(&mut cpu, d, &geometry, &mut tx)
        );

        let d = SwapSegment::Gain(Segment::S1, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDDriverError::InvalidSegmentTransition),
            send(&mut cpu, d, &geometry, &mut tx)
        );
    }

    Ok(())
}
