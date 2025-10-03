use std::mem::size_of;

use super::OperationGenerator;
use crate::{
    datagram::{GPIOOutputType, GPIOOutputs},
    error::AUTDDriverError,
    firmware::{
        driver::{NullOp, Operation},
        tag::TypeTag,
    },
};

use autd3_core::{firmware::GPIOOut, geometry::Device};

use zerocopy::{Immutable, IntoBytes};

#[allow(dead_code)]
#[derive(IntoBytes, Immutable, Clone, Copy)]
pub struct GPIOOutValue(u64);

impl GPIOOutValue {
    pub const fn new(value: u64, tag: u8) -> Self {
        Self((value & 0x00FFFFFFFFFFFFFF) | (tag as u64 & 0xFF) << 56)
    }
}

fn convert(ty: Option<GPIOOutputType<'_>>) -> Result<GPIOOutValue, AUTDDriverError> {
    Ok(GPIOOutValue::new(
        match ty {
            None
            | Some(GPIOOutputType::BaseSignal)
            | Some(GPIOOutputType::Thermo)
            | Some(GPIOOutputType::ForceFan)
            | Some(GPIOOutputType::Sync)
            | Some(GPIOOutputType::ModSegment)
            | Some(GPIOOutputType::StmSegment)
            | Some(GPIOOutputType::IsStmMode) => 0,
            Some(GPIOOutputType::PwmOut(tr)) => tr.idx() as _,
            Some(GPIOOutputType::ModIdx(idx)) | Some(GPIOOutputType::StmIdx(idx)) => idx as _,
            Some(GPIOOutputType::SysTimeEq(time)) => {
                crate::firmware::v10::fpga::ec_time_to_sys_time(time) >> 9
            }
            Some(GPIOOutputType::Direct(v)) => v as _,
            Some(GPIOOutputType::SyncDiff) => {
                return Err(AUTDDriverError::UnsupportedGPIOOutputType(
                    "SyncDiff is supported from v12".to_string(),
                ));
            }
        },
        match ty {
            Some(GPIOOutputType::BaseSignal) => 0x01,
            Some(GPIOOutputType::Thermo) => 0x02,
            Some(GPIOOutputType::ForceFan) => 0x03,
            Some(GPIOOutputType::Sync) => 0x10,
            Some(GPIOOutputType::ModSegment) => 0x20,
            Some(GPIOOutputType::ModIdx(_)) => 0x21,
            Some(GPIOOutputType::StmSegment) => 0x50,
            Some(GPIOOutputType::StmIdx(_)) => 0x51,
            Some(GPIOOutputType::IsStmMode) => 0x52,
            Some(GPIOOutputType::SysTimeEq(_)) => 0x60,
            Some(GPIOOutputType::PwmOut(_)) => 0xE0,
            Some(GPIOOutputType::Direct(_)) => 0xF0,
            _ => 0x00,
        },
    ))
}

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct GPIOOutputMsg {
    tag: TypeTag,
    __: [u8; 7],
    value: [GPIOOutValue; 4],
}

pub struct GPIOOutputsOp {
    is_done: bool,
    value: [Result<GPIOOutValue, AUTDDriverError>; 4],
}

impl GPIOOutputsOp {
    pub(crate) const fn new(value: [Result<GPIOOutValue, AUTDDriverError>; 4]) -> Self {
        Self {
            is_done: false,
            value,
        }
    }
}

impl Operation<'_> for GPIOOutputsOp {
    type Error = AUTDDriverError;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::driver::write_to_tx(
            tx,
            GPIOOutputMsg {
                tag: TypeTag::Debug,
                __: [0; 7],
                value: [
                    self.value[0].clone()?,
                    self.value[1].clone()?,
                    self.value[2].clone()?,
                    self.value[3].clone()?,
                ],
            },
        );

        self.is_done = true;
        Ok(size_of::<GPIOOutputMsg>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<GPIOOutputMsg>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<F: Fn(&Device, GPIOOut) -> Option<GPIOOutputType> + Send + Sync> OperationGenerator<'_>
    for GPIOOutputs<F>
{
    type O1 = GPIOOutputsOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::new(
                [GPIOOut::O0, GPIOOut::O1, GPIOOut::O2, GPIOOut::O3]
                    .map(|gpio| convert((self.f)(device, gpio))),
            ),
            Self::O2 {},
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpio_output_op() {
        const FRAME_SIZE: usize = size_of::<GPIOOutputMsg>();

        let device = crate::autd3_device::tests::create_device();
        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut op = GPIOOutputsOp::new([
            Ok(GPIOOutValue::new(0x02030405060708, 0x01)),
            Ok(GPIOOutValue::new(0x12131415161718, 0x11)),
            Ok(GPIOOutValue::new(0x20304050607080, 0x10)),
            Ok(GPIOOutValue::new(0x21314151617181, 0x11)),
        ]);

        assert_eq!(size_of::<GPIOOutputMsg>(), op.required_size(&device));
        assert_eq!(Ok(size_of::<GPIOOutputMsg>()), op.pack(&device, &mut tx));
        assert!(op.is_done());
        assert_eq!(TypeTag::Debug as u8, tx[0]);
        assert_eq!(0x08, tx[8]);
        assert_eq!(0x07, tx[9]);
        assert_eq!(0x06, tx[10]);
        assert_eq!(0x05, tx[11]);
        assert_eq!(0x04, tx[12]);
        assert_eq!(0x03, tx[13]);
        assert_eq!(0x02, tx[14]);
        assert_eq!(0x01, tx[15]);
        assert_eq!(0x18, tx[16]);
        assert_eq!(0x17, tx[17]);
        assert_eq!(0x16, tx[18]);
        assert_eq!(0x15, tx[19]);
        assert_eq!(0x14, tx[20]);
        assert_eq!(0x13, tx[21]);
        assert_eq!(0x12, tx[22]);
        assert_eq!(0x11, tx[23]);
        assert_eq!(0x80, tx[24]);
        assert_eq!(0x70, tx[25]);
        assert_eq!(0x60, tx[26]);
        assert_eq!(0x50, tx[27]);
        assert_eq!(0x40, tx[28]);
        assert_eq!(0x30, tx[29]);
        assert_eq!(0x20, tx[30]);
        assert_eq!(0x10, tx[31]);
        assert_eq!(0x81, tx[32]);
        assert_eq!(0x71, tx[33]);
        assert_eq!(0x61, tx[34]);
        assert_eq!(0x51, tx[35]);
        assert_eq!(0x41, tx[36]);
        assert_eq!(0x31, tx[37]);
        assert_eq!(0x21, tx[38]);
        assert_eq!(0x11, tx[39]);
    }
}
