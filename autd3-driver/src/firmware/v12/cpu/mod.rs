mod error;

use autd3_core::link::MsgId;
pub use autd3_core::link::{Header, RxMessage, TxMessage};

pub use error::check_firmware_err;

#[doc(hidden)]
pub fn check_if_msg_is_processed(msg_id: MsgId, rx: &[RxMessage]) -> impl Iterator<Item = bool> {
    rx.iter().map(move |r| {
        msg_id.get() == r.ack().msg_id() || r.ack().err() == error::INVALID_MESSAGE_ID
    })
}

#[cfg(test)]
mod tests {
    use autd3_core::link::{Ack, MsgId};
    use itertools::Itertools;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::success(vec![
        RxMessage::new(0, Ack::new().with_err(0).with_msg_id(0)),
        RxMessage::new(0, Ack::new().with_err(0).with_msg_id(0)),
        RxMessage::new(0, Ack::new().with_err(0).with_msg_id(0)),
    ], vec![true, true, true])]
    #[case::success(vec![
        RxMessage::new(0, Ack::new().with_err(0).with_msg_id(1)),
        RxMessage::new(0, Ack::new().with_err(0).with_msg_id(0)),
        RxMessage::new(0, Ack::new().with_err(0).with_msg_id(2)),
    ], vec![false, true, false])]
    fn test_check_if_msg_is_processed(#[case] rx: Vec<RxMessage>, #[case] expect: Vec<bool>) {
        assert_eq!(
            expect,
            check_if_msg_is_processed(MsgId::new(0), &rx).collect_vec()
        );
    }
}
