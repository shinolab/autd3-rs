use crate::{
    datagram::{GPIOOutputType, GPIOOutputs},
    error::AUTDDriverError,
    firmware::{
        operation::{Operation, OperationGenerator, implement::null::NullOp},
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
        crate::firmware::operation::write_to_tx(
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
                    .map(|gpio| Ok(convert((self.f)(device, gpio)))),
            ),
            Self::O2 {},
        ))
    }
}
