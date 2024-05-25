use std::{collections::HashMap, mem::size_of};

use crate::{
    datagram::{Gain, GainFilter},
    derive::Transducer,
    error::AUTDInternalError,
    firmware::{
        cpu::GainSTMMode,
        fpga::{
            Drive, LoopBehavior, STMSamplingConfig, Segment, TransitionMode, GAIN_STM_BUF_SIZE_MAX,
            STM_BUF_SIZE_MIN, TRANSITION_MODE_NONE,
        },
        operation::{cast, Operation, TypeTag},
    },
    geometry::{Device, Geometry},
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
    __padding: [u8; 4],
    freq_div: u32,
    rep: u32,
    transition_value: u64,
}

#[repr(C, align(2))]
struct GainSTMSubseq {
    tag: TypeTag,
    flag: GainSTMControlFlags,
}

pub struct GainSTMOp<'a, F: Fn(usize) -> Box<dyn Fn(&Transducer) -> Drive + Send + Sync + 'a>> {
    gains: F,
    size: usize,
    remains: usize,
    mode: GainSTMMode,
    freq_div: u32,
    rep: u32,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<'a, F: Fn(usize) -> Box<dyn Fn(&Transducer) -> Drive + Send + Sync + 'a>> GainSTMOp<'a, F> {
    pub fn new(
        gains: F,
        size: usize,
        mode: GainSTMMode,
        freq_div: u32,
        rep: u32,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Self {
        Self {
            gains,
            size,
            remains: size,
            mode,
            freq_div,
            rep,
            segment,
            transition_mode,
        }
    }
}

impl<'a, F: Fn(usize) -> Box<dyn Fn(&Transducer) -> Drive + Send + Sync + 'a>> Operation
    for GainSTMOp<'a, F>
{
    fn required_size(&self, device: &Device) -> usize {
        if self.remains == self.size {
            size_of::<GainSTMHead>() + device.num_transducers() * size_of::<Drive>()
        } else {
            size_of::<GainSTMSubseq>() + device.num_transducers() * size_of::<Drive>()
        }
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let sent = self.size - self.remains;

        let offset = if sent == 0 {
            size_of::<GainSTMHead>()
        } else {
            size_of::<GainSTMSubseq>()
        };
        assert!(tx.len() >= offset + device.num_transducers() * size_of::<Drive>());

        let mut send = 0;
        match self.mode {
            GainSTMMode::PhaseIntensityFull => {
                unsafe {
                    let dst = std::slice::from_raw_parts_mut(
                        tx[offset..].as_mut_ptr() as *mut Drive,
                        device.len(),
                    );
                    dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                        *d = (self.gains)(sent)(tr);
                    });
                }
                send += 1;
            }
            GainSTMMode::PhaseFull => unsafe {
                let dst = std::slice::from_raw_parts_mut(
                    tx[offset..].as_mut_ptr() as *mut PhaseFull<0>,
                    device.len(),
                );
                dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                    d.set((self.gains)(sent)(tr));
                });
                send += 1;

