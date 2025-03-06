use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

use zerocopy::{FromBytes, IntoBytes};

impl From<Vec<autd3_driver::firmware::cpu::RxMessage>> for RxMessage {
    fn from(value: Vec<autd3_driver::firmware::cpu::RxMessage>) -> Self {
        Self {
            data: value.as_bytes().to_vec(),
        }
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
        let msg: RxMessage = rx.clone().into();
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
