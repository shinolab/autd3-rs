use std::mem::size_of;

use super::{super::fpga::PWE_BUF_SIZE, Operation, OperationGenerator};
use crate::{
    datagram::PulseWidthEncoderOperationGenerator,
    firmware::{driver::NullOp, tag::TypeTag},
};

use autd3_core::{
    datagram::FirmwareLimits,
    datagram::{PulseWidth, PulseWidthError},
    gain::Intensity,
    geometry::Device,
};

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct PweMsg {
    tag: TypeTag,
    __: u8,
}

pub struct PulseWidthEncoderOp<F: Fn(Intensity) -> PulseWidth> {
    is_done: bool,
    f: F,
    limits: FirmwareLimits,
}

impl<F: Fn(Intensity) -> PulseWidth> PulseWidthEncoderOp<F> {
    pub(crate) const fn new(f: F, limits: FirmwareLimits) -> Self {
        Self {
            is_done: false,
            f,
            limits,
        }
    }
}

impl<F: Fn(Intensity) -> PulseWidth + Send + Sync> Operation<'_> for PulseWidthEncoderOp<F> {
    type Error = PulseWidthError;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::driver::write_to_tx(
            tx,
            PweMsg {
                tag: TypeTag::ConfigPulseWidthEncoderV10,
                __: 0,
            },
        );

        tx[size_of::<PweMsg>()..]
            .iter_mut()
            .take(PWE_BUF_SIZE)
            .enumerate()
            .try_for_each(|(i, dst)| {
                *dst = (self.f)(Intensity(i as u8))
                    .pulse_width::<u8>(self.limits.ultrasound_period)?;
                Ok(())
            })?;

        self.is_done = true;

        Ok(size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u8>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u8>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<H: Fn(Intensity) -> PulseWidth + Send + Sync, F: Fn(&Device) -> H> OperationGenerator<'_>
    for PulseWidthEncoderOperationGenerator<F>
{
    type O1 = PulseWidthEncoderOp<H>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new((self.f)(device), self.limits), Self::O2 {}))
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::{super::super::V10, *};
    use crate::firmware::driver::Driver;

    #[test]
    fn test() {
        let device = crate::autd3_device::tests::create_device();

        let mut tx = [0x00u8; 2 * (size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u8>())];

        let mut op = PulseWidthEncoderOp::new(|i| PulseWidth::new(i.0 as _), V10.firmware_limits());

        assert_eq!(
            size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u8>(),
            op.required_size(&device)
        );

        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::ConfigPulseWidthEncoderV10 as u8);
        assert!((0..PWE_BUF_SIZE).all(|i| i as u8 == tx[2 + i]));
    }
}
