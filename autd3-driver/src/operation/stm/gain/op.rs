use std::collections::HashMap;

use crate::{
    common::{Drive, LoopBehavior, Segment},
    datagram::{Gain, GainFilter},
    error::AUTDInternalError,
    fpga::{FPGADrive, GAIN_STM_BUF_SIZE_MAX},
    geometry::{Device, Geometry},
    operation::{cast, Operation, TypeTag},
};

use super::{
    control_flags::GainSTMControlFlags,
    mode::GainSTMMode,
    reduced_phase::{PhaseFull, PhaseHalf},
};

#[repr(C)]
struct GainSTMHead {
    tag: TypeTag,
    flag: GainSTMControlFlags,
    mode: GainSTMMode,
    segment: u8,
    freq_div: u32,
    rep: u32,
}

#[repr(C)]
struct GainSTMSubseq {
    tag: TypeTag,
    flag: GainSTMControlFlags,
}

pub struct GainSTMOp<G: Gain> {
    gains: Vec<G>,
    drives: Vec<HashMap<usize, Vec<Drive>>>,
    remains: HashMap<usize, usize>,
    sent: HashMap<usize, usize>,
    mode: GainSTMMode,
    freq_div: u32,
    loop_behavior: LoopBehavior,
    segment: Segment,
    update_segment: bool,
}

impl<G: Gain> GainSTMOp<G> {
    pub fn new(
        gains: Vec<G>,
        mode: GainSTMMode,
        freq_div: u32,
        loop_behavior: LoopBehavior,
        segment: Segment,
        update_segment: bool,
    ) -> Self {
        Self {
            gains,
            drives: Default::default(),
            remains: Default::default(),
            sent: Default::default(),
            mode,
            freq_div,
            loop_behavior,
            segment,
            update_segment,
        }
    }
}

impl<G: Gain> Operation for GainSTMOp<G> {
    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.drives = self
            .gains
            .iter()
            .map(|g| g.calc(geometry, GainFilter::All))
            .collect::<Result<_, _>>()?;

        if !(2..=GAIN_STM_BUF_SIZE_MAX).contains(&self.drives.len()) {
            return Err(AUTDInternalError::GainSTMSizeOutOfRange(self.drives.len()));
        }

        self.remains = geometry
            .devices()
            .map(|device| (device.idx(), self.drives.len()))
            .collect();

        self.sent = geometry.devices().map(|device| (device.idx(), 0)).collect();

