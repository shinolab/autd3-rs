use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

use zerocopy::{FromBytes, IntoBytes};

impl From<Vec<autd3_core::link::RxMessage>> for RxMessage {
    fn from(value: Vec<autd3_core::link::RxMessage>) -> Self {
        Self {
            data: value.as_bytes().to_vec(),
        }
    }
}

impl FromMessage<RxMessage> for Vec<autd3_core::link::RxMessage> {
    fn from_msg(msg: RxMessage) -> Result<Self, AUTDProtoBufError> {
        Ok(
            <[autd3_core::link::RxMessage]>::ref_from_bytes(msg.data.as_bytes())
                .unwrap()
                .to_vec(),
        )
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::link::Ack;

    use super::*;

    #[test]
    fn test_rx_message() {
        let rx = (0..10)
            .map(|i| autd3_core::link::RxMessage::new(i, Ack::new().with_err(0).with_msg_id(i)))
            .collect::<Vec<_>>();
        let msg: RxMessage = rx.clone().into();
        assert_eq!(
            10 * std::mem::size_of::<autd3_core::link::RxMessage>(),
            msg.data.len()
        );
        assert_eq!(
            rx,
            Vec::<autd3_core::link::RxMessage>::from_msg(msg).unwrap()
        )
    }
}
