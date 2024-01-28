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
        let mut rx = vec![
            autd3_driver::cpu::RxMessage { ack: 0, data: 0 };
            msg.data.len() / std::mem::size_of::<autd3_driver::cpu::RxMessage>()
        ];
        unsafe {
            std::ptr::copy_nonoverlapping(msg.data.as_ptr(), rx.as_mut_ptr() as _, msg.data.len());
        }
        Some(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rx_message() {
        let mut rx = vec![autd3_driver::cpu::RxMessage { ack: 0, data: 0 }; 10];
        rx[0].ack = 1;
        rx[0].data = 2;
        rx[1].ack = 3;
        rx[1].data = 4;
        let msg = rx.to_msg(None);
        assert_eq!(
            msg.data.len(),
            10 * std::mem::size_of::<autd3_driver::cpu::RxMessage>()
        );
        let rx2 = Vec::<autd3_driver::cpu::RxMessage>::from_msg(&msg).unwrap();
        assert_eq!(rx2.len(), 10);
        assert_eq!(rx2[0].ack, 1);
        assert_eq!(rx2[0].data, 2);
        assert_eq!(rx2[1].ack, 3);
        assert_eq!(rx2[1].data, 4);
    }
}
