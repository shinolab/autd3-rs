use super::OperationGenerator;
use crate::firmware::driver::NullOp;
use crate::{
    datagram::{GPIOOutputType, GPIOOutputs},
    firmware::v11::operation::GPIOOutputOp,
    geometry::Device,
};

use autd3_core::datagram::GPIOOut;

fn convert(ty: Option<GPIOOutputType<'_>>) -> crate::firmware::v11::operation::GPIOOutValue {
    crate::firmware::v11::operation::GPIOOutValue::new()
        .with_value(match &ty {
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
                crate::firmware::v12::fpga::ec_time_to_sys_time(time) >> 9
            }
            Some(GPIOOutputType::Direct(v)) => *v as _,
        })
        .with_tag(match &ty {
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
        })
}

impl<F: Fn(&Device, GPIOOut) -> Option<GPIOOutputType> + Send + Sync> OperationGenerator
    for GPIOOutputs<F>
{
    type O1 = GPIOOutputOp;
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
