use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

use zerocopy::{FromZeros, IntoBytes};

impl ToMessage for &[autd3_driver::firmware::cpu::TxMessage] {
    type Message = TxRawData;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            data: self.as_bytes().to_vec(),
            n: self.len() as _,
        })
    }
}

impl FromMessage<TxRawData> for Vec<autd3_driver::firmware::cpu::TxMessage> {
    fn from_msg(msg: TxRawData) -> Result<Self, AUTDProtoBufError> {
        let mut tx = vec![autd3_driver::firmware::cpu::TxMessage::new_zeroed(); msg.n as _];
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
        let mut tx = vec![autd3_driver::firmware::cpu::TxMessage::new_zeroed(); 10];
        (0..10).for_each(|i| {
            tx[i].header.msg_id = rng.random();
            tx[i].header.slot_2_offset = rng.random();
        });
        let msg = tx.as_slice().to_msg(None).unwrap();
        let tx2 = Vec::<autd3_driver::firmware::cpu::TxMessage>::from_msg(msg).unwrap();
        assert_eq!(&tx, &tx2);
    }
}
