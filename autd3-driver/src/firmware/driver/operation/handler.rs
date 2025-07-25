use super::Operation;
use crate::{
    error::AUTDDriverError,
    geometry::{Device, Geometry},
};

use autd3_core::link::{MsgId, TxMessage};

use rayon::prelude::*;

#[doc(hidden)]
pub struct OperationHandler {}

impl OperationHandler {
    #[must_use]
    pub fn is_done<'a, O1, O2>(operations: &[Option<(O1, O2)>]) -> bool
    where
        O1: Operation<'a>,
        O2: Operation<'a>,
    {
        operations.iter().all(|op| {
            op.as_ref()
                .is_none_or(|(op1, op2)| op1.is_done() && op2.is_done())
        })
    }

    pub fn pack<'a, O1, O2>(
        msg_id: MsgId,
        operations: &mut [Option<(O1, O2)>],
        geometry: &'a Geometry,
        tx: &mut [TxMessage],
        parallel: bool,
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation<'a>,
        O2: Operation<'a>,
        AUTDDriverError: From<O1::Error> + From<O2::Error>,
    {
        if parallel {
            geometry
                .iter()
                .zip(tx.iter_mut())
                .zip(operations.iter_mut())
                .par_bridge()
                .try_for_each(|((dev, tx), op)| {
                    if let Some((op1, op2)) = op {
                        Self::pack_op2(msg_id, op1, op2, dev, tx)
                    } else {
                        Ok(())
                    }
                })
        } else {
            geometry
                .iter()
                .zip(tx.iter_mut())
                .zip(operations.iter_mut())
                .try_for_each(|((dev, tx), op)| {
                    if let Some((op1, op2)) = op {
                        Self::pack_op2(msg_id, op1, op2, dev, tx)
                    } else {
                        Ok(())
                    }
                })
        }
    }

    fn pack_op2<'a, O1, O2>(
        msg_id: MsgId,
        op1: &mut O1,
        op2: &mut O2,
        dev: &'a Device,
        tx: &mut TxMessage,
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation<'a>,
        O2: Operation<'a>,
        AUTDDriverError: From<O1::Error> + From<O2::Error>,
    {
        match (op1.is_done(), op2.is_done()) {
            (true, true) => Result::<_, AUTDDriverError>::Ok(()),
            (true, false) => Self::pack_op(msg_id, op2, dev, tx).map(|_| Ok(()))?,
            (false, true) => Self::pack_op(msg_id, op1, dev, tx).map(|_| Ok(()))?,
            (false, false) => {
                let op1_size = Self::pack_op(msg_id, op1, dev, tx)?;
                if tx.payload().len() - op1_size >= op2.required_size(dev) {
                    op2.pack(dev, &mut tx.payload_mut()[op1_size..])?;
                    tx.header.slot_2_offset = op1_size as u16;
                }
                Ok(())
            }
        }
    }

