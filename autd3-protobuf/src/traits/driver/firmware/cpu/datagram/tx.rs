use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl From<&[autd3_core::link::TxMessage]> for TxRawData {
    fn from(value: &[autd3_core::link::TxMessage]) -> Self {
        let len = std::mem::size_of_val(value);
        let mut data = vec![0; len];
        unsafe {
            std::ptr::copy_nonoverlapping(value.as_ptr() as *const u8, data.as_mut_ptr(), len);
        }
        Self {
            data,
            n: value.len() as _,
        }
    }
}

impl FromMessage<TxRawData> for Vec<autd3_core::link::TxMessage> {
    fn from_msg(msg: TxRawData) -> Result<Self, AUTDProtoBufError> {
        let mut tx = vec![autd3_core::link::TxMessage::new(); msg.n as _];
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
        let mut tx = vec![autd3_core::link::TxMessage::new(); 10];
        (0..10).for_each(|i| {
            tx[i].header.msg_id = autd3_core::link::MsgId::new(rng.random());
            tx[i].header.slot_2_offset = rng.random();
        });
        let msg: TxRawData = tx.as_slice().into();
        let tx2 = Vec::<autd3_core::link::TxMessage>::from_msg(msg).unwrap();
        assert_eq!(&tx, &tx2);
    }
}
