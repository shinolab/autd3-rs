use std::collections::HashMap;

use crate::{
    common::{EmitIntensity, LoopBehavior, Segment},
    error::AUTDInternalError,
    fpga::MOD_BUF_SIZE_MAX,
    geometry::{Device, Geometry},
    operation::{cast, Operation, TypeTag},
};

use super::ModulationControlFlags;

#[repr(C, align(2))]
struct ModulationHead {
    tag: TypeTag,
    flag: ModulationControlFlags,
    size: u16,
    freq_div: u32,
    rep: u32,
    segment: u32,
}

#[repr(C, align(2))]
struct ModulationSubseq {
    tag: TypeTag,
    flag: ModulationControlFlags,
    size: u16,
}

pub struct ModulationOp {
    buf: Vec<EmitIntensity>,
    freq_div: u32,
    remains: HashMap<usize, usize>,
    sent: HashMap<usize, usize>,
    loop_behavior: LoopBehavior,
    segment: Segment,
    update_segment: bool,
}

impl ModulationOp {
    pub fn new(
        buf: Vec<EmitIntensity>,
        freq_div: u32,
        loop_behavior: LoopBehavior,
        segment: Segment,
        update_segment: bool,
    ) -> Self {
        Self {
            buf,
            freq_div,
            remains: HashMap::new(),
            sent: HashMap::new(),
            loop_behavior,
            segment,
            update_segment,
        }
    }
}

