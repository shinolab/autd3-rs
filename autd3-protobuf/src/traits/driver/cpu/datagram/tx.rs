use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::cpu::TxDatagram {
    type Message = TxRawData;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            data: self.all_data().to_vec(),
            num_devices: self.num_devices() as _,
        }
    }
}

impl FromMessage<TxRawData> for autd3_driver::cpu::TxDatagram {
    fn from_msg(msg: &TxRawData) -> Option<Self> {
        let mut tx = autd3_driver::cpu::TxDatagram::new(msg.num_devices as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(
                msg.data.as_ptr(),
                tx.all_data_mut().as_mut_ptr(),
                msg.data.len(),
            );
        }
        Some(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_tx_datagram() {
        let mut rng = rand::thread_rng();
        let mut tx = autd3_driver::cpu::TxDatagram::new(10);
        (0..10).for_each(|i| {
            tx[i].header.msg_id = rng.gen();
            tx[i].header.slot_2_offset = rng.gen();
        });
        let msg = tx.to_msg(None);
        let tx2 = autd3_driver::cpu::TxDatagram::from_msg(&msg).unwrap();
        assert_eq!(tx.all_data(), tx2.all_data());
    }
}
