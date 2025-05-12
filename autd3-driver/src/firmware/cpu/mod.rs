mod gain_stm_mode;

use autd3_core::link::MsgId;
pub use autd3_core::link::{Header, RxMessage, TxMessage};
pub use gain_stm_mode::*;

use crate::error::AUTDDriverError;

#[doc(hidden)]
pub fn check_firmware_err(msg: &RxMessage) -> Result<(), AUTDDriverError> {
    if msg.ack() & 0x80 != 0 {
        return Err(AUTDDriverError::firmware_err(msg.ack()));
    }
    Ok(())
}

#[doc(hidden)]
pub fn check_if_msg_is_processed(msg_id: MsgId, rx: &[RxMessage]) -> impl Iterator<Item = bool> {
    rx.iter().map(move |r| msg_id.get() == r.ack())
}

#[cfg(test)]
mod tests {
    use autd3_core::link::MsgId;
    use itertools::Itertools;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::success(vec![
        RxMessage::new(0,0),
        RxMessage::new(0,0),
        RxMessage::new(0,0),
    ], vec![true, true, true])]
    #[case::success(vec![
        RxMessage::new(0, 1),
        RxMessage::new(0, 0),
        RxMessage::new(0, 1),
    ], vec![false, true, false])]
    fn test_check_if_msg_is_processed(#[case] rx: Vec<RxMessage>, #[case] expect: Vec<bool>) {
        assert_eq!(
            expect,
            check_if_msg_is_processed(MsgId::new(0), &rx).collect_vec()
        );
    }
}
