use std::fmt;

use crate::{
    error::AUTDInternalError,
    firmware::operation::{cast, Operation, TypeTag},
    geometry::{Device, Geometry},
};

use super::Remains;

const BUF_SIZE: usize = 65536;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PWEControlFlags(u8);

bitflags::bitflags! {
    impl PWEControlFlags : u8 {
        const NONE           = 0;
        const BEGIN      = 1 << 0;
        const END        = 1 << 1;
    }
}

impl fmt::Display for PWEControlFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(PWEControlFlags::BEGIN) {
            flags.push("BEGIN")
        }
        if self.contains(PWEControlFlags::END) {
            flags.push("END")
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

#[repr(C, align(2))]
struct PWEHead {
    tag: TypeTag,
    flag: PWEControlFlags,
    size: u16,
    full_width_start: u16,
}

#[repr(C, align(2))]
struct PWESubseq {
    tag: TypeTag,
    flag: PWEControlFlags,
    size: u16,
}

pub struct ConfigurePulseWidthEncoderOp {
    buf: Vec<u8>,
    full_width_start: u16,
    remains: Remains,
}

impl ConfigurePulseWidthEncoderOp {
    pub fn new(buf: Vec<u8>, full_width_start: u16) -> Self {
        Self {
            buf,
            full_width_start,
            remains: Default::default(),
        }
    }
}

impl Operation for ConfigurePulseWidthEncoderOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let sent = BUF_SIZE - self.remains[device];

        let offset = if sent == 0 {
            std::mem::size_of::<PWEHead>()
        } else {
            std::mem::size_of::<PWESubseq>()
        };

        let size = (self.buf.len() - sent).min(tx.len() - offset) & !0x1;
        assert!(size > 0);

        if sent == 0 {
            *cast::<PWEHead>(tx) = PWEHead {
                tag: TypeTag::ConfigPulseWidthEncoder,
                flag: PWEControlFlags::BEGIN,
                size: size as u16,
                full_width_start: self.full_width_start,
            };
        } else {
            *cast::<PWESubseq>(tx) = PWESubseq {
                tag: TypeTag::ConfigPulseWidthEncoder,
                flag: PWEControlFlags::NONE,
                size: size as u16,
            };
        }

        if sent + size == self.buf.len() {
            cast::<PWESubseq>(tx).flag.set(PWEControlFlags::END, true);
        }

        unsafe {
            std::ptr::copy_nonoverlapping(
                self.buf[sent..].as_ptr(),
                tx[offset..].as_mut_ptr() as _,
                size,
            )
        }

        self.remains.send(device, size);
        if sent == 0 {
            Ok(std::mem::size_of::<PWEHead>() + size)
        } else {
            Ok(std::mem::size_of::<PWESubseq>() + size)
        }
    }

    fn required_size(&self, device: &Device) -> usize {
        if self.remains[device] == BUF_SIZE {
            std::mem::size_of::<PWEHead>() + 2
        } else {
            std::mem::size_of::<PWESubseq>() + 2
        }
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if self.buf.len() != BUF_SIZE {
            return Err(AUTDInternalError::InvalidPulseWidthEncoderTableSize(
                self.buf.len(),
            ));
        }

        self.remains.init(geometry, BUF_SIZE);

        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}
