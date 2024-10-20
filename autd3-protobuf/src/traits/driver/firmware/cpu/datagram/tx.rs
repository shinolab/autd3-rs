use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

use zerocopy::{FromBytes, IntoBytes};

impl ToMessage for autd3_driver::firmware::cpu::TxDatagram {
    type Message = TxRawData;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            data: self.as_slice().as_bytes().to_vec(),
            n: self.len() as _,
        }
    }
}

impl FromMessage<TxRawData> for autd3_driver::firmware::cpu::TxDatagram {
    fn from_msg(msg: &TxRawData) -> Result<Self, AUTDProtoBufError> {
        let mut tx = autd3_driver::firmware::cpu::TxDatagram::new(msg.n as _);
        tx.as_mut_slice().clone_from_slice(
            <[autd3_driver::firmware::cpu::TxMessage]>::ref_from_bytes(msg.data.as_bytes())
                .unwrap(),
        );
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
            tx[i].header_mut().msg_id = rng.gen();
            tx[i].header_mut().slot_2_offset = rng.gen();
        });
        let msg = tx.to_msg(None);
        let tx2 = autd3_driver::firmware::cpu::TxDatagram::from_msg(&msg).unwrap();
        assert_eq!(&tx, &tx2);
    }
}