                if self.size > sent + 1 {
                    let dst = std::slice::from_raw_parts_mut(
                        tx[offset..].as_mut_ptr() as *mut PhaseFull<1>,
                        device.len(),
                    );
                    dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                        d.set((self.gains)(sent)(tr));
                    });
                }
                send += 1;
            },
            GainSTMMode::PhaseHalf => unsafe {
                let dst = std::slice::from_raw_parts_mut(
                    tx[offset..].as_mut_ptr() as *mut PhaseHalf<0>,
                    device.len(),
                );
                dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                    d.set((self.gains)(sent)(tr));
                });
                send += 1;

                if self.size > sent + 1 {
                    let dst = std::slice::from_raw_parts_mut(
                        tx[offset..].as_mut_ptr() as *mut PhaseHalf<1>,
                        device.len(),
                    );
                    dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                        d.set((self.gains)(sent)(tr));
                    });
                    send += 1;
                }
                if self.size > sent + 2 {
                    let dst = std::slice::from_raw_parts_mut(
                        tx[offset..].as_mut_ptr() as *mut PhaseHalf<1>,
                        device.len(),
                    );
                    dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                        d.set((self.gains)(sent)(tr));
                    });
                    send += 1;
                }
                if self.size > sent + 3 {
                    let dst = std::slice::from_raw_parts_mut(
                        tx[offset..].as_mut_ptr() as *mut PhaseHalf<1>,
                        device.len(),
                    );
                    dst.iter_mut().zip(device.iter()).for_each(|(d, tr)| {
                        d.set((self.gains)(sent)(tr));
                    });
                    send += 1;
                }
            },
        }

        if sent == 0 {
            *cast::<GainSTMHead>(tx) = GainSTMHead {
                tag: TypeTag::GainSTM,
                flag: GainSTMControlFlags::BEGIN,
                mode: self.mode,
                transition_mode: self
                    .transition_mode
                    .map(|m| m.mode())
                    .unwrap_or(TRANSITION_MODE_NONE),
                transition_value: self.transition_mode.map(|m| m.value()).unwrap_or(0),
                freq_div: self.freq_div,
                rep: self.rep,
                __padding: [0; 4],
            };
        } else {
            *cast::<GainSTMSubseq>(tx) = GainSTMSubseq {
                tag: TypeTag::GainSTM,
                flag: GainSTMControlFlags::NONE,
            };
        }

        cast::<GainSTMSubseq>(tx)
            .flag
            .set(GainSTMControlFlags::SEGMENT, self.segment == Segment::S1);

        let d = cast::<GainSTMSubseq>(tx);
        d.flag.set(
            GainSTMControlFlags::SEND_BIT0,
            ((send as u8 - 1) & 0x01) != 0,
        );
        d.flag.set(
            GainSTMControlFlags::SEND_BIT1,
            ((send as u8 - 1) & 0x02) != 0,
        );

        if sent + send == self.size {
            d.flag.set(GainSTMControlFlags::END, true);
            d.flag.set(
                GainSTMControlFlags::TRANSITION,
                self.transition_mode.is_some(),
            );
        }

        self.remains -= send;
        if sent == 0 {
            Ok(size_of::<GainSTMHead>() + device.num_transducers() * size_of::<Drive>())
        } else {
            Ok(size_of::<GainSTMSubseq>() + device.num_transducers() * size_of::<Drive>())
        }
    }

    fn is_done(&self) -> bool {
        self.remains == 0
    }
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;

    use rand::prelude::*;

    use super::*;
    use crate::{
        defined::FREQ_40K,
        ethercat::DcSysTime,
        firmware::{
            fpga::{EmitIntensity, Phase, SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
            operation::tests::parse_tx_as,
        },
        geometry::tests::create_geometry,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    // #[test]
    // fn test_phase_intensity_full() {
    //     const GAIN_STM_SIZE: usize = 3;
    //     const FRAME_SIZE: usize = size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;

    //     let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

    //     let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

    //     let mut rng = rand::thread_rng();

    //     let gain_data: Vec<HashMap<usize, Vec<Drive>>> = (0..GAIN_STM_SIZE)
    //         .map(|_| {
    //             geometry
    //                 .devices()
    //                 .map(|dev| {
    //                     (
    //                         dev.idx(),
    //                         (0..dev.num_transducers())
    //                             .map(|_| {
    //                                 Drive::new(
    //                                     Phase::new(rng.gen_range(0x00..=0xFF)),
    //                                     EmitIntensity::new(rng.gen_range(0..=0xFF)),
    //                                 )
    //                             })
    //                             .collect(),
    //                     )
    //                 })
    //                 .collect()
    //         })
    //         .collect();
    //     let gains: Vec<TestGain> = (0..GAIN_STM_SIZE)
    //         .map(|i| TestGain {
    //             data: gain_data[i].clone(),
    //         })
    //         .collect();

    //     let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
    //     let loop_behavior = LoopBehavior::infinite();
    //     let rep = loop_behavior.rep;
    //     let segment = Segment::S0;
    //     let transition_value = 0x0123456789ABCDEF;
    //     let transition_mode = TransitionMode::SysTime(
    //         DcSysTime::from_utc(
    //             time::macros::datetime!(2000-01-01 0:00 UTC)
    //                 + std::time::Duration::from_nanos(transition_value),
    //         )
    //         .unwrap(),
    //     );

    //     let mut op = GainSTMOp::<_>::new(
    //         gains,
    //         GainSTMMode::PhaseIntensityFull,
    //         STMSamplingConfig::SamplingConfig(crate::derive::SamplingConfig::DivisionRaw(freq_div)),
    //         loop_behavior,
    //         segment,
    //         Some(transition_mode),
    //     );

    //     assert!(op.init(&geometry).is_ok());

    //     // First frame
    //     {
    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.required_size(dev),
    //                 size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2
    //             )
    //         });

    //         geometry
    //             .devices()
    //             .for_each(|dev| assert_eq!(op.remains[dev], GAIN_STM_SIZE));

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
    //                 Ok(size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2)
    //             );
    //         });

    //         geometry
    //             .devices()
    //             .for_each(|dev| assert_eq!(op.remains[dev], GAIN_STM_SIZE - 1));

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(TypeTag::GainSTM as u8, tx[dev.idx() * FRAME_SIZE]);
    //             assert_eq!(
    //                 GainSTMControlFlags::BEGIN.bits(),
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] & 0x3F
    //             );
    //             assert_eq!(
    //                 0,
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] >> 6
    //             );
    //             assert_eq!(
    //                 GainSTMMode::PhaseIntensityFull as u8,
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, mode)]
    //             );
    //             assert_eq!(
    //                 freq_div,
    //                 parse_tx_as::<u32>(
    //                     &tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, freq_div)..]
    //                 )
    //             );
    //             assert_eq!(
    //                 rep,
    //                 parse_tx_as::<u32>(
    //                     &tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, rep)..]
    //                 )
    //             );
    //             assert_eq!(
    //                 transition_mode.mode(),
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, transition_mode)]
    //             );
    //             assert_eq!(
    //                 ((transition_value / crate::ethercat::EC_CYCLE_TIME_BASE_NANO_SEC) + 1)
    //                     * crate::ethercat::EC_CYCLE_TIME_BASE_NANO_SEC,
    //                 parse_tx_as::<u64>(
    //                     &tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, transition_value)..]
    //                 )
    //             );

    //             tx[FRAME_SIZE * dev.idx() + size_of::<GainSTMHead>()..]
    //                 .chunks(size_of::<Drive>())
    //                 .zip(gain_data[0][&dev.idx()].iter())
    //                 .for_each(|(d, g)| {
    //                     assert_eq!(d[0], g.phase().value());
    //                     assert_eq!(d[1], g.intensity().value());
    //                 })
    //         });
    //     }

    //     // Second frame
    //     {
    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.required_size(dev),
    //                 size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
    //             )
    //         });

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
    //                 Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
    //             );
    //         });

    //         geometry
    //             .devices()
    //             .for_each(|dev| assert_eq!(op.remains[dev], GAIN_STM_SIZE - 2));

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(TypeTag::GainSTM as u8, tx[dev.idx() * FRAME_SIZE]);
    //             assert_eq!(
    //                 GainSTMControlFlags::NONE.bits(),
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] & 0x3F
    //             );
    //             assert_eq!(
    //                 0,
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] >> 6
    //             );

    //             tx[FRAME_SIZE * dev.idx() + 2..]
    //                 .chunks(size_of::<Drive>())
    //                 .zip(gain_data[1][&dev.idx()].iter())
    //                 .for_each(|(d, g)| {
    //                     assert_eq!(d[0], g.phase().value());
    //                     assert_eq!(d[1], g.intensity().value());
    //                 })
    //         });
    //     }

    //     // Final frame
    //     geometry.devices().for_each(|dev| {
    //         assert_eq!(
    //             op.required_size(dev),
    //             size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
    //         )
    //     });

    //     geometry.devices().for_each(|dev| {
    //         assert_eq!(
    //             op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
    //             Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
    //         );
    //     });

    //     geometry
    //         .devices()
    //         .for_each(|dev| assert_eq!(op.remains[dev], 0));

    //     geometry.devices().for_each(|dev| {
    //         assert_eq!(TypeTag::GainSTM as u8, tx[dev.idx() * FRAME_SIZE]);
    //         assert_eq!(
    //             (GainSTMControlFlags::END | GainSTMControlFlags::TRANSITION).bits(),
    //             tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] & 0x3F
    //         );
    //         assert_eq!(
    //             0,
    //             tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] >> 6
    //         );
    //         tx[FRAME_SIZE * dev.idx() + size_of::<GainSTMSubseq>()..]
    //             .chunks(size_of::<Drive>())
    //             .zip(gain_data[2][&dev.idx()].iter())
    //             .for_each(|(d, g)| {
    //                 assert_eq!(d[0], g.phase().value());
    //                 assert_eq!(d[1], g.intensity().value());
    //             })
    //     });
    // }

    // #[test]
    // fn test_phase_full() {
    //     const GAIN_STM_SIZE: usize = 5;
    //     const FRAME_SIZE: usize = size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;

    //     let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

    //     let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

    //     let mut rng = rand::thread_rng();

    //     let gain_data: Vec<HashMap<usize, Vec<Drive>>> = (0..GAIN_STM_SIZE)
    //         .map(|_| {
    //             geometry
    //                 .devices()
    //                 .map(|dev| {
    //                     (
    //                         dev.idx(),
    //                         (0..dev.num_transducers())
    //                             .map(|_| {
    //                                 Drive::new(
    //                                     Phase::new(rng.gen_range(0x00..=0xFF)),
    //                                     EmitIntensity::new(rng.gen_range(0..=0xFF)),
    //                                 )
    //                             })
    //                             .collect(),
    //                     )
    //                 })
    //                 .collect()
    //         })
    //         .collect();
    //     let gains: Vec<TestGain> = (0..GAIN_STM_SIZE)
    //         .map(|i| TestGain {
    //             data: gain_data[i].clone(),
    //         })
    //         .collect();

    //     let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
    //     let loop_behavior = LoopBehavior::finite(rng.gen_range(0x0000001..=0xFFFFFFFF)).unwrap();
    //     let segment = Segment::S1;
    //     let mut op = GainSTMOp::<_>::new(
    //         gains,
    //         GainSTMMode::PhaseFull,
    //         STMSamplingConfig::SamplingConfig(crate::derive::SamplingConfig::DivisionRaw(freq_div)),
    //         loop_behavior,
    //         segment,
    //         None,
    //     );

    //     assert!(op.init(&geometry).is_ok());

    //     geometry
    //         .devices()
    //         .for_each(|dev| assert_eq!(op.remains[dev], GAIN_STM_SIZE));

    //     // First frame
    //     {
    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.required_size(dev),
    //                 size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2
    //             )
    //         });

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
    //                 Ok(size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2)
    //             );
    //         });

    //         geometry
    //             .devices()
    //             .for_each(|dev| assert_eq!(op.remains[dev], GAIN_STM_SIZE - 2));

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(TypeTag::GainSTM as u8, tx[dev.idx() * FRAME_SIZE]);
    //             assert_eq!(
    //                 (GainSTMControlFlags::BEGIN | GainSTMControlFlags::SEGMENT).bits(),
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] & 0x3F
    //             );
    //             assert_eq!(
    //                 1,
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] >> 6
    //             );

    //             assert_eq!(
    //                 GainSTMMode::PhaseFull as u8,
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, mode)]
    //             );
    //             tx[FRAME_SIZE * dev.idx() + size_of::<GainSTMHead>()..]
    //                 .chunks(size_of::<Drive>())
    //                 .zip(gain_data[0][&dev.idx()].iter())
    //                 .zip(gain_data[1][&dev.idx()].iter())
    //                 .for_each(|((d, g0), g1)| {
    //                     assert_eq!(d[0], g0.phase().value());
    //                     assert_eq!(d[1], g1.phase().value());
    //                 })
    //         });
    //     }

    //     // Second frame
    //     {
    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.required_size(dev),
    //                 size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
    //             )
    //         });

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
    //                 Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
    //             );
    //         });

    //         geometry
    //             .devices()
    //             .for_each(|dev| assert_eq!(op.remains[dev], GAIN_STM_SIZE - 4));

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(TypeTag::GainSTM as u8, tx[dev.idx() * FRAME_SIZE]);
    //             assert_eq!(
    //                 GainSTMControlFlags::SEGMENT.bits(),
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] & 0x3F
    //             );
    //             assert_eq!(
    //                 1,
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] >> 6
    //             );
    //             tx[FRAME_SIZE * dev.idx() + size_of::<GainSTMSubseq>()..]
    //                 .chunks(size_of::<Drive>())
    //                 .zip(gain_data[2][&dev.idx()].iter())
    //                 .zip(gain_data[3][&dev.idx()].iter())
    //                 .for_each(|((d, g0), g1)| {
    //                     assert_eq!(d[0], g0.phase().value());
    //                     assert_eq!(d[1], g1.phase().value());
    //                 })
    //         });
    //     }

    //     // Final frame
    //     {
    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.required_size(dev),
    //                 size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
    //             )
    //         });

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
    //                 Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
    //             );
    //         });

    //         geometry
    //             .devices()
    //             .for_each(|dev| assert_eq!(op.remains[dev], 0));

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(TypeTag::GainSTM as u8, tx[dev.idx() * FRAME_SIZE]);
    //             assert_eq!(
    //                 (GainSTMControlFlags::END | GainSTMControlFlags::SEGMENT).bits(),
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] & 0x3F
    //             );
    //             assert_eq!(
    //                 0,
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] >> 6
    //             );
    //             tx[FRAME_SIZE * dev.idx() + size_of::<GainSTMSubseq>()..]
    //                 .chunks(size_of::<Drive>())
    //                 .zip(gain_data[4][&dev.idx()].iter())
    //                 .for_each(|(d, g)| {
    //                     assert_eq!(d[0], g.phase().value());
    //                 })
    //         });
    //     }
    // }

    // #[test]
    // fn test_phase_half() {
    //     const GAIN_STM_SIZE: usize = 11;
    //     const FRAME_SIZE: usize = size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;

    //     let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

    //     let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

    //     let mut rng = rand::thread_rng();

    //     let gain_data: Vec<HashMap<usize, Vec<Drive>>> = (0..GAIN_STM_SIZE)
    //         .map(|_| {
    //             geometry
    //                 .devices()
    //                 .map(|dev| {
    //                     (
    //                         dev.idx(),
    //                         (0..dev.num_transducers())
    //                             .map(|_| {
    //                                 Drive::new(
    //                                     Phase::new(rng.gen_range(0x00..=0xFF)),
    //                                     EmitIntensity::new(rng.gen_range(0..=0xFF)),
    //                                 )
    //                             })
    //                             .collect(),
    //                     )
    //                 })
    //                 .collect()
    //         })
    //         .collect();
    //     let gains: Vec<TestGain> = (0..GAIN_STM_SIZE)
    //         .map(|i| TestGain {
    //             data: gain_data[i].clone(),
    //         })
    //         .collect();

    //     let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
    //     let loop_behavior = LoopBehavior::finite(rng.gen_range(0x0000001..=0xFFFFFFFF)).unwrap();
    //     let segment = Segment::S0;
    //     let mut op = GainSTMOp::<_>::new(
    //         gains,
    //         GainSTMMode::PhaseHalf,
    //         STMSamplingConfig::SamplingConfig(crate::derive::SamplingConfig::DivisionRaw(freq_div)),
    //         loop_behavior,
    //         segment,
    //         Some(TransitionMode::SyncIdx),
    //     );

    //     assert!(op.init(&geometry).is_ok());

    //     geometry
    //         .devices()
    //         .for_each(|dev| assert_eq!(op.remains[dev], GAIN_STM_SIZE));

    //     // First frame
    //     {
    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.required_size(dev),
    //                 size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2
    //             )
    //         });

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
    //                 Ok(size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2)
    //             );
    //         });

    //         geometry
    //             .devices()
    //             .for_each(|dev| assert_eq!(op.remains[dev], GAIN_STM_SIZE - 4));

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(TypeTag::GainSTM as u8, tx[dev.idx() * FRAME_SIZE]);
    //             assert_eq!(
    //                 GainSTMControlFlags::BEGIN.bits(),
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] & 0x3F
    //             );
    //             assert_eq!(
    //                 3,
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] >> 6
    //             );
    //             assert_eq!(
    //                 GainSTMMode::PhaseHalf as u8,
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, mode)]
    //             );

    //             tx[FRAME_SIZE * dev.idx() + size_of::<GainSTMHead>()..]
    //                 .chunks(size_of::<Drive>())
    //                 .zip(gain_data[0][&dev.idx()].iter())
    //                 .zip(gain_data[1][&dev.idx()].iter())
    //                 .zip(gain_data[2][&dev.idx()].iter())
    //                 .zip(gain_data[3][&dev.idx()].iter())
    //                 .for_each(|((((d, g0), g1), g2), g3)| {
    //                     assert_eq!(d[0] & 0x0F, g0.phase().value() >> 4);
    //                     assert_eq!(d[0] >> 4, g1.phase().value() >> 4);
    //                     assert_eq!(d[1] & 0x0F, g2.phase().value() >> 4);
    //                     assert_eq!(d[1] >> 4, g3.phase().value() >> 4);
    //                 })
    //         });
    //     }

    //     // Second frame
    //     {
    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.required_size(dev),
    //                 size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
    //             )
    //         });

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
    //                 Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
    //             );
    //         });

    //         geometry
    //             .devices()
    //             .for_each(|dev| assert_eq!(op.remains[dev], GAIN_STM_SIZE - 8));

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(TypeTag::GainSTM as u8, tx[dev.idx() * FRAME_SIZE]);
    //             assert_eq!(
    //                 GainSTMControlFlags::NONE.bits(),
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] & 0x3F
    //             );
    //             assert_eq!(
    //                 3,
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] >> 6
    //             );
    //             tx[FRAME_SIZE * dev.idx() + size_of::<GainSTMSubseq>()..]
    //                 .chunks(size_of::<Drive>())
    //                 .zip(gain_data[4][&dev.idx()].iter())
    //                 .zip(gain_data[5][&dev.idx()].iter())
    //                 .zip(gain_data[6][&dev.idx()].iter())
    //                 .zip(gain_data[7][&dev.idx()].iter())
    //                 .for_each(|((((d, g0), g1), g2), g3)| {
    //                     assert_eq!(d[0] & 0x0F, g0.phase().value() >> 4);
    //                     assert_eq!(d[0] >> 4, g1.phase().value() >> 4);
    //                     assert_eq!(d[1] & 0x0F, g2.phase().value() >> 4);
    //                     assert_eq!(d[1] >> 4, g3.phase().value() >> 4);
    //                 })
    //         });
    //     }

    //     // Final frame
    //     {
    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.required_size(dev),
    //                 size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
    //             )
    //         });

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(
    //                 op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
    //                 Ok(size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
    //             );
    //         });

    //         geometry
    //             .devices()
    //             .for_each(|dev| assert_eq!(op.remains[dev], 0));

    //         geometry.devices().for_each(|dev| {
    //             assert_eq!(TypeTag::GainSTM as u8, tx[dev.idx() * FRAME_SIZE]);
    //             assert_eq!(
    //                 (GainSTMControlFlags::END | GainSTMControlFlags::TRANSITION).bits(),
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] & 0x3F
    //             );
    //             assert_eq!(
    //                 2,
    //                 tx[dev.idx() * FRAME_SIZE + offset_of!(GainSTMHead, flag)] >> 6
    //             );
    //             tx[FRAME_SIZE * dev.idx() + size_of::<GainSTMSubseq>()..]
    //                 .chunks(size_of::<Drive>())
    //                 .zip(gain_data[8][&dev.idx()].iter())
    //                 .for_each(|(d, g)| {
    //                     assert_eq!(d[0] & 0x0F, g.phase().value() >> 4);
    //                 })
    //         });
    //     }
    // }
}
