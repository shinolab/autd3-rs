use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

use zerocopy::{FromBytes, IntoBytes};

impl ToMessage for Vec<autd3_driver::firmware::cpu::RxMessage> {
    type Message = RxMessage;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            data: self.as_bytes().to_vec(),
        })
    }
}

impl FromMessage<RxMessage> for Vec<autd3_driver::firmware::cpu::RxMessage> {
    fn from_msg(msg: RxMessage) -> Result<Self, AUTDProtoBufError> {
        Ok(
            <[autd3_driver::firmware::cpu::RxMessage]>::ref_from_bytes(msg.data.as_bytes())
                .unwrap()
                .to_vec(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rx_message() {
        let rx = (0..10)
            .map(|i| autd3_driver::firmware::cpu::RxMessage::new(i, i))
            .collect::<Vec<_>>();
        let msg = rx.to_msg(None).unwrap();
        assert_eq!(
            10 * std::mem::size_of::<autd3_driver::firmware::cpu::RxMessage>(),
            msg.data.len()
        );
        assert_eq!(
            rx,
            Vec::<autd3_driver::firmware::cpu::RxMessage>::from_msg(msg).unwrap()
        )
    }
}
