use std::{convert::Infallible, num::NonZeroU16};

use crate::{
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use super::SilencerControlFlags;

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct SilencerFixedUpdateRate {
    tag: TypeTag,
    flag: SilencerControlFlags,
    value_intensity: u16,
    value_phase: u16,
}

pub struct SilencerFixedUpdateRateOp {
    is_done: bool,
    intensity: NonZeroU16,
    phase: NonZeroU16,
}

impl SilencerFixedUpdateRateOp {
    pub(crate) const fn new(intensity: NonZeroU16, phase: NonZeroU16) -> Self {
        Self {
            is_done: false,
            intensity,
            phase,
        }
    }
}

impl Operation for SilencerFixedUpdateRateOp {
    type Error = Infallible;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        super::super::write_to_tx(
            tx,
            SilencerFixedUpdateRate {
                tag: TypeTag::Silencer,
                flag: SilencerControlFlags::FIXED_UPDATE_RATE,
                value_intensity: self.intensity.get(),
                value_phase: self.phase.get(),
            },
        );

        self.is_done = true;
        Ok(std::mem::size_of::<SilencerFixedUpdateRate>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<SilencerFixedUpdateRate>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::firmware::operation::tests::create_device;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[test]
    fn test() {
        let device = create_device(NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<SilencerFixedUpdateRate>()];

        let mut op = SilencerFixedUpdateRateOp::new(
            NonZeroU16::new(0x1234).unwrap(),
            NonZeroU16::new(0x5678).unwrap(),
        );

        assert_eq!(
            op.required_size(&device),
            size_of::<SilencerFixedUpdateRate>()
        );
        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Silencer as u8);
        assert_eq!(tx[1], SilencerControlFlags::FIXED_UPDATE_RATE.bits());
        assert_eq!(tx[2], 0x34);
        assert_eq!(tx[3], 0x12);
        assert_eq!(tx[4], 0x78);
        assert_eq!(tx[5], 0x56);
    }
}