        Ok(())
    }

    fn required_size(&self, device: &Device) -> usize {
        if self.sent[&device.idx()] == 0 {
            std::mem::size_of::<GainSTMHead>()
                + device.num_transducers() * std::mem::size_of::<FPGADrive>()
        } else {
            std::mem::size_of::<GainSTMSubseq>()
                + device.num_transducers() * std::mem::size_of::<FPGADrive>()
        }
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert!(self.remains[&device.idx()] > 0);

        let sent = self.sent[&device.idx()];

        let offset = if sent == 0 {
            std::mem::size_of::<GainSTMHead>()
        } else {
            std::mem::size_of::<GainSTMSubseq>()
        };
        assert!(tx.len() >= offset + device.num_transducers() * std::mem::size_of::<FPGADrive>());

        let mut send = 0;
        match self.mode {
            GainSTMMode::PhaseIntensityFull => {
                let d = &self.drives[sent][&device.idx()];
                unsafe {
                    std::slice::from_raw_parts_mut(
                        tx[offset..].as_mut_ptr() as *mut FPGADrive,
                        d.len(),
                    )
                    .iter_mut()
                    .zip(d.iter())
                    .for_each(|(d, s)| d.set(s));
                }
                send += 1;
            }
            GainSTMMode::PhaseFull => {
                let d = &self.drives[sent][&device.idx()];
                unsafe {
                    std::slice::from_raw_parts_mut(
                        tx[offset..].as_mut_ptr() as *mut PhaseFull<0>,
                        d.len(),
                    )
                    .iter_mut()
                    .zip(d.iter())
                    .for_each(|(d, s)| d.set(s));
                }
                send += 1;
                if self.drives.len() > sent + 1 {
                    let d = &self.drives[sent + 1][&device.idx()];
                    unsafe {
                        std::slice::from_raw_parts_mut(
                            tx[offset..].as_mut_ptr() as *mut PhaseFull<1>,
                            d.len(),
                        )
                        .iter_mut()
                        .zip(d.iter())
                        .for_each(|(d, s)| d.set(s));
                    }
                    send += 1;
                }
            }
            GainSTMMode::PhaseHalf => {
                let d = &self.drives[sent][&device.idx()];
                unsafe {
                    std::slice::from_raw_parts_mut(
                        tx[offset..].as_mut_ptr() as *mut PhaseHalf<0>,
                        d.len(),
                    )
                    .iter_mut()
                    .zip(d.iter())
                    .for_each(|(d, s)| d.set(s));
                }
                send += 1;
                if self.drives.len() > sent + 1 {
                    let d = &self.drives[sent + 1][&device.idx()];
                    unsafe {
                        std::slice::from_raw_parts_mut(
                            tx[offset..].as_mut_ptr() as *mut PhaseHalf<1>,
                            d.len(),
                        )
                        .iter_mut()
                        .zip(d.iter())
                        .for_each(|(d, s)| d.set(s));
                    }
                    send += 1;
                }
                if self.drives.len() > sent + 2 {
                    let d = &self.drives[sent + 2][&device.idx()];
                    unsafe {
                        std::slice::from_raw_parts_mut(
                            tx[offset..].as_mut_ptr() as *mut PhaseHalf<2>,
                            d.len(),
                        )
                        .iter_mut()
                        .zip(d.iter())
                        .for_each(|(d, s)| d.set(s));
                    }
                    send += 1;
                }
                if self.drives.len() > sent + 3 {
                    let d = &self.drives[sent + 3][&device.idx()];
                    unsafe {
                        std::slice::from_raw_parts_mut(
                            tx[offset..].as_mut_ptr() as *mut PhaseHalf<3>,
                            d.len(),
                        )
                        .iter_mut()
                        .zip(d.iter())
                        .for_each(|(d, s)| d.set(s));
                    }
                    send += 1;
                }
            }
        }

        if sent == 0 {
            let d = cast::<GainSTMHead>(tx);
            d.tag = TypeTag::GainSTM;
            d.flag = GainSTMControlFlags::BEGIN;
            d.mode = self.mode;
            d.segment = self.segment as u8;
            d.freq_div = self.freq_div;
            d.rep = self.loop_behavior.to_rep();
        } else {
            let d = cast::<GainSTMSubseq>(tx);
            d.tag = TypeTag::GainSTM;
            d.flag = GainSTMControlFlags::NONE;
        }

        let d = cast::<GainSTMSubseq>(tx);
        d.flag.set(
            GainSTMControlFlags::SEND_BIT0,
            ((send as u8 - 1) & 0x01) != 0,
        );
        d.flag.set(
            GainSTMControlFlags::SEND_BIT1,
            ((send as u8 - 1) & 0x02) != 0,
        );

        if sent + send == self.drives.len() {
            d.flag.set(GainSTMControlFlags::END, true);
            d.flag.set(GainSTMControlFlags::UPDATE, self.update_segment);
        }

        self.sent.insert(device.idx(), sent + send);

        if sent == 0 {
            Ok(std::mem::size_of::<GainSTMHead>()
                + device.num_transducers() * std::mem::size_of::<FPGADrive>())
        } else {
            Ok(std::mem::size_of::<GainSTMSubseq>()
                + device.num_transducers() * std::mem::size_of::<FPGADrive>())
        }
    }

    fn commit(&mut self, device: &Device) {
        self.remains
            .insert(device.idx(), self.drives.len() - self.sent[&device.idx()]);
    }

    fn remains(&self, device: &Device) -> usize {
        self.remains[&device.idx()]
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use rand::prelude::*;

    use super::*;
    use crate::{
        common::{EmitIntensity, Phase},
        fpga::{SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
        geometry::tests::create_geometry,
        operation::tests::{NullGain, TestGain},
    };

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test_phase_intensity_full() {
        const GAIN_STM_SIZE: usize = 3;
        const FRAME_SIZE: usize = std::mem::size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let gain_data: Vec<HashMap<usize, Vec<Drive>>> = (0..GAIN_STM_SIZE)
            .map(|_| {
                geometry
                    .devices()
                    .map(|dev| {
                        (
                            dev.idx(),
                            (0..dev.num_transducers())
                                .map(|_| {
                                    Drive::new(
                                        Phase::new(rng.gen_range(0x00..=0xFF)),
                                        EmitIntensity::new(rng.gen_range(0..=0xFF)),
                                    )
                                })
                                .collect(),
                        )
                    })
                    .collect()
            })
            .collect();
        let gains: Vec<TestGain> = (0..GAIN_STM_SIZE)
            .map(|i| TestGain {
                data: gain_data[i].clone(),
            })
            .collect();

        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
        let loop_behavior = LoopBehavior::Infinite;
        let segment = Segment::S0;

        let mut op = GainSTMOp::<_>::new(
            gains,
            GainSTMMode::PhaseIntensityFull,
            freq_div,
            loop_behavior,
            segment,
            true,
        );

        assert!(op.init(&geometry).is_ok());

        // First frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2
            )
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), GAIN_STM_SIZE));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(std::mem::size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), GAIN_STM_SIZE - 1));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::GainSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_ne!(flag & GainSTMControlFlags::BEGIN.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::END.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::UPDATE.bits(), 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 1] >> 6, 0);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 2],
                GainSTMMode::PhaseIntensityFull as u8
            );
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 3], segment as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 4], (freq_div & 0xFF) as u8);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 5],
                ((freq_div >> 8) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 6],
                ((freq_div >> 16) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 7],
                ((freq_div >> 24) & 0xFF) as u8
            );
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 8], 0xFF);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 9], 0xFF);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 10], 0xFF);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 11], 0xFF);

            tx[FRAME_SIZE * dev.idx() + std::mem::size_of::<GainSTMHead>()..]
                .chunks(std::mem::size_of::<FPGADrive>())
                .zip(gain_data[0][&dev.idx()].iter())
                .for_each(|(d, g)| {
                    assert_eq!(d[0], g.phase().value());
                    assert_eq!(d[1], g.intensity().value());
                })
        });

        // Second frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), GAIN_STM_SIZE - 2));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::GainSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_eq!(flag & GainSTMControlFlags::BEGIN.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::END.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::UPDATE.bits(), 0x00);

            assert_eq!(tx[dev.idx() * FRAME_SIZE + 1] >> 6, 0);

            tx[FRAME_SIZE * dev.idx() + 2..]
                .chunks(std::mem::size_of::<FPGADrive>())
                .zip(gain_data[1][&dev.idx()].iter())
                .for_each(|(d, g)| {
                    assert_eq!(d[0], g.phase().value());
                    assert_eq!(d[1], g.intensity().value());
                })
        });

        // Final frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::GainSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_eq!(flag & GainSTMControlFlags::BEGIN.bits(), 0x00);
            assert_ne!(flag & GainSTMControlFlags::END.bits(), 0x00);
            assert_ne!(flag & GainSTMControlFlags::UPDATE.bits(), 0x00);

            assert_eq!(tx[dev.idx() * FRAME_SIZE + 1] >> 6, 0);

            tx[FRAME_SIZE * dev.idx() + std::mem::size_of::<GainSTMSubseq>()..]
                .chunks(std::mem::size_of::<FPGADrive>())
                .zip(gain_data[2][&dev.idx()].iter())
                .for_each(|(d, g)| {
                    assert_eq!(d[0], g.phase().value());
                    assert_eq!(d[1], g.intensity().value());
                })
        });
    }

    #[test]
    fn test_phase_full() {
        const GAIN_STM_SIZE: usize = 5;
        const FRAME_SIZE: usize = std::mem::size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let gain_data: Vec<HashMap<usize, Vec<Drive>>> = (0..GAIN_STM_SIZE)
            .map(|_| {
                geometry
                    .devices()
                    .map(|dev| {
                        (
                            dev.idx(),
                            (0..dev.num_transducers())
                                .map(|_| {
                                    Drive::new(
                                        Phase::new(rng.gen_range(0x00..=0xFF)),
                                        EmitIntensity::new(rng.gen_range(0..=0xFF)),
                                    )
                                })
                                .collect(),
                        )
                    })
                    .collect()
            })
            .collect();
        let gains: Vec<TestGain> = (0..GAIN_STM_SIZE)
            .map(|i| TestGain {
                data: gain_data[i].clone(),
            })
            .collect();

        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
        let loop_behavior = LoopBehavior::Finite(
            NonZeroU32::new(rng.gen_range(0x0000001..=0xFFFFFFFF)).unwrap_or(NonZeroU32::MIN),
        );
        let rep = loop_behavior.to_rep();
        let segment = Segment::S1;
        let mut op = GainSTMOp::<_>::new(
            gains,
            GainSTMMode::PhaseFull,
            freq_div,
            loop_behavior,
            segment,
            false,
        );

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), GAIN_STM_SIZE));

        // First frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(std::mem::size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), GAIN_STM_SIZE - 2));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::GainSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_ne!(flag & GainSTMControlFlags::BEGIN.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::END.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::UPDATE.bits(), 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 1] >> 6, 1);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 2], GainSTMMode::PhaseFull as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 3], segment as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 4], (freq_div & 0xFF) as u8);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 5],
                ((freq_div >> 8) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 6],
                ((freq_div >> 16) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 7],
                ((freq_div >> 24) & 0xFF) as u8
            );
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 8], (rep & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 9], ((rep >> 8) & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 10], ((rep >> 16) & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 11], ((rep >> 24) & 0xFF) as u8);

            tx[FRAME_SIZE * dev.idx() + 12..]
                .chunks(std::mem::size_of::<FPGADrive>())
                .zip(gain_data[0][&dev.idx()].iter())
                .zip(gain_data[1][&dev.idx()].iter())
                .for_each(|((d, g0), g1)| {
                    assert_eq!(d[0], g0.phase().value());
                    assert_eq!(d[1], g1.phase().value());
                })
        });

        // Second frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), GAIN_STM_SIZE - 4));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::GainSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_eq!(flag & GainSTMControlFlags::BEGIN.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::END.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::UPDATE.bits(), 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 1] >> 6, 1);
            tx[FRAME_SIZE * dev.idx() + std::mem::size_of::<GainSTMSubseq>()..]
                .chunks(std::mem::size_of::<FPGADrive>())
                .zip(gain_data[2][&dev.idx()].iter())
                .zip(gain_data[3][&dev.idx()].iter())
                .for_each(|((d, g0), g1)| {
                    assert_eq!(d[0], g0.phase().value());
                    assert_eq!(d[1], g1.phase().value());
                })
        });

        // Final frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::GainSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_eq!(flag & GainSTMControlFlags::BEGIN.bits(), 0x00);
            assert_ne!(flag & GainSTMControlFlags::END.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::UPDATE.bits(), 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 1] >> 6, 0);
            tx[FRAME_SIZE * dev.idx() + std::mem::size_of::<GainSTMSubseq>()..]
                .chunks(std::mem::size_of::<FPGADrive>())
                .zip(gain_data[4][&dev.idx()].iter())
                .for_each(|(d, g)| {
                    assert_eq!(d[0], g.phase().value());
                })
        });
    }

    #[test]
    fn test_phase_half() {
        const GAIN_STM_SIZE: usize = 11;
        const FRAME_SIZE: usize = std::mem::size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let gain_data: Vec<HashMap<usize, Vec<Drive>>> = (0..GAIN_STM_SIZE)
            .map(|_| {
                geometry
                    .devices()
                    .map(|dev| {
                        (
                            dev.idx(),
                            (0..dev.num_transducers())
                                .map(|_| {
                                    Drive::new(
                                        Phase::new(rng.gen_range(0x00..=0xFF)),
                                        EmitIntensity::new(rng.gen_range(0..=0xFF)),
                                    )
                                })
                                .collect(),
                        )
                    })
                    .collect()
            })
            .collect();
        let gains: Vec<TestGain> = (0..GAIN_STM_SIZE)
            .map(|i| TestGain {
                data: gain_data[i].clone(),
            })
            .collect();

        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
        let loop_behavior = LoopBehavior::Finite(
            NonZeroU32::new(rng.gen_range(0x0000001..=0xFFFFFFFF)).unwrap_or(NonZeroU32::MIN),
        );
        let rep = loop_behavior.to_rep();
        let segment = Segment::S0;
        let mut op = GainSTMOp::<_>::new(
            gains,
            GainSTMMode::PhaseHalf,
            freq_div,
            loop_behavior,
            segment,
            true,
        );

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), GAIN_STM_SIZE));

        // First frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(std::mem::size_of::<GainSTMHead>() + NUM_TRANS_IN_UNIT * 2)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), GAIN_STM_SIZE - 4));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::GainSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_ne!(flag & GainSTMControlFlags::BEGIN.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::END.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::UPDATE.bits(), 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 1] >> 6, 3);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 2], GainSTMMode::PhaseHalf as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 3], segment as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 4], (freq_div & 0xFF) as u8);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 5],
                ((freq_div >> 8) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 6],
                ((freq_div >> 16) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 7],
                ((freq_div >> 24) & 0xFF) as u8
            );
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 8], (rep & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 9], ((rep >> 8) & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 10], ((rep >> 16) & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 11], ((rep >> 24) & 0xFF) as u8);

            tx[FRAME_SIZE * dev.idx() + std::mem::size_of::<GainSTMHead>()..]
                .chunks(std::mem::size_of::<FPGADrive>())
                .zip(gain_data[0][&dev.idx()].iter())
                .zip(gain_data[1][&dev.idx()].iter())
                .zip(gain_data[2][&dev.idx()].iter())
                .zip(gain_data[3][&dev.idx()].iter())
                .for_each(|((((d, g0), g1), g2), g3)| {
                    assert_eq!(d[0] & 0x0F, g0.phase().value() >> 4);
                    assert_eq!(d[0] >> 4, g1.phase().value() >> 4);
                    assert_eq!(d[1] & 0x0F, g2.phase().value() >> 4);
                    assert_eq!(d[1] >> 4, g3.phase().value() >> 4);
                })
        });

        // Second frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), GAIN_STM_SIZE - 8));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::GainSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_eq!(flag & GainSTMControlFlags::BEGIN.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::END.bits(), 0x00);
            assert_eq!(flag & GainSTMControlFlags::UPDATE.bits(), 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 1] >> 6, 3);
            tx[FRAME_SIZE * dev.idx() + std::mem::size_of::<GainSTMSubseq>()..]
                .chunks(std::mem::size_of::<FPGADrive>())
                .zip(gain_data[4][&dev.idx()].iter())
                .zip(gain_data[5][&dev.idx()].iter())
                .zip(gain_data[6][&dev.idx()].iter())
                .zip(gain_data[7][&dev.idx()].iter())
                .for_each(|((((d, g0), g1), g2), g3)| {
                    assert_eq!(d[0] & 0x0F, g0.phase().value() >> 4);
                    assert_eq!(d[0] >> 4, g1.phase().value() >> 4);
                    assert_eq!(d[1] & 0x0F, g2.phase().value() >> 4);
                    assert_eq!(d[1] >> 4, g3.phase().value() >> 4);
                })
        });

        // Final frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(std::mem::size_of::<GainSTMSubseq>() + NUM_TRANS_IN_UNIT * 2)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::GainSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_eq!(flag & GainSTMControlFlags::BEGIN.bits(), 0x00);
            assert_ne!(flag & GainSTMControlFlags::END.bits(), 0x00);
            assert_ne!(flag & GainSTMControlFlags::UPDATE.bits(), 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 1] >> 6, 2);
            tx[FRAME_SIZE * dev.idx() + std::mem::size_of::<GainSTMSubseq>()..]
                .chunks(std::mem::size_of::<FPGADrive>())
                .zip(gain_data[8][&dev.idx()].iter())
                .for_each(|(d, g)| {
                    assert_eq!(d[0] & 0x0F, g.phase().value() >> 4);
                })
        });
    }

    #[test]
    fn test_buffer_out_of_range() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let test = |n: usize| {
            let gains: Vec<NullGain> = (0..n).map(|_| NullGain {}).collect();

            let mut op = GainSTMOp::<_>::new(
                gains,
                GainSTMMode::PhaseIntensityFull,
                SAMPLING_FREQ_DIV_MIN,
                LoopBehavior::Infinite,
                Segment::S0,
                true,
            );
            op.init(&geometry)
        };

        assert_eq!(test(1), Err(AUTDInternalError::GainSTMSizeOutOfRange(1)));
        assert_eq!(test(2), Ok(()));
        assert_eq!(test(GAIN_STM_BUF_SIZE_MAX), Ok(()));
        assert_eq!(
            test(GAIN_STM_BUF_SIZE_MAX + 1),
            Err(AUTDInternalError::GainSTMSizeOutOfRange(
                GAIN_STM_BUF_SIZE_MAX + 1
            ))
        );
    }
}
