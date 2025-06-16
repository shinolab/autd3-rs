mod boxed;
mod clear;
mod cpu_gpio_out;
mod force_fan;
mod fpga_gpio_out;
mod gain;
mod gpio_in;
mod group;
mod info;
mod modulation;
mod nop;
mod null;
mod phase_corr;
mod pulse_width_encoder;
mod reads_fpga_state;
mod segment;
mod silencer;
mod stm;
mod sync;
mod tuple;

use super::Version;

use autd3_core::link::{MsgId, TxMessage};

pub use boxed::{BoxedDatagram, DOperationGenerator};

pub(crate) use null::NullOp;

use crate::{
    error::AUTDDriverError,
    geometry::{Device, Geometry},
};

use rayon::prelude::*;

#[doc(hidden)]
pub trait Operation: Send + Sync {
    type Error: std::error::Error;

    #[must_use]
    fn required_size(&self, device: &Device) -> usize;
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, Self::Error>;
    #[must_use]
    fn is_done(&self) -> bool;
}

#[doc(hidden)]
pub trait OperationGenerator {
    type O1: Operation;
    type O2: Operation;

    #[must_use]
    fn generate(&mut self, device: &Device, version: Version) -> Option<(Self::O1, Self::O2)>;
}

#[doc(hidden)]
pub struct OperationHandler {}

impl OperationHandler {
    #[must_use]
    pub fn generate<G: OperationGenerator>(
        mut generator: G,
        geometry: &Geometry,
        version: Version,
    ) -> Vec<Option<(G::O1, G::O2)>> {
        geometry
            .iter()
            .map(|dev| generator.generate(dev, version))
            .collect()
    }

    #[must_use]
    pub fn is_done<O1, O2>(operations: &[Option<(O1, O2)>]) -> bool
    where
        O1: Operation,
        O2: Operation,
    {
        operations.iter().all(|op| {
            op.as_ref()
                .is_none_or(|(op1, op2)| op1.is_done() && op2.is_done())
        })
    }

    pub fn pack<O1, O2>(
        msg_id: MsgId,
        operations: &mut [Option<(O1, O2)>],
        geometry: &Geometry,
        tx: &mut [TxMessage],
        parallel: bool,
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation,
        O2: Operation,
        AUTDDriverError: From<O1::Error> + From<O2::Error>,
    {
        if parallel {
            geometry
                .iter()
                .zip(tx.iter_mut())
                .zip(operations.iter_mut())
                .par_bridge()
                .try_for_each(|((dev, tx), op)| {
                    if let Some((op1, op2)) = op {
                        Self::pack_op2(msg_id, op1, op2, dev, tx)
                    } else {
                        Ok(())
                    }
                })
        } else {
            geometry
                .iter()
                .zip(tx.iter_mut())
                .zip(operations.iter_mut())
                .try_for_each(|((dev, tx), op)| {
                    if let Some((op1, op2)) = op {
                        Self::pack_op2(msg_id, op1, op2, dev, tx)
                    } else {
                        Ok(())
                    }
                })
        }
    }

    fn pack_op2<O1, O2>(
        msg_id: MsgId,
        op1: &mut O1,
        op2: &mut O2,
        dev: &Device,
        tx: &mut TxMessage,
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation,
        O2: Operation,
        AUTDDriverError: From<O1::Error> + From<O2::Error>,
    {
        match (op1.is_done(), op2.is_done()) {
            (true, true) => Result::<_, AUTDDriverError>::Ok(()),
            (true, false) => Self::pack_op(msg_id, op2, dev, tx).map(|_| Ok(()))?,
            (false, true) => Self::pack_op(msg_id, op1, dev, tx).map(|_| Ok(()))?,
            (false, false) => {
                let op1_size = Self::pack_op(msg_id, op1, dev, tx)?;
                if tx.payload().len() - op1_size >= op2.required_size(dev) {
                    op2.pack(dev, &mut tx.payload_mut()[op1_size..])?;
                    tx.header.slot_2_offset = op1_size as u16;
                }
                Ok(())
            }
        }
    }

    fn pack_op<O>(
        msg_id: MsgId,
        op: &mut O,
        dev: &Device,
        tx: &mut TxMessage,
    ) -> Result<usize, AUTDDriverError>
    where
        O: Operation,
        AUTDDriverError: From<O::Error>,
    {
        tx.header.msg_id = msg_id;
        tx.header.slot_2_offset = 0;
        Ok(op.pack(dev, tx.payload_mut())?)
    }
}
