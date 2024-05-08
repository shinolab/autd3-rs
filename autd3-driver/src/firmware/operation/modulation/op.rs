use std::collections::HashMap;

use crate::{
    derive::Modulation,
    error::AUTDInternalError,
    firmware::{
        fpga::{Segment, TransitionMode, MOD_BUF_SIZE_MAX, TRANSITION_MODE_NONE},
        operation::{cast, Operation, Remains, TypeTag},
    },
    geometry::{Device, Geometry},
};

use super::ModulationControlFlags;

#[repr(C, align(2))]
struct ModulationHead {
    tag: TypeTag,
    flag: ModulationControlFlags,
    size: u16,
    transition_mode: u8,
    __pad: [u8; 3],
    freq_div: u32,
    rep: u32,
    transition_value: u64,
}

#[repr(C, align(2))]
struct ModulationSubseq {
    tag: TypeTag,
    flag: ModulationControlFlags,
    size: u16,
}

pub struct ModulationOp<M: Modulation> {
    modulation: M,
    buf: HashMap<usize, Vec<u8>>,
    remains: Remains,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<M: Modulation> ModulationOp<M> {
    pub fn new(modulation: M, segment: Segment, transition_mode: Option<TransitionMode>) -> Self {
        Self {
            modulation,
            buf: Default::default(),
            remains: Default::default(),
            segment,
            transition_mode,
        }
    }
}

impl<M: Modulation> Operation for ModulationOp<M> {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let buf = &self.buf[&device.idx()];

        let sent = buf.len() - self.remains[device];

        let offset = if sent == 0 {
            std::mem::size_of::<ModulationHead>()
        } else {
            std::mem::size_of::<ModulationSubseq>()
        };

        let mod_size = (buf.len() - sent).min(tx.len() - offset);
        assert!(mod_size > 0);

        if sent == 0 {
            *cast::<ModulationHead>(tx) = ModulationHead {
                tag: TypeTag::Modulation,
                flag: ModulationControlFlags::BEGIN,
                size: mod_size as u16,
                __pad: [0; 3],
                freq_div: self
                    .modulation
                    .sampling_config()
                    .division(device.ultrasound_freq())?,
                rep: self.modulation.loop_behavior().rep,
                transition_mode: self
                    .transition_mode
                    .map(|m| m.mode())
                    .unwrap_or(TRANSITION_MODE_NONE),
                transition_value: self.transition_mode.map(|m| m.value()).unwrap_or(0),
            };
        } else {
            *cast::<ModulationSubseq>(tx) = ModulationSubseq {
                tag: TypeTag::Modulation,
                flag: ModulationControlFlags::NONE,
                size: mod_size as u16,
            };
        }
        cast::<ModulationSubseq>(tx)
            .flag
            .set(ModulationControlFlags::SEGMENT, self.segment == Segment::S1);

        if sent + mod_size == buf.len() {
            let d = cast::<ModulationSubseq>(tx);
            d.flag.set(ModulationControlFlags::END, true);
            d.flag.set(
                ModulationControlFlags::TRANSITION,
                self.transition_mode.is_some(),
            );
        }

        unsafe {
            std::ptr::copy_nonoverlapping(
                buf[sent..].as_ptr(),
                tx[offset..].as_mut_ptr() as _,
                mod_size,
            )
        }

        self.remains[device] -= mod_size;
        if sent == 0 {
            Ok(std::mem::size_of::<ModulationHead>() + mod_size)
        } else {
            Ok(std::mem::size_of::<ModulationSubseq>() + mod_size)
        }
    }

