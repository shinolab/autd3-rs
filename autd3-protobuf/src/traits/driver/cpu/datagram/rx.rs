use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for Vec<autd3_driver::cpu::RxMessage> {
    type Message = RxMessage;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        let mut data = vec![0; std::mem::size_of::<autd3_driver::cpu::RxMessage>() * self.len()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.as_ptr() as *const u8,
                data.as_mut_ptr(),
                data.len(),
            );
        }
        Self::Message { data }
    }
}

impl FromMessage<RxMessage> for Vec<autd3_driver::cpu::RxMessage> {
    fn from_msg(msg: &RxMessage) -> Option<Self> {
        unsafe {
            let mut rx = vec![
                std::mem::zeroed::<autd3_driver::cpu::RxMessage>();
                msg.data.len() / std::mem::size_of::<autd3_driver::cpu::RxMessage>()
            ];
            std::ptr::copy_nonoverlapping(msg.data.as_ptr(), rx.as_mut_ptr() as _, msg.data.len());
            Some(rx)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rx_message() {
        let rx = (0..10)
            .map(|i| autd3_driver::cpu::RxMessage::new(i, i))
            .collect::<Vec<_>>();
        let msg = rx.to_msg(None);
        assert_eq!(
            10 * std::mem::size_of::<autd3_driver::cpu::RxMessage>(),
            msg.data.len()
        );
        assert_eq!(
            rx,
            Vec::<autd3_driver::cpu::RxMessage>::from_msg(&msg).unwrap()
        )
    }
}
