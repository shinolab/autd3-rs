mod rx;
mod tx;

pub use rx::RxMessage;
pub use tx::TxDatagram;

pub fn check_if_msg_is_processed<'a>(
    tx: &'a TxDatagram,
    rx: &'a mut [RxMessage],
) -> impl Iterator<Item = bool> + 'a {
    tx.headers().zip(rx.iter()).map(|(h, r)| h.msg_id == r.ack)
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_check_if_msg_is_processed() {
        let mut tx = TxDatagram::new(3);
        let mut rx = vec![RxMessage { ack: 0, data: 0 }; 3];

        tx.header_mut(0).msg_id = 1;
        tx.header_mut(1).msg_id = 2;
        tx.header_mut(2).msg_id = 3;

        rx[0].ack = 1;
        rx[1].ack = 2;
        rx[2].ack = 3;

        check_if_msg_is_processed(&tx, &mut rx).for_each(|b| assert!(b));

        tx.header_mut(0).msg_id = 2;
        rx[2].ack = 2;

        let processed = check_if_msg_is_processed(&tx, &mut rx).collect_vec();
        assert!(!processed[0]);
        assert!(processed[1]);
        assert!(!processed[2]);
    }
}
