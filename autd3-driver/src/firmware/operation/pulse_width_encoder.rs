use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::PWE_BUF_SIZE,
        operation::{cast, Operation, TypeTag},
    },
    geometry::Device,
};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PWEControlFlags(u8);

bitflags::bitflags! {
    impl PWEControlFlags : u8 {
        const NONE  = 0;
        const BEGIN = 1 << 0;
        const END   = 1 << 1;
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

pub struct PulseWidthEncoderOp<F: Fn(usize) -> u16> {
    f: F,
    full_width_start: u16,
    remains: usize,
}

impl<F: Fn(usize) -> u16> PulseWidthEncoderOp<F> {
    pub fn new(f: F, full_width_start: u16) -> Self {
        Self {
            f,
            full_width_start,
            remains: PWE_BUF_SIZE,
        }
    }
}

impl<F: Fn(usize) -> u16> Operation for PulseWidthEncoderOp<F> {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let sent = PWE_BUF_SIZE - self.remains;

        let offset = if sent == 0 {
            std::mem::size_of::<PWEHead>()
        } else {
            std::mem::size_of::<PWESubseq>()
        };

        let size = self.remains.min(tx.len() - offset) & !0x1;

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

        if sent + size == PWE_BUF_SIZE {
            cast::<PWESubseq>(tx).flag.set(PWEControlFlags::END, true);
        }

        tx[offset..]
            .iter_mut()
            .take(size)
            .enumerate()
            .for_each(|(i, x)| {
                *x = (self.f)(sent + i) as u8;
            });

        self.remains -= size;
        if sent == 0 {
            Ok(std::mem::size_of::<PWEHead>() + size)
        } else {
            Ok(std::mem::size_of::<PWESubseq>() + size)
        }
    }

    fn required_size(&self, _: &Device) -> usize {
        if self.remains == PWE_BUF_SIZE {
            std::mem::size_of::<PWEHead>() + 2
        } else {
            std::mem::size_of::<PWESubseq>() + 2
        }
    }

    fn is_done(&self) -> bool {
        self.remains == 0
    }
}
