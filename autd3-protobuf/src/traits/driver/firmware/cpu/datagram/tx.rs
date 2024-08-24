use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::firmware::cpu::TxDatagram {
    type Message = TxRawData;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        let mut data = vec![0; self.total_len()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.as_ptr() as *const u8,
                data.as_mut_ptr(),
                self.total_len(),
            );
        }
        Self::Message {
            data,
            n: self.len() as _,
        }
    }
}

impl FromMessage<TxRawData> for autd3_driver::firmware::cpu::TxDatagram {
    fn from_msg(msg: &TxRawData) -> Result<Self, AUTDProtoBufError> {
        let mut tx = autd3_driver::firmware::cpu::TxDatagram::new(msg.n as _);
        unsafe {
            std::ptr::copy_nonoverlapping(
                msg.data.as_ptr(),
                tx.as_mut_ptr() as *mut u8,
                tx.total_len(),
            );
        }
        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_tx_datagram() {
        let mut rng = rand::thread_rng();
        let mut tx = autd3_driver::firmware::cpu::TxDatagram::new(10);
        (0..10).for_each(|i| {
            tx[i].header.msg_id = rng.gen();
            tx[i].header.slot_2_offset = rng.gen();
        });
        let msg = tx.to_msg(None);
        let tx2 = autd3_driver::firmware::cpu::TxDatagram::from_msg(&msg).unwrap();
        assert_eq!(&tx, &tx2);
    }
}
