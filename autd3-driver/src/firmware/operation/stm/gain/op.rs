#![allow(clippy::type_complexity)]

use std::{iter::Peekable, mem::size_of};

use crate::{
    derive::{LoopBehavior, SamplingConfig, Transducer},
    error::AUTDInternalError,
    firmware::{
        cpu::GainSTMMode,
        fpga::{
            Drive, Segment, TransitionMode, GAIN_STM_BUF_SIZE_MAX, STM_BUF_SIZE_MIN,
            TRANSITION_MODE_NONE,
        },
        operation::{write_to_tx, Operation, TypeTag},
    },
    geometry::Device,
};

use super::{
    control_flags::GainSTMControlFlags,
    reduced_phase::{PhaseFull, PhaseHalf},
};

#[repr(C, align(2))]
struct GainSTMHead {
    tag: TypeTag,
    flag: GainSTMControlFlags,
    mode: GainSTMMode,
    transition_mode: u8,
    freq_div: u16,
    rep: u16,
    transition_value: u64,
}

#[repr(C, align(2))]
struct GainSTMSubseq {
    tag: TypeTag,
    flag: GainSTMControlFlags,
}

pub struct GainSTMOp {
    gains: Peekable<std::vec::IntoIter<Box<dyn Fn(&Transducer) -> Drive + Sync + Send>>>,
    sent: usize,
    is_done: bool,
    mode: GainSTMMode,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl GainSTMOp {
    pub fn new(
        gains: Vec<Box<dyn Fn(&Transducer) -> Drive + Sync + Send>>,
        mode: GainSTMMode,
        config: SamplingConfig,
        loop_behavior: LoopBehavior,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Self {
        Self {
            gains: gains.into_iter().peekable(),
            sent: 0,
            is_done: false,
            mode,
            config,
            loop_behavior,
            segment,
            transition_mode,
        }
    }
}

impl Operation for GainSTMOp {
    fn required_size(&self, device: &Device) -> usize {
        if self.sent == 0 {
            size_of::<GainSTMHead>() + device.num_transducers() * size_of::<Drive>()
        } else {
            size_of::<GainSTMSubseq>() + device.num_transducers() * size_of::<Drive>()
        }
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let is_first = self.sent == 0;

        let offset = if is_first {
            size_of::<GainSTMHead>()
        } else {
            size_of::<GainSTMSubseq>()
        };

        let send = unsafe {
            let mut send = 0;
            match self.mode {
                GainSTMMode::PhaseIntensityFull => {
                    if let Some(g) = self.gains.next() {
                        let dst = std::slice::from_raw_parts_mut(
                            tx[offset..].as_mut_ptr() as *mut Drive,
                            device.len(),
                        );
                        dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                            *d = g(tr);
                        });
                        send += 1;
                    }
                }
                GainSTMMode::PhaseFull => {
                    if let Some(g) = self.gains.next() {
                        let dst = std::slice::from_raw_parts_mut(
                            tx[offset..].as_mut_ptr() as *mut PhaseFull<0>,
                            device.len(),
                        );
                        dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                            d.set(g(tr));
                        });
                        send += 1;
                    }
                    if let Some(g) = self.gains.next() {
                        let dst = std::slice::from_raw_parts_mut(
                            tx[offset..].as_mut_ptr() as *mut PhaseFull<1>,
                            device.len(),
                        );
                        dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                            d.set(g(tr));
                        });
                        send += 1;
                    }
                }
                GainSTMMode::PhaseHalf => {
                    if let Some(g) = self.gains.next() {
                        let dst = std::slice::from_raw_parts_mut(
                            tx[offset..].as_mut_ptr() as *mut PhaseHalf<0>,
                            device.len(),
                        );
                        dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                            d.set(g(tr));
                        });
                        send += 1;
                    }
                    if let Some(g) = self.gains.next() {
                        let dst = std::slice::from_raw_parts_mut(
                            tx[offset..].as_mut_ptr() as *mut PhaseHalf<1>,
                            device.len(),
                        );
                        dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                            d.set(g(tr));
                        });
                        send += 1;
                    }
                    if let Some(g) = self.gains.next() {
                        let dst = std::slice::from_raw_parts_mut(
                            tx[offset..].as_mut_ptr() as *mut PhaseHalf<2>,
                            device.len(),
                        );
                        dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                            d.set(g(tr));
                        });
                        send += 1;
                    }
                    if let Some(g) = self.gains.next() {
                        let dst = std::slice::from_raw_parts_mut(
                            tx[offset..].as_mut_ptr() as *mut PhaseHalf<3>,
                            device.len(),
                        );
                        dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                            d.set(g(tr));
                        });
                        send += 1;
                    }
                }
            }
            send
        };

        self.sent += send;
        if self.sent > GAIN_STM_BUF_SIZE_MAX {
            return Err(AUTDInternalError::GainSTMSizeOutOfRange(self.sent));
        }

        let mut flag = if self.segment == Segment::S1 {
            GainSTMControlFlags::SEGMENT
        } else {
            GainSTMControlFlags::NONE
        };
        if self.gains.peek().is_none() {
            if self.sent < STM_BUF_SIZE_MIN {
                return Err(AUTDInternalError::GainSTMSizeOutOfRange(self.sent));
            }
            self.is_done = true;
            flag.set(GainSTMControlFlags::END, true);
            flag.set(
                GainSTMControlFlags::TRANSITION,
                self.transition_mode.is_some(),
            );
        }

        flag.set(
            GainSTMControlFlags::SEND_BIT0,
            ((send as u8 - 1) & 0x01) != 0,
        );
        flag.set(
            GainSTMControlFlags::SEND_BIT1,
            ((send as u8 - 1) & 0x02) != 0,
        );

        if is_first {
            write_to_tx(
                GainSTMHead {
                    tag: TypeTag::GainSTM,
                    flag: GainSTMControlFlags::BEGIN | flag,
                    mode: self.mode,
                    transition_mode: self
                        .transition_mode
                        .map(|m| m.mode())
                        .unwrap_or(TRANSITION_MODE_NONE),
                    transition_value: self.transition_mode.map(|m| m.value()).unwrap_or(0),
                    freq_div: self.config.division(),
                    rep: self.loop_behavior.rep(),
                },
                tx,
            );
        } else {
            write_to_tx(
                GainSTMSubseq {
                    tag: TypeTag::GainSTM,
                    flag,
                },
                tx,
            );
        }

        if is_first {
            Ok(size_of::<GainSTMHead>() + device.num_transducers() * size_of::<Drive>())
        } else {
            Ok(size_of::<GainSTMSubseq>() + device.num_transducers() * size_of::<Drive>())
        }
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::{mem::offset_of, num::NonZeroU16};

    use rand::prelude::*;

    use super::*;
    use crate::{
        ethercat::DcSysTime,
        firmware::{
            fpga::{EmitIntensity, Phase},
            operation::tests::parse_tx_as,
        },
        geometry::tests::create_device,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[test]
    fn test_phase_intensity_full() {
        const GAIN_STM_SIZE: usize = 3;
        const FRAME_SIZE: usize = size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::thread_rng();

        let gain_data: Vec<Vec<Drive>> = (0..GAIN_STM_SIZE)
            .map(|_| {
                (0..NUM_TRANS_IN_UNIT)
                    .map(|_| {
                        Drive::new(
                            Phase::new(rng.gen_range(0x00..=0xFF)),
                            EmitIntensity::new(rng.gen_range(0..=0xFF)),
                        )
                    })
                    .collect()
            })
            .collect();

        let freq_div = rng.gen_range(0x0001..=0xFFFF);
        let rep = rng.gen_range(0x000..=0xFFFF);
        let segment = Segment::S0;
        let transition_value = 0x0123456789ABCDEF;
        let transition_mode = TransitionMode::SysTime(
            DcSysTime::from_utc(
                time::macros::datetime!(2000-01-01 0:00 UTC)
                    + std::time::Duration::from_nanos(transition_value),
            )
            .unwrap(),
        );

        let mut op = GainSTMOp::new(
            {
                let gain_data = gain_data.clone();
                gain_data
                    .into_iter()
                    .map(|g| Box::new(move |tr: &Transducer| g[tr.idx()]) as Box<_>)
                    .collect()
            },
            GainSTMMode::PhaseIntensityFull,
            SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            LoopBehavior { rep },
            segment,
            Some(transition_mode),
        );

        // First frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2
            );

            assert_eq!(op.sent, 0);

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2)
            );

            assert_eq!(op.sent, 1);

            assert_eq!(TypeTag::GainSTM as u8, tx[0]);
            assert_eq!(
                GainSTMControlFlags::BEGIN.bits(),
                tx[offset_of!(GainSTMHead, flag)] & 0x3F
            );
            assert_eq!(0, tx[offset_of!(GainSTMHead, flag)] >> 6);
            assert_eq!(
                GainSTMMode::PhaseIntensityFull as u8,
                tx[offset_of!(GainSTMHead, mode)]
            );
            assert_eq!(
                freq_div,
                parse_tx_as::<u16>(&tx[offset_of!(GainSTMHead, freq_div)..])
            );
            assert_eq!(rep, parse_tx_as::<u16>(&tx[offset_of!(GainSTMHead, rep)..]));
            assert_eq!(
                transition_mode.mode(),
                tx[offset_of!(GainSTMHead, transition_mode)]
            );
            assert_eq!(
                ((transition_value / crate::ethercat::EC_CYCLE_TIME_BASE_NANO_SEC) + 1)
                    * crate::ethercat::EC_CYCLE_TIME_BASE_NANO_SEC,
                parse_tx_as::<u64>(&tx[offset_of!(GainSTMHead, transition_value)..])
            );

            tx[size_of::<GainSTMHead>()..]
                .chunks(size_of::<Drive>())
                .zip(gain_data[0].iter())
                .for_each(|(d, g)| {
                    assert_eq!(d[0], g.phase().value());
                    assert_eq!(d[1], g.intensity().value());
                })
        }

        // Second frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );

            assert_eq!(op.sent, 2);

            assert_eq!(TypeTag::GainSTM as u8, tx[0]);
            assert_eq!(
                GainSTMControlFlags::NONE.bits(),
                tx[offset_of!(GainSTMHead, flag)] & 0x3F
            );
            assert_eq!(0, tx[offset_of!(GainSTMHead, flag)] >> 6);

            tx[size_of::<GainSTMSubseq>()..]
                .chunks(size_of::<Drive>())
                .zip(gain_data[1].iter())
                .for_each(|(d, g)| {
                    assert_eq!(d[0], g.phase().value());
                    assert_eq!(d[1], g.intensity().value());
                })
        }

        // Final frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );

            assert_eq!(op.sent, GAIN_STM_SIZE);

            assert_eq!(TypeTag::GainSTM as u8, tx[0]);
            assert_eq!(
                (GainSTMControlFlags::END | GainSTMControlFlags::TRANSITION).bits(),
                tx[offset_of!(GainSTMHead, flag)] & 0x3F
            );
            assert_eq!(0, tx[offset_of!(GainSTMHead, flag)] >> 6);
            tx[size_of::<GainSTMSubseq>()..]
                .chunks(size_of::<Drive>())
                .zip(gain_data[2].iter())
                .for_each(|(d, g)| {
                    assert_eq!(d[0], g.phase().value());
                    assert_eq!(d[1], g.intensity().value());
                })
        }
    }

    #[test]
    fn test_phase_full() {
        const GAIN_STM_SIZE: usize = 5;
        const FRAME_SIZE: usize = size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::thread_rng();

        let gain_data: Vec<Vec<Drive>> = (0..GAIN_STM_SIZE)
            .map(|_| {
                (0..NUM_TRANS_IN_UNIT)
                    .map(|_| {
                        Drive::new(
                            Phase::new(rng.gen_range(0x00..=0xFF)),
                            EmitIntensity::new(rng.gen_range(0..=0xFF)),
                        )
                    })
                    .collect()
            })
            .collect();

        let freq_div = rng.gen_range(0x0001..=0xFFFF);
        let rep = rng.gen_range(0x0001..=0xFFFF);
        let segment = Segment::S1;
        let mut op = GainSTMOp::new(
            {
                let gain_data = gain_data.clone();
                gain_data
                    .into_iter()
                    .map(|g| Box::new(move |tr: &Transducer| g[tr.idx()]) as Box<_>)
                    .collect()
            },
            GainSTMMode::PhaseFull,
            SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            LoopBehavior { rep },
            segment,
            None,
        );

        assert_eq!(op.sent, 0);

        // First frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2)
            );

            assert_eq!(op.sent, 2);

            assert_eq!(TypeTag::GainSTM as u8, tx[0]);
            assert_eq!(
                (GainSTMControlFlags::BEGIN | GainSTMControlFlags::SEGMENT).bits(),
                tx[offset_of!(GainSTMHead, flag)] & 0x3F
            );
            assert_eq!(1, tx[offset_of!(GainSTMHead, flag)] >> 6);

            assert_eq!(
                GainSTMMode::PhaseFull as u8,
                tx[offset_of!(GainSTMHead, mode)]
            );
            tx[size_of::<GainSTMHead>()..]
                .chunks(size_of::<Drive>())
                .zip(gain_data[0].iter())
                .zip(gain_data[1].iter())
                .for_each(|((d, g0), g1)| {
                    assert_eq!(d[0], g0.phase().value());
                    assert_eq!(d[1], g1.phase().value());
                });
        }

        // Second frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );

            assert_eq!(op.sent, 4);

            assert_eq!(TypeTag::GainSTM as u8, tx[0]);
            assert_eq!(
                GainSTMControlFlags::SEGMENT.bits(),
                tx[offset_of!(GainSTMHead, flag)] & 0x3F
            );
            assert_eq!(1, tx[offset_of!(GainSTMHead, flag)] >> 6);
            tx[size_of::<GainSTMSubseq>()..]
                .chunks(size_of::<Drive>())
                .zip(gain_data[2].iter())
                .zip(gain_data[3].iter())
                .for_each(|((d, g0), g1)| {
                    assert_eq!(d[0], g0.phase().value());
                    assert_eq!(d[1], g1.phase().value());
                });
        }

        // Final frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );

            assert_eq!(op.sent, GAIN_STM_SIZE);

            assert_eq!(TypeTag::GainSTM as u8, tx[0]);
            assert_eq!(
                (GainSTMControlFlags::END | GainSTMControlFlags::SEGMENT).bits(),
                tx[offset_of!(GainSTMHead, flag)] & 0x3F
            );
            assert_eq!(0, tx[offset_of!(GainSTMHead, flag)] >> 6);
            tx[size_of::<GainSTMSubseq>()..]
                .chunks(size_of::<Drive>())
                .zip(gain_data[4].iter())
                .for_each(|(d, g)| {
                    assert_eq!(d[0], g.phase().value());
                })
        }
    }

    #[test]
    fn test_phase_half() {
        const GAIN_STM_SIZE: usize = 9;
        const FRAME_SIZE: usize = size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::thread_rng();

        let gain_data: Vec<Vec<Drive>> = (0..GAIN_STM_SIZE)
            .map(|_| {
                (0..NUM_TRANS_IN_UNIT)
                    .map(|_| {
                        Drive::new(
                            Phase::new(rng.gen_range(0x00..=0xFF)),
                            EmitIntensity::new(rng.gen_range(0..=0xFF)),
                        )
                    })
                    .collect()
            })
            .collect();

        let freq_div = rng.gen_range(0x0001..=0xFFFF);
        let rep = rng.gen_range(0x001..=0xFFFF);
        let segment = Segment::S0;
        let mut op = GainSTMOp::new(
            {
                let gain_data = gain_data.clone();
                gain_data
                    .into_iter()
                    .map(|g| Box::new(move |tr: &Transducer| g[tr.idx()]) as Box<_>)
                    .collect()
            },
            GainSTMMode::PhaseHalf,
            SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            LoopBehavior { rep },
            segment,
            Some(TransitionMode::SyncIdx),
        );

        assert_eq!(op.sent, 0);

        // First frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2)
            );

            assert_eq!(op.sent, 4);

            assert_eq!(TypeTag::GainSTM as u8, tx[0]);
            assert_eq!(
                GainSTMControlFlags::BEGIN.bits(),
                tx[offset_of!(GainSTMHead, flag)] & 0x3F
            );
            assert_eq!(3, tx[offset_of!(GainSTMHead, flag)] >> 6);
            assert_eq!(
                GainSTMMode::PhaseHalf as u8,
                tx[offset_of!(GainSTMHead, mode)]
            );

            tx[size_of::<GainSTMHead>()..]
                .chunks(size_of::<Drive>())
                .zip(gain_data[0].iter())
                .zip(gain_data[1].iter())
                .zip(gain_data[2].iter())
                .zip(gain_data[3].iter())
                .for_each(|((((d, g0), g1), g2), g3)| {
                    assert_eq!(d[0] & 0x0F, g0.phase().value() >> 4);
                    assert_eq!(d[0] >> 4, g1.phase().value() >> 4);
                    assert_eq!(d[1] & 0x0F, g2.phase().value() >> 4);
                    assert_eq!(d[1] >> 4, g3.phase().value() >> 4);
                });
        }

        // Second frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );

            assert_eq!(op.sent, 8);

            assert_eq!(TypeTag::GainSTM as u8, tx[0]);
            assert_eq!(
                GainSTMControlFlags::NONE.bits(),
                tx[offset_of!(GainSTMHead, flag)] & 0x3F
            );
            assert_eq!(3, tx[offset_of!(GainSTMHead, flag)] >> 6);
            tx[size_of::<GainSTMSubseq>()..]
                .chunks(size_of::<Drive>())
                .zip(gain_data[4].iter())
                .zip(gain_data[5].iter())
                .zip(gain_data[6].iter())
                .zip(gain_data[7].iter())
                .for_each(|((((d, g0), g1), g2), g3)| {
                    assert_eq!(d[0] & 0x0F, g0.phase().value() >> 4);
                    assert_eq!(d[0] >> 4, g1.phase().value() >> 4);
                    assert_eq!(d[1] & 0x0F, g2.phase().value() >> 4);
                    assert_eq!(d[1] >> 4, g3.phase().value() >> 4);
                });
        }

        // Final frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );

            assert_eq!(op.sent, GAIN_STM_SIZE);

            assert_eq!(TypeTag::GainSTM as u8, tx[0]);
            assert_eq!(
                (GainSTMControlFlags::END | GainSTMControlFlags::TRANSITION).bits(),
                tx[offset_of!(GainSTMHead, flag)] & 0x3F
            );
            assert_eq!(0, tx[offset_of!(GainSTMHead, flag)] >> 6);
            tx[size_of::<GainSTMSubseq>()..]
                .chunks(size_of::<Drive>())
                .zip(gain_data[8].iter())
                .for_each(|(d, g)| {
                    assert_eq!(d[0] & 0x0F, g.phase().value() >> 4);
                });
        }
    }

    #[rstest::rstest]
    #[test]
    #[case(Err(AUTDInternalError::GainSTMSizeOutOfRange(0)), 0)]
    #[case(Err(AUTDInternalError::GainSTMSizeOutOfRange(STM_BUF_SIZE_MIN-1)), STM_BUF_SIZE_MIN-1)]
    #[case(Ok(()), STM_BUF_SIZE_MIN)]
    #[case(Ok(()), GAIN_STM_BUF_SIZE_MAX)]
    #[case(
        Err(AUTDInternalError::GainSTMSizeOutOfRange(GAIN_STM_BUF_SIZE_MAX+1)),
        GAIN_STM_BUF_SIZE_MAX+1
    )]
    #[cfg_attr(miri, ignore)]
    fn out_of_range(#[case] expected: Result<(), AUTDInternalError>, #[case] size: usize) {
        let send = |n: usize| {
            const FRAME_SIZE: usize = size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;
            let device = create_device(0, NUM_TRANS_IN_UNIT);
            let mut tx = vec![0x00u8; FRAME_SIZE];
            let gain_data: Vec<Vec<Drive>> = vec![vec![Drive::null(); NUM_TRANS_IN_UNIT]; n];
            let mut op = GainSTMOp::new(
                {
                    let gain_data = gain_data.clone();
                    gain_data
                        .into_iter()
                        .map(|g| Box::new(move |tr: &Transducer| g[tr.idx()]) as Box<_>)
                        .collect()
                },
                GainSTMMode::PhaseIntensityFull,
                SamplingConfig::FREQ_40K,
                LoopBehavior::infinite(),
                Segment::S0,
                None,
            );
            loop {
                op.pack(&device, &mut tx)?;
                if op.is_done() {
                    break;
                }
            }
            Result::<(), AUTDInternalError>::Ok(())
        };
        assert_eq!(expected, send(size));
    }
}
