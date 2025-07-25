use std::collections::HashMap;

use autd3_core::{
    derive::*,
    firmware::transition_mode::{Immediate, Later},
    link::{MsgId, TxMessage},
};
use autd3_driver::{datagram::*, error::AUTDDriverError, geometry::Point3};
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

impl GainCalculator<'_> for Impl {
    fn calc(&self, tr: &Transducer) -> Drive {
        self.data[tr.idx()]
    }
}

impl GainCalculatorGenerator<'_> for TestGain {
    type Calculator = Impl;

    fn generate(&mut self, device: &Device) -> Self::Calculator {
        Impl {
            data: self.data.remove(&device.idx()).unwrap(),
        }
    }
}

impl Gain<'_> for TestGain {
    type G = Self;

    fn init(
        self,
        _: &Geometry,
        _: &Environment,
        _filter: &TransducerFilter,
    ) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

#[test]
fn send_gain_unsafe() -> anyhow::Result<()> {
    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    {
        let buf: HashMap<usize, Vec<Drive>> = geometry
            .iter()
            .map(|dev| {
                (
                    dev.idx(),
                    dev.iter()
                        .map(|_| Drive {
                            phase: Phase(rng.random()),
                            intensity: Intensity(rng.random()),
                        })
                        .collect(),
                )
            })
            .collect();
        let g = TestGain { data: buf.clone() };

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, g, &mut geometry, &mut tx)
        );

        assert!(cpu.fpga().is_stm_gain_mode(Segment::S0));
        assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
        assert_eq!(1, cpu.fpga().stm_cycle(Segment::S0));
        assert_eq!(0xFFFF, cpu.fpga().stm_freq_divide(Segment::S0));
        assert_eq!(0xFFFF, cpu.fpga().stm_loop_count(Segment::S0));
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
                            intensity: Intensity(rng.random()),
                        })
                        .collect(),
                )
            })
            .collect();
        let g = WithSegment::new(TestGain { data: buf.clone() }, Segment::S1, Later);

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, g, &mut geometry, &mut tx)
        );

        assert!(cpu.fpga().is_stm_gain_mode(Segment::S1));
        assert_eq!(Segment::S0, cpu.fpga().req_stm_segment());
        assert_eq!(1, cpu.fpga().stm_cycle(Segment::S1));
        assert_eq!(0xFFFF, cpu.fpga().stm_freq_divide(Segment::S1));
        assert_eq!(0xFFFF, cpu.fpga().stm_loop_count(Segment::S1));
        buf[&0]
            .iter()
            .zip(cpu.fpga().drives_at(Segment::S1, 0))
            .for_each(|(&a, b)| {
                assert_eq!(a, b);
            });
    }

    {
        let d = SwapSegmentGain(Segment::S1);

        assert_eq!(
            Ok(()),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        assert_eq!(Segment::S1, cpu.fpga().req_stm_segment());
    }

    Ok(())
}

#[test]
fn send_gain_invalid_segment_transition() -> anyhow::Result<()> {
    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new_zeroed(); 1];
    let mut msg_id = MsgId::new(0);

    // segment 0: FociSTM
    send(
        &mut msg_id,
        &mut cpu,
        WithSegment {
            inner: FociSTM {
                config: SamplingConfig::new(std::num::NonZeroU16::MAX),
                foci: (0..2)
                    .map(|_| ControlPoint::from(Point3::origin()))
                    .collect::<Vec<_>>(),
            },
            segment: Segment::S0,
            transition_mode: Immediate,
        },
        &mut geometry,
        &mut tx,
    )?;

    // segment 1: GainSTM
    send(
        &mut msg_id,
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
            transition_mode: Immediate,
        },
        &mut geometry,
        &mut tx,
    )?;

    {
        let d = SwapSegmentGain(Segment::S0);
        assert_eq!(
            Err(AUTDDriverError::InvalidSegmentTransition),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );

        let d = SwapSegmentGain(Segment::S1);
        assert_eq!(
            Err(AUTDDriverError::InvalidSegmentTransition),
            send(&mut msg_id, &mut cpu, d, &mut geometry, &mut tx)
        );
    }

    Ok(())
}
