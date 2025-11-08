use crate::{
    datagram::{GPIOOutputType, GPIOOutputs},
    error::AUTDDriverError,
    firmware::{
        operation::{Operation, OperationGenerator, implement::null::NullOp},
        tag::TypeTag,
    },
};

use autd3_core::{firmware::GPIOOut, geometry::Device};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct GPIOOutValue(u64);

impl GPIOOutValue {
    pub const fn new(value: u64, tag: u8) -> Self {
        Self((value & 0x00FFFFFFFFFFFFFF) | (tag as u64 & 0xFF) << 56)
    }
}

fn convert(ty: Option<GPIOOutputType<'_>>) -> GPIOOutValue {
    GPIOOutValue::new(
        match &ty {
            None
            | Some(GPIOOutputType::BaseSignal)
            | Some(GPIOOutputType::Thermo)
            | Some(GPIOOutputType::ForceFan)
            | Some(GPIOOutputType::Sync)
            | Some(GPIOOutputType::ModSegment)
            | Some(GPIOOutputType::StmSegment)
            | Some(GPIOOutputType::IsStmMode)
            | Some(GPIOOutputType::SyncDiff) => 0,
            Some(GPIOOutputType::PwmOut(tr)) => tr.idx() as _,
            Some(GPIOOutputType::ModIdx(idx)) | Some(GPIOOutputType::StmIdx(idx)) => *idx as _,
            Some(GPIOOutputType::SysTimeEq(time)) => {
                crate::firmware::fpga::ec_time_to_sys_time(time) >> 9
            }
            Some(GPIOOutputType::Direct(v)) => *v as _,
        },
        match &ty {
            None => 0x00,
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
            Some(GPIOOutputType::SyncDiff) => 0x70,
            Some(GPIOOutputType::PwmOut(_)) => 0xE0,
            Some(GPIOOutputType::Direct(_)) => 0xF0,
        },
    )
}

#[repr(C, align(2))]
struct GPIOOutputMsg {
    tag: TypeTag,
    __: [u8; 7],
    value: [GPIOOutValue; 4],
}

pub struct GPIOOutputsOp {
    value: Option<[GPIOOutValue; 4]>,
}

impl GPIOOutputsOp {
    pub(crate) const fn new(value: [GPIOOutValue; 4]) -> Self {
        Self { value: Some(value) }
    }
}

impl Operation<'_> for GPIOOutputsOp {
    type Error = AUTDDriverError;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, Self::Error> {
        crate::firmware::operation::write_to_tx(
            tx,
            GPIOOutputMsg {
                tag: TypeTag::FpgaGPIOOut,
                __: [0; 7],
                value: self
                    .value
                    .take()
                    .expect("GPIOOutputsOp::pack called more than once"),
            },
        );

        Ok(size_of::<GPIOOutputMsg>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<GPIOOutputMsg>()
    }

    fn is_done(&self) -> bool {
        self.value.is_none()
    }
}

impl<F: Fn(&Device, GPIOOut) -> Option<GPIOOutputType>> OperationGenerator<'_> for GPIOOutputs<F> {
    type O1 = GPIOOutputsOp;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::new(
                [GPIOOut::O0, GPIOOut::O1, GPIOOut::O2, GPIOOut::O3]
                    .map(|gpio| (self.f)(device, gpio))
                    .map(convert),
            ),
            Self::O2 {},
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;

    use super::*;

    #[test]
    fn op() {
        let device = crate::tests::create_device();

        let mut tx = [0x00u8; size_of::<GPIOOutputMsg>()];

        let value = [
            GPIOOutValue::new(0x123456789ABCDEF0, 0xF0),
            GPIOOutValue::new(0x0FEDCBA987654321, 0xE0),
            GPIOOutValue::new(0x1111111111111111, 0x02),
            GPIOOutValue::new(0x2222222222222222, 0x01),
        ];

        let mut op = GPIOOutputsOp::new(value);

        assert_eq!(op.required_size(&device), size_of::<GPIOOutputMsg>());
        assert!(!op.is_done());
        assert!(op.pack(&device, &mut tx).is_ok());
        assert!(op.is_done());
        assert_eq!(tx[0], TypeTag::FpgaGPIOOut as u8);
        assert_eq!(
            u64::from_le_bytes(
                tx[offset_of!(GPIOOutputMsg, value)..offset_of!(GPIOOutputMsg, value) + 8]
                    .try_into()
                    .unwrap()
            ),
            0xF03456789ABCDEF0u64
        );
        assert_eq!(
            u64::from_le_bytes(
                tx[offset_of!(GPIOOutputMsg, value) + 8..offset_of!(GPIOOutputMsg, value) + 16]
                    .try_into()
                    .unwrap()
            ),
            0xE0EDCBA987654321u64
        );
        assert_eq!(
            u64::from_le_bytes(
                tx[offset_of!(GPIOOutputMsg, value) + 16..offset_of!(GPIOOutputMsg, value) + 24]
                    .try_into()
                    .unwrap()
            ),
            0x0211111111111111u64
        );
        assert_eq!(
            u64::from_le_bytes(
                tx[offset_of!(GPIOOutputMsg, value) + 24..offset_of!(GPIOOutputMsg, value) + 32]
                    .try_into()
                    .unwrap()
            ),
            0x0122222222222222u64
        );
    }
}
