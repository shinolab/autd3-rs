mod rx;
mod tx;

pub use rx::RxMessage;
pub use tx::TxMessage;

#[doc(hidden)]
pub fn check_if_msg_is_processed<'a>(
    tx: &'a [TxMessage],
    rx: &'a [RxMessage],
) -> impl Iterator<Item = bool> + 'a {
    tx.iter()
        .zip(rx.iter())
        .map(|(tx, r)| tx.header().msg_id == r.ack())
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use zerocopy::FromZeros;

    use super::*;

    #[rstest::fixture]
    fn tx() -> Vec<TxMessage> {
        let mut tx = vec![TxMessage::new_zeroed(); 3];
        tx[0].header_mut().msg_id = 0;
        tx[1].header_mut().msg_id = 1;
        tx[2].header_mut().msg_id = 2;
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
