use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

use zerocopy::{FromZeros, IntoBytes};

impl From<&[autd3_core::link::TxMessage]> for TxRawData {
    fn from(value: &[autd3_core::link::TxMessage]) -> Self {
        Self {
            data: value.as_bytes().to_vec(),
            n: value.len() as _,
        }
    }
}

impl FromMessage<TxRawData> for Vec<autd3_core::link::TxMessage> {
    fn from_msg(msg: TxRawData) -> Result<Self, AUTDProtoBufError> {
        let mut tx = vec![autd3_core::link::TxMessage::new_zeroed(); msg.n as _];
        unsafe {
            std::ptr::copy_nonoverlapping(msg.data.as_ptr(), tx.as_mut_ptr() as _, msg.data.len());
        }
        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_tx_datagram_unsafe() {
        let mut rng = rand::rng();
        let mut tx = vec![autd3_core::link::TxMessage::new_zeroed(); 10];
        (0..10).for_each(|i| {
            tx[i].header.msg_id = autd3_core::link::MsgId::new(rng.random());
            tx[i].header.slot_2_offset = rng.random();
        });
        let msg: TxRawData = tx.as_slice().into();
        let tx2 = Vec::<autd3_core::link::TxMessage>::from_msg(msg).unwrap();
        assert_eq!(&tx, &tx2);
    }
}
