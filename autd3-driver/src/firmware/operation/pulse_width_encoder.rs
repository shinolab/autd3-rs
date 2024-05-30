use std::mem::size_of;

use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{PULSE_WIDTH_MAX, PWE_BUF_SIZE},
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
    pub fn new(f: F) -> Self {
        Self {
            f,
            full_width_start: 0,
            remains: PWE_BUF_SIZE,
        }
    }
}

impl<F: Fn(usize) -> u16 + Send + Sync> Operation for PulseWidthEncoderOp<F> {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let sent = PWE_BUF_SIZE - self.remains;

        let offset = if sent == 0 {
            if !is_sorted::IsSorted::is_sorted(&mut (0..PWE_BUF_SIZE).map(|i| (self.f)(i))) {
                return Err(AUTDInternalError::InvalidPulseWidthEncoderData);
            }
            if (0..PWE_BUF_SIZE).any(|i| (self.f)(i) > PULSE_WIDTH_MAX) {
                return Err(AUTDInternalError::InvalidPulseWidthEncoderData);
            }
            self.full_width_start = (0..PWE_BUF_SIZE)
                .position(|i| (self.f)(i) == PULSE_WIDTH_MAX)
                .unwrap_or(0xFFFF) as u16;
            size_of::<PWEHead>()
        } else {
            size_of::<PWESubseq>()
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
            Ok(size_of::<PWEHead>() + size)
        } else {
            Ok(size_of::<PWESubseq>() + size)
        }
    }

    fn required_size(&self, _: &Device) -> usize {
        if self.remains == PWE_BUF_SIZE {
            size_of::<PWEHead>() + 2
        } else {
            size_of::<PWESubseq>() + 2
        }
    }

    fn is_done(&self) -> bool {
        self.remains == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::tests::create_device;

    use super::*;

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[rstest::rstest]
    #[test]
    #[case(Err(AUTDInternalError::InvalidPulseWidthEncoderData), |i| { (PWE_BUF_SIZE - i) as u16 })]
    #[case(Err(AUTDInternalError::InvalidPulseWidthEncoderData), |_| { PULSE_WIDTH_MAX + 1 })]
    #[case(Ok(()), |_| { 0 })]
    fn invalid_data(
        #[case] expected: Result<(), AUTDInternalError>,
        #[case] f: impl Fn(usize) -> u16 + Send + Sync,
    ) {
        let send = || {
            const FRAME_SIZE: usize = size_of::<PWEHead>() + NUM_TRANS_IN_UNIT * 2;

            let device = create_device(0, NUM_TRANS_IN_UNIT);
            let mut tx = vec![0x00u8; FRAME_SIZE];

            let mut op = PulseWidthEncoderOp::new(f);

            let mut first = true;
            loop {
                if first {
                    assert_eq!(size_of::<PWEHead>() + 2, op.required_size(&device));
                } else {
                    assert_eq!(size_of::<PWESubseq>() + 2, op.required_size(&device));
                }
                first = false;
                op.pack(&device, &mut tx)?;
                if op.is_done() {
                    break;
                }
            }
            Ok(())
        };

        assert_eq!(expected, send());
    }
}
