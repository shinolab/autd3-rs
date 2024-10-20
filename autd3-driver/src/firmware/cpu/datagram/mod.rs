mod rx;
mod tx;

pub use rx::RxMessage;
pub use tx::{TxDatagram, TxMessage};

pub fn check_if_msg_is_processed<'a>(
    tx: &'a TxDatagram,
    rx: &'a mut [RxMessage],
) -> impl Iterator<Item = bool> + 'a {
    tx.iter()
        .zip(rx.iter())
        .map(|(tx, r)| tx.header().msg_id == r.ack())
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[rstest::fixture]
    fn tx() -> TxDatagram {
        let mut tx = TxDatagram::new(3);
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
        #[case] mut rx: Vec<RxMessage>,
        #[case] expect: Vec<bool>,
        tx: TxDatagram,
    ) {
        assert_eq!(
            expect,
            check_if_msg_is_processed(&tx, &mut rx).collect_vec()
        );
    }
}
