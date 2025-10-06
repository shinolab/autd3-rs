use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl From<Vec<autd3_core::link::RxMessage>> for RxMessage {
    fn from(value: Vec<autd3_core::link::RxMessage>) -> Self {
        let mut data = vec![0; value.len() * std::mem::size_of::<autd3_core::link::RxMessage>()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                value.as_ptr() as *const u8,
                data.as_mut_ptr(),
                data.len(),
            );
        }
        Self { data }
    }
}

impl FromMessage<RxMessage> for Vec<autd3_core::link::RxMessage> {
    fn from_msg(msg: RxMessage) -> Result<Self, AUTDProtoBufError> {
        let len = msg.data.len() / std::mem::size_of::<autd3_core::link::RxMessage>();
        let mut res =
            vec![autd3_core::link::RxMessage::new(0, autd3_core::link::Ack::new(0, 0)); len];
        unsafe {
            std::ptr::copy_nonoverlapping(msg.data.as_ptr(), res.as_mut_ptr() as _, msg.data.len());
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::link::Ack;

    use super::*;

    #[test]
    fn test_rx_message() {
        let rx = (0..10)
            .map(|i| autd3_core::link::RxMessage::new(i, Ack::new(i, 0)))
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