    fn required_size(&self, device: &Device) -> usize {
        if self.remains[device] == self.buf[&device.idx()].len() {
            std::mem::size_of::<ModulationHead>() + 1
        } else {
            std::mem::size_of::<ModulationSubseq>() + 1
        }
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.buf = self.modulation.calc(geometry)?;
        self.buf.values().try_for_each(|buf| {
            if !(2..=MOD_BUF_SIZE_MAX).contains(&buf.len()) {
                return Err(AUTDInternalError::ModulationSizeOutOfRange(buf.len()));
            }
            Ok(())
        })?;

        self.remains
            .init(geometry, |dev| self.buf[&dev.idx()].len());

        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;

    use rand::prelude::*;

    use super::*;
    use crate::{
        defined::FREQ_40K,
        derive::{LoopBehavior, SamplingConfig},
        ethercat::DcSysTime,
        firmware::{
            fpga::{SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
            operation::tests::{parse_tx_as, TestModulation},
        },
        geometry::tests::create_geometry,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test() {
        const MOD_SIZE: usize = 100;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

        let mut tx = vec![0x00u8; (std::mem::size_of::<ModulationHead>() + MOD_SIZE) * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let buf: Vec<u8> = (0..MOD_SIZE).map(|_| rng.gen()).collect();
        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
        let loop_behavior = LoopBehavior::infinite();
        let segment = Segment::S0;
        let rep = loop_behavior.rep;
        let transition_mode = TransitionMode::SysTime(
            DcSysTime::from_utc(time::macros::datetime!(2000-01-01 0:00 UTC)).unwrap()
                + std::time::Duration::from_nanos(0x0123456789ABCDEF),
        );

        let mut op = ModulationOp::new(
            TestModulation {
                buf: buf.clone(),
                config: SamplingConfig::DivisionRaw(freq_div),
                loop_behavior,
            },
            segment,
            Some(transition_mode),
        );

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<ModulationHead>() + 1
            )
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], MOD_SIZE));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE)..]
                ),
                Ok(std::mem::size_of::<ModulationHead>() + MOD_SIZE)
            );
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                TypeTag::Modulation as u8,
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE)
                    + offset_of!(ModulationHead, tag)],
            );
            assert_eq!(
                (ModulationControlFlags::BEGIN
                    | ModulationControlFlags::END
                    | ModulationControlFlags::TRANSITION)
                    .bits(),
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE)
                    + offset_of!(ModulationHead, flag)]
            );
            assert_eq!(
                MOD_SIZE as u16,
                parse_tx_as::<u16>(
                    &tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE)
                        + offset_of!(ModulationHead, size)..]
                )
            );
            assert_eq!(
                freq_div,
                parse_tx_as::<u32>(
                    &tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE)
                        + offset_of!(ModulationHead, freq_div)..]
                )
            );
            assert_eq!(
                rep,
                parse_tx_as::<u32>(
                    &tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE)
                        + offset_of!(ModulationHead, rep)..]
                )
            );
            assert_eq!(
                transition_mode.mode(),
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE)
                    + offset_of!(ModulationHead, transition_mode)]
            );
            assert_eq!(
                transition_mode.value(),
                parse_tx_as::<u64>(
                    &tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE)
                        + offset_of!(ModulationHead, transition_value)..]
                )
            );

            tx.iter()
                .skip((std::mem::size_of::<ModulationHead>() + MOD_SIZE) * dev.idx())
                .skip(std::mem::size_of::<ModulationHead>())
                .zip(buf.iter())
                .for_each(|(&d, &m)| {
                    assert_eq!(d, m);
                })
        });
    }

    #[test]
    fn test_div() {
        const FRAME_SIZE: usize = 30;
        const MOD_SIZE: usize = FRAME_SIZE - std::mem::size_of::<ModulationHead>()
            + (FRAME_SIZE - std::mem::size_of::<ModulationSubseq>()) * 2;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

        let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let buf: Vec<u8> = (0..MOD_SIZE).map(|_| rng.gen()).collect();

        let mut op = ModulationOp::new(
            TestModulation {
                buf: buf.clone(),
                config: SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MIN),
                loop_behavior: LoopBehavior::infinite(),
            },
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );

        assert!(op.init(&geometry).is_ok());

        // First frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<ModulationHead>() + 1
            )
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], MOD_SIZE));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                ),
                Ok(FRAME_SIZE)
            );
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.remains[dev],
                MOD_SIZE - (FRAME_SIZE - std::mem::size_of::<ModulationHead>())
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(TypeTag::Modulation as u8, tx[dev.idx() * FRAME_SIZE]);
            assert_eq!(
                ModulationControlFlags::BEGIN.bits(),
                tx[dev.idx() * FRAME_SIZE + offset_of!(ModulationHead, flag)]
            );
            let mod_size = FRAME_SIZE - std::mem::size_of::<ModulationHead>();
            assert_eq!(
                mod_size as u16,
                parse_tx_as::<u16>(
                    &tx[dev.idx() * FRAME_SIZE + offset_of!(ModulationHead, size)..]
                )
            );
            tx.iter()
                .skip(FRAME_SIZE * dev.idx())
                .skip(std::mem::size_of::<ModulationHead>())
                .zip(buf.iter().take(mod_size))
                .for_each(|(&d, &m)| {
                    assert_eq!(d, m);
                })
        });

        // Second frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<ModulationSubseq>() + 1
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                ),
                Ok(FRAME_SIZE)
            );
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.remains[dev],
                MOD_SIZE
                    - (FRAME_SIZE - std::mem::size_of::<ModulationHead>())
                    - (FRAME_SIZE - std::mem::size_of::<ModulationSubseq>())
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(TypeTag::Modulation as u8, tx[dev.idx() * FRAME_SIZE]);
            assert_eq!(
                ModulationControlFlags::NONE.bits(),
                tx[dev.idx() * FRAME_SIZE + offset_of!(ModulationHead, flag)]
            );
            let mod_size = FRAME_SIZE - std::mem::size_of::<ModulationSubseq>();
            assert_eq!(
                mod_size as u16,
                parse_tx_as::<u16>(
                    &tx[dev.idx() * FRAME_SIZE + offset_of!(ModulationHead, size)..]
                )
            );
            tx.iter()
                .skip(FRAME_SIZE * dev.idx())
                .skip(std::mem::size_of::<ModulationSubseq>())
                .zip(
                    buf.iter()
                        .skip(FRAME_SIZE - std::mem::size_of::<ModulationHead>())
                        .take(mod_size),
                )
                .for_each(|(&d, &m)| {
                    assert_eq!(d, m);
                })
        });

        // Final frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<ModulationSubseq>() + 1
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                ),
                Ok(FRAME_SIZE)
            );
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(TypeTag::Modulation as u8, tx[dev.idx() * FRAME_SIZE]);
            assert_eq!(
                (ModulationControlFlags::TRANSITION | ModulationControlFlags::END).bits(),
                tx[dev.idx() * FRAME_SIZE + offset_of!(ModulationHead, flag)]
            );
            let mod_size = FRAME_SIZE - std::mem::size_of::<ModulationSubseq>();
            assert_eq!(
                mod_size as u16,
                parse_tx_as::<u16>(
                    &tx[dev.idx() * FRAME_SIZE + offset_of!(ModulationHead, size)..]
                )
            );
            tx.iter()
                .skip(FRAME_SIZE * dev.idx())
                .skip(std::mem::size_of::<ModulationSubseq>())
                .zip(
                    buf.iter()
                        .skip(
                            FRAME_SIZE - std::mem::size_of::<ModulationHead>() + FRAME_SIZE
                                - std::mem::size_of::<ModulationSubseq>(),
                        )
                        .take(mod_size),
                )
                .for_each(|(&d, &m)| {
                    assert_eq!(d, m);
                })
        });
    }

    #[rstest::rstest]
    #[test]
    #[case(Err(AUTDInternalError::ModulationSizeOutOfRange(1)), 1)]
    #[case(Ok(()), 2)]
    #[case(Ok(()), MOD_BUF_SIZE_MAX)]
    #[case(Err(AUTDInternalError::ModulationSizeOutOfRange(
        MOD_BUF_SIZE_MAX + 1
    )), MOD_BUF_SIZE_MAX + 1)]
    fn test_buffer_out_of_range(#[case] expect: Result<(), AUTDInternalError>, #[case] n: usize) {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, FREQ_40K);

        let mut rng = rand::thread_rng();

        let mut op = ModulationOp::new(
            TestModulation {
                buf: (0..n).map(|_| rng.gen()).collect(),
                config: SamplingConfig::DivisionRaw(
                    rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX),
                ),
                loop_behavior: LoopBehavior::infinite(),
            },
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );

        assert_eq!(expect, op.init(&geometry));
    }
}
