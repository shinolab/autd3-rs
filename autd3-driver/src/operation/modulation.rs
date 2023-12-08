/*
 * File: modulation.rs
 * Project: operation
 * Created Date: 08/01/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 27/11/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use std::{collections::HashMap, fmt};

use crate::{
    common::EmitIntensity,
    error::AUTDInternalError,
    fpga::MOD_BUF_SIZE_MAX,
    geometry::{Device, Geometry},
    operation::{Operation, TypeTag},
};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ModulationControlFlags(u8);

bitflags::bitflags! {
    impl ModulationControlFlags : u8 {
        const NONE      = 0;
        const MOD_BEGIN = 1 << 0;
        const MOD_END   = 1 << 1;
    }
}

impl fmt::Display for ModulationControlFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(ModulationControlFlags::MOD_BEGIN) {
            flags.push("MOD_BEGIN")
        }
        if self.contains(ModulationControlFlags::MOD_END) {
            flags.push("MOD_END")
        }
        if self.is_empty() {
            flags.push("NONE")
        }
        write!(
            f,
            "{}",
            flags
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        )
    }
}

pub struct ModulationOp {
    buf: Vec<EmitIntensity>,
    freq_div: u32,
    remains: HashMap<usize, usize>,
    sent: HashMap<usize, usize>,
}

impl ModulationOp {
    pub fn new(buf: Vec<EmitIntensity>, freq_div: u32) -> Self {
        Self {
            buf,
            freq_div,
            remains: HashMap::new(),
            sent: HashMap::new(),
        }
    }
}

impl Operation for ModulationOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert!(self.remains[&device.idx()] > 0);

        tx[0] = TypeTag::Modulation as u8;

        let sent = self.sent[&device.idx()];
        let mut offset = 4;
        if sent == 0 {
            offset += 4;
        }
        let mod_size = (self.buf.len() - sent).min(tx.len() - offset);
        assert!(mod_size > 0);

        let mut f = ModulationControlFlags::NONE;
        f.set(ModulationControlFlags::MOD_BEGIN, sent == 0);
        f.set(
            ModulationControlFlags::MOD_END,
            sent + mod_size == self.buf.len(),
        );
        tx[1] = f.bits();

        tx[2] = (mod_size & 0xFF) as u8;
        tx[3] = (mod_size >> 8) as u8;

        if sent == 0 {
            let freq_div = self.freq_div;
            tx[4] = (freq_div & 0xFF) as u8;
            tx[5] = ((freq_div >> 8) & 0xFF) as u8;
            tx[6] = ((freq_div >> 16) & 0xFF) as u8;
            tx[7] = ((freq_div >> 24) & 0xFF) as u8;
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
            Ok(2 + std::mem::size_of::<u16>() + std::mem::size_of::<u32>() + mod_size)
        } else {
            Ok(2 + std::mem::size_of::<u16>() + mod_size)
        }
    }

    fn required_size(&self, device: &Device) -> usize {
        if self.sent[&device.idx()] == 0 {
            std::mem::size_of::<TypeTag>()
                + std::mem::size_of::<ModulationControlFlags>()
                + std::mem::size_of::<u16>() // size
                + std::mem::size_of::<u32>() // freq_div
                + 1
        } else {
            std::mem::size_of::<TypeTag>()
                + std::mem::size_of::<ModulationControlFlags>()
                + std::mem::size_of::<u16>() // size
                + 1
        }
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if self.buf.len() < 2 || self.buf.len() > MOD_BUF_SIZE_MAX {
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
    fn mod_controll_flag() {
        assert_eq!(std::mem::size_of::<ModulationControlFlags>(), 1);

        let flags = ModulationControlFlags::MOD_BEGIN;

        let flagsc = Clone::clone(&flags);
        assert!(flagsc.contains(ModulationControlFlags::MOD_BEGIN));
        assert!(!flagsc.contains(ModulationControlFlags::MOD_END));
    }

    #[test]
    fn mod_controll_flag_fmt() {
        assert_eq!(format!("{}", ModulationControlFlags::NONE), "NONE");
        assert_eq!(
            format!("{}", ModulationControlFlags::MOD_BEGIN),
            "MOD_BEGIN"
        );
        assert_eq!(format!("{}", ModulationControlFlags::MOD_END), "MOD_END");
        assert_eq!(
            format!(
                "{}",
                ModulationControlFlags::MOD_BEGIN | ModulationControlFlags::MOD_END
            ),
            "MOD_BEGIN | MOD_END"
        );
    }

    #[test]
    fn modulation_op() {
        const MOD_SIZE: usize = 100;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; (2 + 2 + 4 + MOD_SIZE) * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let buf: Vec<EmitIntensity> = (0..MOD_SIZE)
            .map(|_| EmitIntensity::new(rng.gen()))
            .collect();
        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);

        let mut op = ModulationOp::new(buf.clone(), freq_div);

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 9));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), MOD_SIZE));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * (2 + 2 + 4 + MOD_SIZE)..]),
                Ok(2 + 2 + 4 + MOD_SIZE)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                tx[dev.idx() * (2 + 2 + 4 + MOD_SIZE)],
                TypeTag::Modulation as u8
            );
            let flag = tx[dev.idx() * (2 + 2 + 4 + MOD_SIZE) + 1];
            assert_ne!(flag & ModulationControlFlags::MOD_BEGIN.bits(), 0x00);
            assert_ne!(flag & ModulationControlFlags::MOD_END.bits(), 0x00);

            assert_eq!(
                tx[dev.idx() * (2 + 2 + 4 + MOD_SIZE) + 2],
                (MOD_SIZE & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (2 + 2 + 4 + MOD_SIZE) + 3],
                ((MOD_SIZE >> 8) & 0xFF) as u8
            );

            assert_eq!(
                tx[dev.idx() * (2 + 2 + 4 + MOD_SIZE) + 4],
                (freq_div & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (2 + 2 + 4 + MOD_SIZE) + 5],
                ((freq_div >> 8) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (2 + 2 + 4 + MOD_SIZE) + 6],
                ((freq_div >> 16) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (2 + 2 + 4 + MOD_SIZE) + 7],
                ((freq_div >> 24) & 0xFF) as u8
            );

            tx.iter()
                .skip((2 + 2 + 4 + MOD_SIZE) * dev.idx())
                .skip(8)
                .zip(buf.iter())
                .for_each(|(&d, &m)| {
                    assert_eq!(d, m.value());
                })
        });
    }

    #[test]
    fn modulation_op_div() {
        const FRAME_SIZE: usize = 30;
        const MOD_SIZE: usize = FRAME_SIZE - 4 + FRAME_SIZE * 2;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; (2 + 2 + FRAME_SIZE) * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let buf: Vec<EmitIntensity> = (0..MOD_SIZE)
            .map(|_| EmitIntensity::new(rng.gen()))
            .collect();

        let mut op = ModulationOp::new(buf.clone(), SAMPLING_FREQ_DIV_MIN);

        assert!(op.init(&geometry).is_ok());

        // First frame
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 9));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), MOD_SIZE));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx
                        [dev.idx() * (2 + 2 + FRAME_SIZE)..(dev.idx() + 1) * (2 + 2 + FRAME_SIZE)]
                ),
                Ok(2 + 2 + FRAME_SIZE)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), MOD_SIZE - (FRAME_SIZE - 4)));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                tx[dev.idx() * (2 + 2 + FRAME_SIZE)],
                TypeTag::Modulation as u8
            );
            let flag = tx[dev.idx() * (2 + 2 + FRAME_SIZE) + 1];
            assert_ne!(flag & ModulationControlFlags::MOD_BEGIN.bits(), 0x00);
            assert_eq!(flag & ModulationControlFlags::MOD_END.bits(), 0x00);

            let mod_size = FRAME_SIZE - 4;
            assert_eq!(
                tx[dev.idx() * (2 + 2 + FRAME_SIZE) + 2],
                (mod_size & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (2 + 2 + FRAME_SIZE) + 3],
                ((mod_size >> 8) & 0xFF) as u8
            );

            tx.iter()
                .skip((2 + 2 + FRAME_SIZE) * dev.idx())
                .skip(8)
                .zip(buf.iter().take(mod_size))
                .for_each(|(&d, &m)| {
                    assert_eq!(d, m.value());
                })
        });

        // Second frame
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 5));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx
                        [dev.idx() * (2 + 2 + FRAME_SIZE)..(dev.idx() + 1) * (2 + 2 + FRAME_SIZE)]
                ),
                Ok(2 + 2 + FRAME_SIZE)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), MOD_SIZE - (FRAME_SIZE - 4) - FRAME_SIZE));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                tx[dev.idx() * (2 + 2 + FRAME_SIZE)],
                TypeTag::Modulation as u8
            );
            let flag = tx[dev.idx() * (2 + 2 + FRAME_SIZE) + 1];
            assert_eq!(flag & ModulationControlFlags::MOD_BEGIN.bits(), 0x00);
            assert_eq!(flag & ModulationControlFlags::MOD_END.bits(), 0x00);

            let mod_size = FRAME_SIZE;
            assert_eq!(
                tx[dev.idx() * (2 + 2 + FRAME_SIZE) + 2],
                (mod_size & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (2 + 2 + FRAME_SIZE) + 3],
                ((mod_size >> 8) & 0xFF) as u8
            );

            tx.iter()
                .skip((2 + 2 + FRAME_SIZE) * dev.idx())
                .skip(4)
                .zip(buf.iter().skip(FRAME_SIZE - 4).take(mod_size))
                .for_each(|(&d, &m)| {
                    assert_eq!(d, m.value());
                })
        });

        // Final frame
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 5));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx
                        [dev.idx() * (2 + 2 + FRAME_SIZE)..(dev.idx() + 1) * (2 + 2 + FRAME_SIZE)]
                ),
                Ok(2 + 2 + FRAME_SIZE)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                tx[dev.idx() * (2 + 2 + FRAME_SIZE)],
                TypeTag::Modulation as u8
            );
            let flag = tx[dev.idx() * (2 + 2 + FRAME_SIZE) + 1];
            assert_eq!(flag & ModulationControlFlags::MOD_BEGIN.bits(), 0x00);
            assert_ne!(flag & ModulationControlFlags::MOD_END.bits(), 0x00);

            let mod_size = FRAME_SIZE;
            assert_eq!(
                tx[dev.idx() * (2 + 2 + FRAME_SIZE) + 2],
                (mod_size & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * (2 + 2 + FRAME_SIZE) + 3],
                ((mod_size >> 8) & 0xFF) as u8
            );

            tx.iter()
                .skip((2 + 2 + FRAME_SIZE) * dev.idx())
                .skip(4)
                .zip(buf.iter().skip(FRAME_SIZE - 4 + FRAME_SIZE).take(mod_size))
                .for_each(|(&d, &m)| {
                    assert_eq!(d, m.value());
                })
        });
    }

    #[test]
    fn modulation_op_buffer_out_of_range() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let check = |n: usize| {
            let mut rng = rand::thread_rng();

            let buf: Vec<EmitIntensity> = (0..n).map(|_| EmitIntensity::new(rng.gen())).collect();
            let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);

            let mut op = ModulationOp::new(buf.clone(), freq_div);

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
