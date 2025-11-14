use std::mem::size_of;

use crate::{
    datagram::PulseWidthEncoderOperationGenerator,
    firmware::{
        operation::{Operation, OperationGenerator, implement::null::NullOp},
        tag::TypeTag,
    },
};

use autd3_core::{
    firmware::{Intensity, PWE_BUF_SIZE, PulseWidth, PulseWidthError},
    geometry::Device,
};

#[repr(C, align(2))]
struct PweMsg {
    tag: TypeTag,
    __: u8,
}

pub struct PulseWidthEncoderOp<F: Fn(Intensity) -> PulseWidth> {
    is_done: bool,
    f: F,
}

impl<F: Fn(Intensity) -> PulseWidth> PulseWidthEncoderOp<F> {
    pub(crate) const fn new(f: F) -> Self {
        Self { is_done: false, f }
    }
}

impl<F: Fn(Intensity) -> PulseWidth + Send> Operation<'_> for PulseWidthEncoderOp<F> {
    type Error = PulseWidthError;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::operation::write_to_tx(
            tx,
            PweMsg {
                tag: TypeTag::ConfigPulseWidthEncoder,
                __: 0,
            },
        );

        tx[size_of::<PweMsg>()..]
            .chunks_mut(size_of::<u16>())
            .take(PWE_BUF_SIZE)
            .enumerate()
            .try_for_each(|(i, dst)| {
                crate::firmware::operation::write_to_tx(
                    dst,
                    (self.f)(Intensity(i as u8)).pulse_width::<u16>()?,
                );
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

impl<'a, H: Fn(Intensity) -> PulseWidth + Send, F: Fn(&'a Device) -> H> OperationGenerator<'a>
    for PulseWidthEncoderOperationGenerator<F>
{
    type O1 = PulseWidthEncoderOp<H>;
    type O2 = NullOp;

    fn generate(&mut self, device: &'a Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1::new((self.f)(device)), Self::O2 {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let device = crate::tests::create_device();

        let mut tx = [0x00u8; 2 * (size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u16>())];

        let mut op = PulseWidthEncoderOp::new(|i| PulseWidth::new(i.0 as _));

        assert_eq!(
            size_of::<PweMsg>() + PWE_BUF_SIZE * size_of::<u8>(),
            op.required_size(&device)
        );
        assert!(!op.is_done());
        assert!(op.pack(&device, &mut tx).is_ok());
        assert!(op.is_done());
        assert_eq!(tx[0], TypeTag::ConfigPulseWidthEncoder as u8);
        assert!((0..PWE_BUF_SIZE).all(|i| i as u8 == tx[2 + 2 * i]));
    }
}