    fn pack_op<'a, O>(
        msg_id: MsgId,
        op: &mut O,
        dev: &'a Device,
        tx: &mut TxMessage,
    ) -> Result<usize, AUTDDriverError>
    where
        O: Operation<'a>,
        AUTDDriverError: From<O::Error>,
    {
        debug_assert!(tx.payload().len() >= op.required_size(dev));
        tx.header.msg_id = msg_id;
        tx.header.slot_2_offset = 0;
        Ok(op.pack(dev, tx.payload_mut())?)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::mem::size_of;

    use crate::{ethercat::EC_OUTPUT_FRAME_SIZE, link::Header};

    use super::*;

    use zerocopy::FromZeros;

    struct OperationMock {
        pub pack_size: usize,
        pub required_size: usize,
        pub num_frames: usize,
        pub broken: bool,
    }

    impl Operation<'_> for OperationMock {
        type Error = AUTDDriverError;

        fn required_size(&self, _: &Device) -> usize {
            self.required_size
        }

        fn pack(&mut self, _: &Device, _: &mut [u8]) -> Result<usize, AUTDDriverError> {
            if self.broken {
                return Err(AUTDDriverError::NotSupportedTag);
            }
            self.num_frames -= 1;
            Ok(self.pack_size)
        }

        fn is_done(&self) -> bool {
            self.num_frames == 0
        }
    }

    #[rstest::rstest]
    #[test]
    #[case::serial(false)]
    #[case::parallel(true)]
    fn operation_handler(#[case] parallel: bool) {
        let geometry = Geometry::new(vec![crate::autd3_device::tests::create_device()]);

        let mut op = vec![Some((
            OperationMock {
                pack_size: 1,
                required_size: 2,
                num_frames: 3,
                broken: false,
            },
            OperationMock {
                pack_size: 1,
                required_size: 2,
                num_frames: 3,
                broken: false,
            },
        ))];

        assert!(!OperationHandler::is_done(&op));

        let msg_id = MsgId::new(0);
        let mut tx = vec![TxMessage::new_zeroed(); 1];

        assert!(OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, parallel).is_ok());
        assert_eq!(op[0].as_ref().unwrap().0.num_frames, 2);
        assert_eq!(op[0].as_ref().unwrap().1.num_frames, 2);
        assert!(!OperationHandler::is_done(&op));

        op[0].as_mut().unwrap().0.pack_size =
            EC_OUTPUT_FRAME_SIZE - size_of::<Header>() - op[0].as_ref().unwrap().1.required_size;
        assert!(OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, parallel).is_ok());
        assert_eq!(op[0].as_ref().unwrap().0.num_frames, 1);
        assert_eq!(op[0].as_ref().unwrap().1.num_frames, 1);
        assert!(!OperationHandler::is_done(&op));

        op[0].as_mut().unwrap().0.pack_size =
            EC_OUTPUT_FRAME_SIZE - size_of::<Header>() - op[0].as_ref().unwrap().1.required_size
                + 1;
        assert!(OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, parallel).is_ok());
        assert_eq!(op[0].as_ref().unwrap().0.num_frames, 0);
        assert_eq!(op[0].as_ref().unwrap().1.num_frames, 1);
        assert!(!OperationHandler::is_done(&op));

        assert!(OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, parallel).is_ok());
        assert_eq!(op[0].as_ref().unwrap().0.num_frames, 0);
        assert_eq!(op[0].as_ref().unwrap().1.num_frames, 0);
        assert!(OperationHandler::is_done(&op));
    }

    #[rstest::rstest]
    #[test]
    #[case::serial(false)]
    #[case::parallel(true)]
    fn operation_handler_none(#[case] parallel: bool) {
        let geometry = Geometry::new(vec![crate::autd3_device::tests::create_device()]);

        let mut op: Vec<Option<(OperationMock, OperationMock)>> = vec![None, None];

        assert!(OperationHandler::is_done(&op));

        let msg_id = MsgId::new(0);
        let mut tx = vec![TxMessage::new_zeroed(); 1];

        assert!(OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, parallel).is_ok());
        assert!(OperationHandler::is_done(&op));
    }

    #[test]
    fn test_first() {
        let geometry = Geometry::new(vec![crate::autd3_device::tests::create_device()]);

        let mut op = vec![Some((
            OperationMock {
                pack_size: 0,
                required_size: 0,
                num_frames: 1,
                broken: false,
            },
            OperationMock {
                pack_size: 0,
                required_size: 0,
                num_frames: 0,
                broken: false,
            },
        ))];

        assert!(!op[0].as_ref().unwrap().0.is_done());
        assert!(op[0].as_ref().unwrap().1.is_done());
        assert!(!OperationHandler::is_done(&op));

        let msg_id = MsgId::new(0);
        let mut tx = vec![TxMessage::new_zeroed(); 1];

        assert!(OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false).is_ok());
        assert!(op[0].as_ref().unwrap().0.is_done());
        assert!(op[0].as_ref().unwrap().1.is_done());
        assert!(OperationHandler::is_done(&op));
    }

    #[test]
    fn test_second() {
        let geometry = Geometry::new(vec![crate::autd3_device::tests::create_device()]);

        let mut op = vec![Some((
            OperationMock {
                pack_size: 0,
                required_size: 0,
                num_frames: 0,
                broken: false,
            },
            OperationMock {
                pack_size: 0,
                required_size: 0,
                num_frames: 1,
                broken: false,
            },
        ))];

        assert!(op[0].as_ref().unwrap().0.is_done());
        assert!(!op[0].as_ref().unwrap().1.is_done());
        assert!(!OperationHandler::is_done(&op));

        let msg_id = MsgId::new(0);
        let mut tx = vec![TxMessage::new_zeroed(); 1];

        assert!(OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false).is_ok());
        assert!(op[0].as_ref().unwrap().0.is_done());
        assert!(op[0].as_ref().unwrap().1.is_done());
        assert!(OperationHandler::is_done(&op));
    }

    #[test]
    fn test_broken_pack() {
        let geometry = Geometry::new(vec![crate::autd3_device::tests::create_device()]);

        let mut op = vec![Some((
            OperationMock {
                pack_size: 0,
                required_size: 0,
                num_frames: 1,
                broken: true,
            },
            OperationMock {
                pack_size: 0,
                required_size: 0,
                num_frames: 1,
                broken: false,
            },
        ))];

        let msg_id = MsgId::new(0);
        let mut tx = vec![TxMessage::new_zeroed(); 1];

        assert_eq!(
            Err(AUTDDriverError::NotSupportedTag),
            OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false)
        );

        op[0].as_mut().unwrap().0.broken = false;
        op[0].as_mut().unwrap().1.broken = true;

        assert_eq!(
            Err(AUTDDriverError::NotSupportedTag),
            OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false)
        );

        op[0].as_mut().unwrap().0.num_frames = 0;

        assert_eq!(
            Err(AUTDDriverError::NotSupportedTag),
            OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false)
        );

        op[0].as_mut().unwrap().0.broken = true;
        op[0].as_mut().unwrap().1.broken = false;

        op[0].as_mut().unwrap().0.num_frames = 1;
        op[0].as_mut().unwrap().1.num_frames = 0;

        assert_eq!(
            Err(AUTDDriverError::NotSupportedTag),
            OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false)
        );
    }

    #[test]
    fn test_finished() {
        let geometry = Geometry::new(vec![crate::autd3_device::tests::create_device()]);

        let mut op = vec![Some((
            OperationMock {
                pack_size: 0,
                required_size: 0,
                num_frames: 0,
                broken: false,
            },
            OperationMock {
                pack_size: 0,
                required_size: 0,
                num_frames: 0,
                broken: false,
            },
        ))];

        assert!(OperationHandler::is_done(&op));

        let msg_id = MsgId::new(0);
        let mut tx = vec![TxMessage::new_zeroed(); 1];

        assert!(OperationHandler::pack(msg_id, &mut op, &geometry, &mut tx, false).is_ok());
        assert!(OperationHandler::is_done(&op));
    }
}