impl Operation for ModulationOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert!(self.remains[&device.idx()] > 0);

        let sent = self.sent[&device.idx()];

        let offset = if sent == 0 {
            std::mem::size_of::<ModulationHead>()
        } else {
            std::mem::size_of::<ModulationSubseq>()
        };

        let mod_size = (self.buf.len() - sent).min(tx.len() - offset);
        assert!(mod_size > 0);

        if sent == 0 {
            let d = cast::<ModulationHead>(tx);
            d.tag = TypeTag::Modulation;
            d.flag = ModulationControlFlags::MOD_BEGIN;
            d.size = mod_size as u16;
            d.freq_div = self.freq_div;
            d.rep = self.loop_behavior.to_rep();
            d.segment = self.segment as u32;
        } else {
            let d = cast::<ModulationSubseq>(tx);
            d.tag = TypeTag::Modulation;
            d.flag = ModulationControlFlags::NONE;
            d.size = mod_size as u16;
        }

        if sent + mod_size == self.buf.len() {
            let d = cast::<ModulationSubseq>(tx);
            d.flag.set(ModulationControlFlags::MOD_END, true);
            d.flag
                .set(ModulationControlFlags::UPDATE_SEGMENT, self.update_segment);
        }

        unsafe {
            std::ptr::copy_nonoverlapping(
                self.buf[sent..].as_ptr(),
                tx[offset..].as_mut_ptr() as _,
                mod_size,
            )
        }

        self.sent.insert(device.idx(), sent + mod_size);
        if sent == 0 {
            Ok(std::mem::size_of::<ModulationHead>() + mod_size)
        } else {
            Ok(std::mem::size_of::<ModulationSubseq>() + mod_size)
        }
    }

    fn required_size(&self, device: &Device) -> usize {
        if self.sent[&device.idx()] == 0 {
            std::mem::size_of::<ModulationHead>() + 1
        } else {
            std::mem::size_of::<ModulationSubseq>() + 1
        }
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if !(2..=MOD_BUF_SIZE_MAX).contains(&self.buf.len()) {
            return Err(AUTDInternalError::ModulationSizeOutOfRange(self.buf.len()));
        }

        self.remains = geometry
            .devices()
            .map(|device| (device.idx(), self.buf.len()))
            .collect();
        self.sent = geometry.devices().map(|device| (device.idx(), 0)).collect();

        Ok(())
    }

    fn remains(&self, device: &Device) -> usize {
        self.remains[&device.idx()]
    }

    fn commit(&mut self, device: &Device) {
        self.remains
            .insert(device.idx(), self.buf.len() - self.sent[&device.idx()]);
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;

    use super::*;
    use crate::{
        fpga::{SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
        geometry::tests::create_geometry,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test() {
        const MOD_SIZE: usize = 100;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; (std::mem::size_of::<ModulationHead>() + MOD_SIZE) * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let buf: Vec<EmitIntensity> = (0..MOD_SIZE)
            .map(|_| EmitIntensity::new(rng.gen()))
            .collect();
        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
        let loop_behavior = LoopBehavior::Infinite;
        let segment = Segment::S0;
        let rep = loop_behavior.to_rep();

        let mut op = ModulationOp::new(buf.clone(), freq_div, loop_behavior, segment, true);

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                std::mem::size_of::<ModulationHead>() + 1
            )
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), MOD_SIZE));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE)..]
                ),
                Ok(std::mem::size_of::<ModulationHead>() + MOD_SIZE)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE)],
                TypeTag::Modulation as u8
            );
            let flag = tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 1];
            assert_ne!(flag & ModulationControlFlags::MOD_BEGIN.bits(), 0x00);
            assert_ne!(flag & ModulationControlFlags::MOD_END.bits(), 0x00);

            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 2],
                (MOD_SIZE & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 3],
                ((MOD_SIZE >> 8) & 0xFF) as u8
            );

            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 4],
                (freq_div & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 5],
                ((freq_div >> 8) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 6],
                ((freq_div >> 16) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 7],
                ((freq_div >> 24) & 0xFF) as u8
            );

            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 8],
                (rep & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 9],
                ((rep >> 8) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 10],
                ((rep >> 16) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 11],
                ((rep >> 24) & 0xFF) as u8
            );

            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 12],
                segment as u8
            );
            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 13],
                0x00
            );
            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 14],
                0x00
            );
            assert_eq!(
                tx[dev.idx() * (std::mem::size_of::<ModulationHead>() + MOD_SIZE) + 15],
                0x00
            );

            tx.iter()
                .skip((std::mem::size_of::<ModulationHead>() + MOD_SIZE) * dev.idx())
                .skip(std::mem::size_of::<ModulationHead>())
                .zip(buf.iter())
                .for_each(|(&d, &m)| {
                    assert_eq!(d, m.value());
                })
        });
    }

    #[test]
    fn test_div() {
        const FRAME_SIZE: usize = 30;
        const MOD_SIZE: usize = FRAME_SIZE - std::mem::size_of::<ModulationHead>()
            + (FRAME_SIZE - std::mem::size_of::<ModulationSubseq>()) * 2;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let buf: Vec<EmitIntensity> = (0..MOD_SIZE)
            .map(|_| EmitIntensity::new(rng.gen()))
            .collect();

        let mut op = ModulationOp::new(
            buf.clone(),
            SAMPLING_FREQ_DIV_MIN,
            LoopBehavior::Infinite,
            Segment::S0,
            true,
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
            .for_each(|dev| assert_eq!(op.remains(dev), MOD_SIZE));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                ),
                Ok(FRAME_SIZE)
            );
            op.commit(dev);
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.remains(dev),
                MOD_SIZE - (FRAME_SIZE - std::mem::size_of::<ModulationHead>())
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::Modulation as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_ne!(flag & ModulationControlFlags::MOD_BEGIN.bits(), 0x00);
            assert_eq!(flag & ModulationControlFlags::MOD_END.bits(), 0x00);

            let mod_size = FRAME_SIZE - std::mem::size_of::<ModulationHead>();
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 2], (mod_size & 0xFF) as u8);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 3],
                ((mod_size >> 8) & 0xFF) as u8
            );

            tx.iter()
                .skip(FRAME_SIZE * dev.idx())
                .skip(std::mem::size_of::<ModulationHead>())
                .zip(buf.iter().take(mod_size))
                .for_each(|(&d, &m)| {
                    assert_eq!(d, m.value());
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
            op.commit(dev);
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.remains(dev),
                MOD_SIZE
                    - (FRAME_SIZE - std::mem::size_of::<ModulationHead>())
                    - (FRAME_SIZE - std::mem::size_of::<ModulationSubseq>())
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::Modulation as u8);
            let flag = tx[dev.idx() * FRAME_SIZE];
            assert_eq!(flag & ModulationControlFlags::MOD_BEGIN.bits(), 0x00);
            assert_eq!(flag & ModulationControlFlags::MOD_END.bits(), 0x00);

            let mod_size = FRAME_SIZE - std::mem::size_of::<ModulationSubseq>();
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 2], (mod_size & 0xFF) as u8);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 3],
                ((mod_size >> 8) & 0xFF) as u8
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
                    assert_eq!(d, m.value());
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
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::Modulation as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_eq!(flag & ModulationControlFlags::MOD_BEGIN.bits(), 0x00);
            assert_ne!(flag & ModulationControlFlags::MOD_END.bits(), 0x00);

            let mod_size = FRAME_SIZE - std::mem::size_of::<ModulationSubseq>();
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 2], (mod_size & 0xFF) as u8);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 3],
                ((mod_size >> 8) & 0xFF) as u8
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
                    assert_eq!(d, m.value());
                })
        });
    }

    #[test]
    fn test_buffer_out_of_range() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let check = |n: usize| {
            let mut rng = rand::thread_rng();

            let buf: Vec<EmitIntensity> = (0..n).map(|_| EmitIntensity::new(rng.gen())).collect();
            let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);

            let mut op = ModulationOp::new(
                buf.clone(),
                freq_div,
                LoopBehavior::Infinite,
                Segment::S0,
                true,
            );

            op.init(&geometry)
        };

        assert_eq!(
            check(1),
            Err(AUTDInternalError::ModulationSizeOutOfRange(1))
        );
        assert_eq!(check(2), Ok(()));
        assert_eq!(check(MOD_BUF_SIZE_MAX), Ok(()));
        assert_eq!(
            check(MOD_BUF_SIZE_MAX + 1),
            Err(AUTDInternalError::ModulationSizeOutOfRange(
                MOD_BUF_SIZE_MAX + 1
            ))
        );
    }
}
