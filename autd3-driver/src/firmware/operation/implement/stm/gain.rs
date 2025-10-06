#![allow(clippy::type_complexity)]

use std::mem::size_of;

use crate::{
    datagram::{GainSTMIterator, GainSTMIteratorGenerator, GainSTMMode, GainSTMOperationGenerator},
    error::AUTDDriverError,
    firmware::{
        operation::{Operation, OperationGenerator, implement::null::NullOp},
        tag::TypeTag,
    },
};

use autd3_core::{
    firmware::{
        Drive, GAIN_STM_BUF_SIZE_MAX, STM_BUF_SIZE_MIN, SamplingConfig, Segment,
        transition_mode::{Later, TransitionMode, TransitionModeParams},
    },
    gain::{GainCalculator, GainCalculatorGenerator},
    geometry::Device,
};

#[derive(Clone, Copy)]
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

struct PhaseFull {
    phase_0: u8,
    phase_1: u8,
}

impl PhaseFull {
    fn mut_from_bytes(bytes: &mut [u8]) -> &mut Self {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut Self) }
    }
}

struct PhaseHalf(u16);

impl PhaseHalf {
    const fn set_phase_0(&mut self, phase: u8) {
        self.0 = (self.0 & !0x000F) | ((phase as u16) & 0x000F);
    }

    const fn set_phase_1(&mut self, phase: u8) {
        self.0 = (self.0 & !0x00F0) | (((phase as u16) << 4) & 0x00F0);
    }

    const fn set_phase_2(&mut self, phase: u8) {
        self.0 = (self.0 & !0x0F00) | (((phase as u16) << 8) & 0x0F00);
    }

    const fn set_phase_3(&mut self, phase: u8) {
        self.0 = (self.0 & !0xF000) | (((phase as u16) << 12) & 0xF000);
    }

    fn mut_from_bytes(bytes: &mut [u8]) -> &mut Self {
        unsafe { &mut *(bytes.as_mut_ptr() as *mut Self) }
    }
}

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

pub struct GainSTMOp<G, Iterator> {
    iter: Iterator,
    size: usize,
    sent: usize,
    mode: GainSTMMode,
    config: SamplingConfig,
    rep: u16,
    segment: Segment,
    transition_params: TransitionModeParams,
    __phantom: std::marker::PhantomData<G>,
}

