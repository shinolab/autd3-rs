mod gain_stm_mode;

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
pub fn check_if_msg_is_processed<'a>(
    tx: &'a [TxMessage],
    rx: &'a [RxMessage],
) -> impl Iterator<Item = bool> + 'a {
    tx.iter()
        .zip(rx.iter())
        .map(|(tx, r)| tx.header.msg_id.get() == r.ack())
}

#[cfg(test)]
mod tests {
    use autd3_core::link::MsgId;
    use itertools::Itertools;
    use zerocopy::FromZeros;

    use super::*;

    #[rstest::fixture]
    fn tx() -> Vec<TxMessage> {
        let mut tx = vec![TxMessage::new_zeroed(); 3];
        tx[0].header.msg_id = MsgId::new(0);
        tx[1].header.msg_id = MsgId::new(1);
        tx[2].header.msg_id = MsgId::new(2);
        tx
    }

    #[rstest::rstest]
    #[test]
    #[case::success(vec![
        RxMessage::new(0,0),
        RxMessage::new(0,1),
        RxMessage::new(0,2),
    ], vec![true, true, true])]
    #[case::success(vec![
        RxMessage::new(0, 1),
        RxMessage::new(0, 1),
        RxMessage::new(0, 1),
    ], vec![false, true, false])]
    fn test_check_if_msg_is_processed(
        #[case] rx: Vec<RxMessage>,
        #[case] expect: Vec<bool>,
        tx: Vec<TxMessage>,
    ) {
        assert_eq!(expect, check_if_msg_is_processed(&tx, &rx).collect_vec());
    }
}
