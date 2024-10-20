#![allow(clippy::type_complexity)]

use std::{iter::Peekable, mem::size_of};

use crate::{
    derive::{LoopBehavior, SamplingConfig},
    error::AUTDInternalError,
    firmware::{
        cpu::GainSTMMode,
        fpga::{
            Drive, Segment, TransitionMode, GAIN_STM_BUF_SIZE_MAX, STM_BUF_SIZE_MIN,
            TRANSITION_MODE_NONE,
        },
        operation::{Operation, TypeTag},
    },
    geometry::Device,
};

use super::GainContext;

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[derive(Clone, Copy, IntoBytes, Immutable)]
#[repr(C)]
pub struct GainSTMControlFlags(u8);

bitflags::bitflags! {
    impl GainSTMControlFlags : u8 {
        const NONE       = 0;
        const BEGIN      = 1 << 0;
        const END        = 1 << 1;
        const TRANSITION = 1 << 2;
        const SEGMENT    = 1 << 3;
        const SEND_BIT0  = 1 << 6;
        const SEND_BIT1  = 1 << 7;
    }
}

#[derive(IntoBytes, Immutable, FromBytes, KnownLayout)]
struct PhaseFull {
    phase_0: u8,
    phase_1: u8,
}

#[bitfield_struct::bitfield(u16)]
#[derive(IntoBytes, Immutable, FromBytes, KnownLayout)]
struct PhaseHalf {
    #[bits(4)]
    phase_0: u8,
    #[bits(4)]
    phase_1: u8,
    #[bits(4)]
    phase_2: u8,
    #[bits(4)]
    phase_3: u8,
}

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
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
#[derive(IntoBytes, Immutable)]
struct GainSTMSubseq {
    tag: TypeTag,
    flag: GainSTMControlFlags,
}

pub struct GainSTMOp<Context: GainContext> {
    gains: Peekable<std::vec::IntoIter<Context>>,
    sent: usize,
    is_done: bool,
    mode: GainSTMMode,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<Context: GainContext> GainSTMOp<Context> {
    pub(crate) fn new(
        gains: Vec<Context>,
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

impl<Context: GainContext> Operation for GainSTMOp<Context> {
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

        let send = {
            let mut send = 0;
            match self.mode {
                GainSTMMode::PhaseIntensityFull => {
                    if let Some(g) = self.gains.next() {
                        tx[offset..]
                            .chunks_mut(size_of::<Drive>())
                            .zip(device.iter())
                            .for_each(|(dst, tr)| {
                                dst.copy_from_slice(g.calc(tr).as_bytes());
                            });
                        send += 1;
                    }
                }
                GainSTMMode::PhaseFull => {
                    seq_macro::seq!(N in 0..2 {
                        #(
                            if let Some(g) = self.gains.next() {
                                tx[offset..].chunks_exact_mut(size_of::<PhaseFull>()).zip(device.iter()).for_each(|(dst, tr)| {
                                    PhaseFull::mut_from_bytes(dst).unwrap().phase_~N = g.calc(tr).phase().value();
                                });
                                send += 1;
                            }
                        )*
                    });
                }
                GainSTMMode::PhaseHalf => {
                    seq_macro::seq!(N in 0..4 {
                        #(
                            if let Some(g) = self.gains.next() {
                                tx[offset..].chunks_exact_mut(size_of::<PhaseHalf>()).zip(device.iter()).for_each(|(dst, tr)| {
                                    PhaseHalf::mut_from_bytes(dst).unwrap().set_phase_~N(g.calc(tr).phase().value() >> 4);
                                });
                                send += 1;
                            }
                        )*
                    });
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
            tx[..size_of::<GainSTMHead>()].copy_from_slice(
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
                }
                .as_bytes(),
            );
        } else {
            tx[..size_of::<GainSTMSubseq>()].copy_from_slice(
                GainSTMSubseq {
                    tag: TypeTag::GainSTM,
                    flag,
                }
                .as_bytes(),
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
    use std::mem::offset_of;

    use rand::prelude::*;

    use super::*;
    use crate::{
        derive::Transducer,
        ethercat::DcSysTime,
        firmware::{
            cpu::TxDatagram,
            fpga::{EmitIntensity, Phase},
        },
        geometry::tests::create_device,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;

    struct Context {
        g: Vec<Drive>,
    }

    impl GainContext for Context {
        fn calc(&self, tr: &Transducer) -> Drive {
            self.g[tr.idx()]
        }
    }

    #[test]
    fn test_phase_intensity_full() {
        const GAIN_STM_SIZE: usize = 3;
        const FRAME_SIZE: usize = size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;

        let device = create_device(0, NUM_TRANS_IN_UNIT as _);

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
                gain_data.into_iter().map(|g| Context { g }).collect()
            },
            GainSTMMode::PhaseIntensityFull,
            SamplingConfig::new(freq_div).unwrap(),
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
            assert_eq!(GainSTMControlFlags::BEGIN.bits(), tx[1] & 0x3F);
            assert_eq!(0, tx[1] >> 6);
            assert_eq!(GainSTMMode::PhaseIntensityFull as u8, tx[2]);
            assert_eq!(transition_mode.mode(), tx[3]);
            assert_eq!(freq_div as u8, tx[4]);
            assert_eq!((freq_div >> 8) as u8, tx[5]);
            assert_eq!(rep as u8, tx[6]);
            assert_eq!((rep >> 8) as u8, tx[7]);
            assert_eq!(transition_value as u8, tx[8]);
            assert_eq!((transition_value >> 8) as u8, tx[9]);
            assert_eq!((transition_value >> 16) as u8, tx[10]);
            assert_eq!((transition_value >> 24) as u8, tx[11]);
            assert_eq!((transition_value >> 32) as u8, tx[12]);
            assert_eq!((transition_value >> 40) as u8, tx[13]);
            assert_eq!((transition_value >> 48) as u8, tx[14]);
            assert_eq!((transition_value >> 56) as u8, tx[15]);
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

        let device = create_device(0, NUM_TRANS_IN_UNIT as _);

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
                gain_data.into_iter().map(|g| Context { g }).collect()
            },
            GainSTMMode::PhaseFull,
            SamplingConfig::new(freq_div).unwrap(),
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

        let device = create_device(0, NUM_TRANS_IN_UNIT as _);

        let mut tx = TxDatagram::new(1);
        let tx = tx[0].payload_mut();

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
                gain_data.into_iter().map(|g| Context { g }).collect()
            },
            GainSTMMode::PhaseHalf,
            SamplingConfig::new(freq_div).unwrap(),
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
                op.pack(&device, tx),
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
                op.pack(&device, tx),
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
                op.pack(&device, tx),
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
    fn out_of_range(#[case] expected: Result<(), AUTDInternalError>, #[case] size: usize) {
        let send = |n: usize| {
            const FRAME_SIZE: usize = size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;
            let device = create_device(0, NUM_TRANS_IN_UNIT as _);
            let mut tx = vec![0x00u8; FRAME_SIZE];
            let gain_data: Vec<Vec<Drive>> = vec![vec![Drive::NULL; NUM_TRANS_IN_UNIT]; n];
            let mut op = GainSTMOp::new(
                {
                    let gain_data = gain_data.clone();
                    gain_data.into_iter().map(|g| Context { g }).collect()
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