impl<'a, G: GainCalculator<'a>, Iterator: GainSTMIterator<'a, Calculator = G>>
    GainSTMOp<G, Iterator>
{
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        iter: Iterator,
        size: usize,
        mode: GainSTMMode,
        config: SamplingConfig,
        rep: u16,
        segment: Segment,
        transition_params: TransitionModeParams,
    ) -> Self {
        Self {
            iter,
            size,
            sent: 0,
            mode,
            config,
            rep,
            segment,
            transition_params,
            __phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, G: GainCalculator<'a>, Iterator: GainSTMIterator<'a, Calculator = G>> Operation<'a>
    for GainSTMOp<G, Iterator>
{
    type Error = AUTDDriverError;

    fn required_size(&self, device: &'a Device) -> usize {
        if self.sent == 0 {
            size_of::<GainSTMHead>() + device.num_transducers() * size_of::<Drive>()
        } else {
            size_of::<GainSTMSubseq>() + device.num_transducers() * size_of::<Drive>()
        }
    }

    fn pack(&mut self, device: &'a Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        if !(STM_BUF_SIZE_MIN..=GAIN_STM_BUF_SIZE_MAX).contains(&self.size) {
            return Err(AUTDDriverError::GainSTMSizeOutOfRange(self.size));
        }

        let is_first = self.sent == 0;

        let send = {
            let offset = if is_first {
                size_of::<GainSTMHead>()
            } else {
                size_of::<GainSTMSubseq>()
            };

            let mut send = 0;
            match self.mode {
                GainSTMMode::PhaseIntensityFull => {
                    if let Some(g) = self.iter.next() {
                        tx[offset..]
                            .chunks_mut(size_of::<Drive>())
                            .zip(device.iter())
                            .for_each(|(dst, tr)| {
                                crate::firmware::operation::write_to_tx(dst, g.calc(tr));
                            });
                        send += 1;
                    } // GRCOV_EXCL_LINE
                }
                GainSTMMode::PhaseFull => {
                    if let Some(g) = self.iter.next() {
                        tx[offset..]
                            .chunks_exact_mut(size_of::<PhaseFull>())
                            .zip(device.iter())
                            .for_each(|(dst, tr)| {
                                PhaseFull::mut_from_bytes(dst).phase_0 = g.calc(tr).phase.0;
                            });
                        send += 1;
                    } // GRCOV_EXCL_LINE
                    if let Some(g) = self.iter.next() {
                        tx[offset..]
                            .chunks_exact_mut(size_of::<PhaseFull>())
                            .zip(device.iter())
                            .for_each(|(dst, tr)| {
                                PhaseFull::mut_from_bytes(dst).phase_1 = g.calc(tr).phase.0;
                            });
                        send += 1;
                    }
                }
                GainSTMMode::PhaseHalf => {
                    if let Some(g) = self.iter.next() {
                        tx[offset..]
                            .chunks_exact_mut(size_of::<PhaseHalf>())
                            .zip(device.iter())
                            .for_each(|(dst, tr)| {
                                PhaseHalf::mut_from_bytes(dst).set_phase_0(g.calc(tr).phase.0 >> 4);
                            });
                        send += 1;
                    } // GRCOV_EXCL_LINE
                    if let Some(g) = self.iter.next() {
                        tx[offset..]
                            .chunks_exact_mut(size_of::<PhaseHalf>())
                            .zip(device.iter())
                            .for_each(|(dst, tr)| {
                                PhaseHalf::mut_from_bytes(dst).set_phase_1(g.calc(tr).phase.0 >> 4);
                            });
                        send += 1;
                    }
                    if let Some(g) = self.iter.next() {
                        tx[offset..]
                            .chunks_exact_mut(size_of::<PhaseHalf>())
                            .zip(device.iter())
                            .for_each(|(dst, tr)| {
                                PhaseHalf::mut_from_bytes(dst).set_phase_2(g.calc(tr).phase.0 >> 4);
                            });
                        send += 1;
                    }
                    if let Some(g) = self.iter.next() {
                        tx[offset..]
                            .chunks_exact_mut(size_of::<PhaseHalf>())
                            .zip(device.iter())
                            .for_each(|(dst, tr)| {
                                PhaseHalf::mut_from_bytes(dst).set_phase_3(g.calc(tr).phase.0 >> 4);
                            });
                        send += 1;
                    }
                }
            }
            send
        };

        self.sent += send;

        let mut flag = if self.sent == self.size {
            GainSTMControlFlags::END
                | if self.transition_params != Later.params() {
                    GainSTMControlFlags::TRANSITION
                } else {
                    GainSTMControlFlags::NONE
                }
        } else {
            GainSTMControlFlags::NONE
        };

        flag.set(GainSTMControlFlags::SEGMENT, self.segment == Segment::S1);
        flag.set(
            GainSTMControlFlags::SEND_BIT0,
            ((send as u8 - 1) & 0x01) != 0,
        );
        flag.set(
            GainSTMControlFlags::SEND_BIT1,
            ((send as u8 - 1) & 0x02) != 0,
        );

        if is_first {
            crate::firmware::operation::write_to_tx(
                tx,
                GainSTMHead {
                    tag: TypeTag::GainSTM,
                    flag: GainSTMControlFlags::BEGIN | flag,
                    mode: self.mode,
                    transition_mode: self.transition_params.mode,
                    transition_value: self.transition_params.value,
                    freq_div: self.config.divide()?,
                    rep: self.rep,
                },
            );
        } else {
            crate::firmware::operation::write_to_tx(
                tx,
                GainSTMSubseq {
                    tag: TypeTag::GainSTM,
                    flag,
                },
            );
        }

        if is_first {
            Ok(size_of::<GainSTMHead>() + device.num_transducers() * size_of::<Drive>())
        } else {
            Ok(size_of::<GainSTMSubseq>() + device.num_transducers() * size_of::<Drive>())
        }
    }

    fn is_done(&self) -> bool {
        self.sent == self.size
    }
}

impl<'a, G: GainSTMIteratorGenerator<'a>> OperationGenerator<'a>
    for GainSTMOperationGenerator<'a, G>
{
    type O1 = GainSTMOp<<G::Gain as GainCalculatorGenerator<'a>>::Calculator, G::Iterator>;
    type O2 = NullOp;

    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::new(
                self.g.generate(device),
                self.size,
                self.mode,
                self.sampling_config,
                self.rep,
                self.segment,
                self.transition_params,
            ),
            Self::O2 {},
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{collections::VecDeque, mem::offset_of, num::NonZeroU16};

    use crate::geometry::Transducer;

    use autd3_core::{
        derive::transition_mode,
        ethercat::DcSysTime,
        firmware::{Intensity, Phase},
        link::TxMessage,
    };

    use rand::Rng;

    struct Impl {
        g: Vec<Drive>,
    }

    impl GainCalculator<'_> for Impl {
        fn calc(&self, tr: &Transducer) -> Drive {
            self.g[tr.idx()]
        }
    }

    struct STMIterator {
        data: VecDeque<Vec<Drive>>,
    }

    impl GainSTMIterator<'_> for STMIterator {
        type Calculator = Impl;

        fn next(&mut self) -> Option<Impl> {
            self.data.pop_front().map(|g| Impl { g })
        }
    }

    #[test]
    fn test_phase_intensity_full() {
        let device = crate::autd3_device::tests::create_device();

        const GAIN_STM_SIZE: usize = 3;
        let frame_size = size_of::<GainSTMHead>() + device.num_transducers() * 2;

        let mut tx = vec![0x00u8; frame_size];

        let mut rng = rand::rng();

        let gain_data: VecDeque<Vec<Drive>> = (0..GAIN_STM_SIZE)
            .map(|_| {
                (0..device.num_transducers())
                    .map(|_| Drive {
                        phase: Phase(rng.random_range(0x00..=0xFF)),
                        intensity: Intensity(rng.random_range(0..=0xFF)),
                    })
                    .collect()
            })
            .collect();

        let freq_div = rng.random_range(0x0001..=0xFFFF);
        let rep = rng.random_range(0x0000..0xFFFF);
        let segment = Segment::S0;
        let transition_value = 0x0123456789ABCDEF;
        let transition_mode = transition_mode::SysTime(
            DcSysTime::ZERO + std::time::Duration::from_nanos(transition_value),
        );

        let mut op = GainSTMOp::new(
            {
                STMIterator {
                    data: gain_data.clone(),
                }
            },
            GAIN_STM_SIZE,
            GainSTMMode::PhaseIntensityFull,
            SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            rep,
            segment,
            transition_mode.params(),
        );

        // First frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMHead>() + device.num_transducers() * 2
            );

            assert_eq!(op.sent, 0);

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMHead>() + device.num_transducers() * 2)
            );

            assert_eq!(op.sent, 1);

            assert_eq!(TypeTag::GainSTM as u8, tx[0]);
            assert_eq!(GainSTMControlFlags::BEGIN.bits(), tx[1] & 0x3F);
            assert_eq!(0, tx[1] >> 6);
            assert_eq!(GainSTMMode::PhaseIntensityFull as u8, tx[2]);
            assert_eq!(transition_mode.params().mode, tx[3]);
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
                    assert_eq!(d[0], g.phase.0);
                    assert_eq!(d[1], g.intensity.0);
                })
        }

        // Second frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + device.num_transducers() * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMSubseq>() + device.num_transducers() * 2)
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
                    assert_eq!(d[0], g.phase.0);
                    assert_eq!(d[1], g.intensity.0);
                })
        }

        // Final frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + device.num_transducers() * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMSubseq>() + device.num_transducers() * 2)
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
                    assert_eq!(d[0], g.phase.0);
                    assert_eq!(d[1], g.intensity.0);
                })
        }
    }

    #[test]
    fn test_phase_full() {
        let device = crate::autd3_device::tests::create_device();

        const GAIN_STM_SIZE: usize = 5;
        let frame_size = size_of::<GainSTMHead>() + device.num_transducers() * 2;

        let mut tx = vec![0x00u8; frame_size];

        let mut rng = rand::rng();

        let gain_data: VecDeque<Vec<Drive>> = (0..GAIN_STM_SIZE)
            .map(|_| {
                (0..device.num_transducers())
                    .map(|_| Drive {
                        phase: Phase(rng.random_range(0x00..=0xFF)),
                        intensity: Intensity(rng.random_range(0..=0xFF)),
                    })
                    .collect()
            })
            .collect();

        let freq_div = rng.random_range(0x0001..=0xFFFF);
        let rep = rng.random_range(0x0000..0xFFFF);
        let segment = Segment::S1;
        let mut op = GainSTMOp::<_, _>::new(
            STMIterator {
                data: gain_data.clone(),
            },
            GAIN_STM_SIZE,
            GainSTMMode::PhaseFull,
            SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            rep,
            segment,
            Later.params(),
        );

        assert_eq!(op.sent, 0);

        // First frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMHead>() + device.num_transducers() * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMHead>() + device.num_transducers() * 2)
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
                    assert_eq!(d[0], g0.phase.0);
                    assert_eq!(d[1], g1.phase.0);
                });
        }

        // Second frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + device.num_transducers() * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMSubseq>() + device.num_transducers() * 2)
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
                    assert_eq!(d[0], g0.phase.0);
                    assert_eq!(d[1], g1.phase.0);
                });
        }

        // Final frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + device.num_transducers() * 2
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<GainSTMSubseq>() + device.num_transducers() * 2)
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
                    assert_eq!(d[0], g.phase.0);
                })
        }
    }

    #[test]
    fn test_phase_half() {
        const GAIN_STM_SIZE: usize = 9;

        let device = crate::autd3_device::tests::create_device();

        let mut tx = vec![TxMessage::new(); 1];
        let tx = tx[0].payload_mut();

        let mut rng = rand::rng();

        let gain_data: VecDeque<Vec<Drive>> = (0..GAIN_STM_SIZE)
            .map(|_| {
                (0..device.num_transducers())
                    .map(|_| Drive {
                        phase: Phase(rng.random_range(0x00..=0xFF)),
                        intensity: Intensity(rng.random_range(0..=0xFF)),
                    })
                    .collect()
            })
            .collect();

        let freq_div = rng.random_range(0x0001..=0xFFFF);
        let rep = rng.random_range(0x0000..0xFFFF);
        let segment = Segment::S0;
        let mut op = GainSTMOp::new(
            STMIterator {
                data: gain_data.clone(),
            },
            GAIN_STM_SIZE,
            GainSTMMode::PhaseHalf,
            SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            rep,
            segment,
            transition_mode::SyncIdx.params(),
        );

        assert_eq!(op.sent, 0);

        // First frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMHead>() + device.num_transducers() * 2
            );

            assert_eq!(
                op.pack(&device, tx),
                Ok(size_of::<GainSTMHead>() + device.num_transducers() * 2)
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
                    assert_eq!(d[0] & 0x0F, g0.phase.0 >> 4);
                    assert_eq!(d[0] >> 4, g1.phase.0 >> 4);
                    assert_eq!(d[1] & 0x0F, g2.phase.0 >> 4);
                    assert_eq!(d[1] >> 4, g3.phase.0 >> 4);
                });
        }

        // Second frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + device.num_transducers() * 2
            );

            assert_eq!(
                op.pack(&device, tx),
                Ok(size_of::<GainSTMSubseq>() + device.num_transducers() * 2)
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
                    assert_eq!(d[0] & 0x0F, g0.phase.0 >> 4);
                    assert_eq!(d[0] >> 4, g1.phase.0 >> 4);
                    assert_eq!(d[1] & 0x0F, g2.phase.0 >> 4);
                    assert_eq!(d[1] >> 4, g3.phase.0 >> 4);
                });
        }

        // Final frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<GainSTMSubseq>() + device.num_transducers() * 2
            );

            assert_eq!(
                op.pack(&device, tx),
                Ok(size_of::<GainSTMSubseq>() + device.num_transducers() * 2)
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
                    assert_eq!(d[0] & 0x0F, g.phase.0 >> 4);
                });
        }
    }

    #[rstest::rstest]
    #[case(Err(AUTDDriverError::GainSTMSizeOutOfRange(0)), 0)]
    #[case(Err(AUTDDriverError::GainSTMSizeOutOfRange(STM_BUF_SIZE_MIN - 1)), STM_BUF_SIZE_MIN-1)]
    #[case(Ok(()), STM_BUF_SIZE_MIN)]
    #[case(Ok(()), GAIN_STM_BUF_SIZE_MAX)]
    #[case(
        Err(AUTDDriverError::GainSTMSizeOutOfRange(GAIN_STM_BUF_SIZE_MAX + 1)),
        GAIN_STM_BUF_SIZE_MAX + 1
    )]
    fn out_of_range(#[case] expected: Result<(), AUTDDriverError>, #[case] size: usize) {
        let send = |n: usize| {
            let device = crate::autd3_device::tests::create_device();

            let frame_size = size_of::<GainSTMHead>() + device.num_transducers() * 2;

            let mut tx = vec![0x00u8; frame_size];
            let data = (0..n)
                .map(|_| vec![Drive::NULL; device.num_transducers()])
                .collect();
            let mut op = GainSTMOp::new(
                STMIterator { data },
                n,
                GainSTMMode::PhaseIntensityFull,
                SamplingConfig::FREQ_40K,
                0xFFFF,
                Segment::S0,
                Later.params(),
            );
            loop {
                op.pack(&device, &mut tx)?;
                if op.is_done() {
                    break;
                }
            }
            Result::<(), AUTDDriverError>::Ok(())
        };
        assert_eq!(expected, send(size));
    }
}
