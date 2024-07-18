mod clear;
mod debug;
mod force_fan;
mod gain;
mod gpio_in;
mod info;
mod modulation;
mod null;
mod pulse_width_encoder;
mod reads_fpga_state;
mod segment;
mod silencer;
mod stm;
mod sync;

pub use clear::*;
pub use debug::*;
pub use force_fan::*;
pub use gain::*;
pub use gpio_in::*;
pub use info::*;
pub use modulation::*;
pub use null::*;
pub use pulse_width_encoder::*;
pub use reads_fpga_state::*;
pub use segment::*;
pub use silencer::*;
pub use stm::*;
pub use sync::*;

use crate::{
    error::AUTDInternalError,
    firmware::cpu::{TxMessage, MSG_ID_MAX},
    geometry::{Device, Geometry},
};

use super::cpu::TxDatagram;

use rayon::prelude::*;

#[derive(PartialEq, Debug)]
#[repr(u8)]
#[non_exhaustive]
pub enum TypeTag {
    NONE = 0x00,
    Clear = 0x01,
    Sync = 0x02,
    FirmwareVersion = 0x03,
    Modulation = 0x10,
    ModulationSwapSegment = 0x11,
    Silencer = 0x20,
    Gain = 0x30,
    GainSwapSegment = 0x31,
    GainSTM = 0x41,
    FociSTM = 0x42,
    GainSTMSwapSegment = 0x43,
    FociSTMSwapSegment = 0x44,
    ForceFan = 0x60,
    ReadsFPGAState = 0x61,
    ConfigPulseWidthEncoder = 0x71,
    Debug = 0xF0,
    EmulateGPIOIn = 0xF1,
}

fn cast<T>(tx: &mut [u8]) -> &mut T {
    unsafe { (tx.as_mut_ptr() as *mut T).as_mut().unwrap() }
}

pub trait Operation: Send + Sync {
    fn required_size(&self, device: &Device) -> usize;
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError>;
    fn is_done(&self) -> bool;
}

pub trait OperationGenerator {
    type O1: Operation;
    type O2: Operation;
    fn generate(&self, device: &Device) -> (Self::O1, Self::O2);
}

// GRCOV_EXCL_START
impl Operation for Box<dyn Operation> {
    fn required_size(&self, device: &Device) -> usize {
        self.as_ref().required_size(device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        self.as_mut().pack(device, tx)
    }

    fn is_done(&self) -> bool {
        self.as_ref().is_done()
    }
}
// GRCOV_EXCL_STOP

pub struct OperationHandler {}

impl OperationHandler {
    pub fn generate<G: OperationGenerator>(gen: G, geometry: &Geometry) -> Vec<(G::O1, G::O2)> {
        geometry.devices().map(|dev| gen.generate(dev)).collect()
    }

    pub fn is_done(operations: &[(impl Operation, impl Operation)]) -> bool {
        operations.iter().all(|op| op.0.is_done() && op.1.is_done())
    }

    pub fn pack(
        operations: &mut [(impl Operation, impl Operation)],
        geometry: &Geometry,
        tx: &mut TxDatagram,
        parallel_threshold: usize,
    ) -> Result<(), AUTDInternalError> {
        if geometry.num_devices() > parallel_threshold {
            geometry
                .iter()
                .zip(tx.iter_mut())
                .filter(|(dev, _)| dev.enable)
                .zip(operations.iter_mut())
                .par_bridge()
                .try_for_each(|((dev, tx), op)| {
                    let (op1, op2) = op;
                    Self::pack_op2(op1, op2, dev, tx)
                })
        } else {
            geometry
                .iter()
                .zip(tx.iter_mut())
                .filter(|(dev, _)| dev.enable)
                .zip(operations.iter_mut())
                .try_for_each(|((dev, tx), op)| {
                    let (op1, op2) = op;
                    Self::pack_op2(op1, op2, dev, tx)
                })
        }
    }

    fn pack_op2(
        op1: &mut impl Operation,
        op2: &mut impl Operation,
        dev: &Device,
        tx: &mut TxMessage,
    ) -> Result<(), AUTDInternalError> {
        match (op1.is_done(), op2.is_done()) {
            (true, true) => Result::<_, AUTDInternalError>::Ok(()),
            (true, false) => Self::pack_op(op2, dev, tx).map(|_| Ok(()))?,
            (false, true) => Self::pack_op(op1, dev, tx).map(|_| Ok(()))?,
            (false, false) => {
                let op1_size = Self::pack_op(op1, dev, tx)?;
                let t = &mut tx.payload;
                if t.len() - op1_size >= op2.required_size(dev) {
                    op2.pack(dev, &mut t[op1_size..])?;
                    tx.header.slot_2_offset = op1_size as u16;
                }
                Ok(())
            }
        }
    }

    fn pack_op(
        op: &mut impl Operation,
        dev: &Device,
        tx: &mut TxMessage,
    ) -> Result<usize, AUTDInternalError> {
        let header = &mut tx.header;
        header.msg_id = if header.msg_id == MSG_ID_MAX {
            0
        } else {
            header.msg_id + 1
        };
        header.slot_2_offset = 0;

        let t = &mut tx.payload;

        op.pack(dev, t)
    }
}

#[cfg(test)]
pub mod tests {

    use std::mem::size_of;

    use crate::{
        derive::*,
        ethercat::EC_OUTPUT_FRAME_SIZE,
        firmware::cpu::{Header, TxDatagram},
        geometry::{UnitQuaternion, Vector3},
    };

    use super::*;

    pub(crate) fn parse_tx_as<T>(tx: &[u8]) -> T {
        unsafe {
            let ptr = tx.as_ptr() as *const T;
            ptr.read()
        }
    }

    struct OperationMock {
        pub pack_size: usize,
        pub required_size: usize,
        pub num_frames: usize,
        pub broken: bool,
    }

    impl Operation for OperationMock {
        fn required_size(&self, _: &Device) -> usize {
            self.required_size
        }

        fn pack(&mut self, _: &Device, _: &mut [u8]) -> Result<usize, AUTDInternalError> {
            if self.broken {
                return Err(AUTDInternalError::NotSupported("test".to_owned()));
            }
            self.num_frames -= 1;
            Ok(self.pack_size)
        }

        fn is_done(&self) -> bool {
            self.num_frames == 0
        }
    }

    #[test]
    fn test() {
        let geometry = Geometry::new(vec![Device::new(
            0,
            UnitQuaternion::identity(),
            vec![Transducer::new(0, Vector3::zeros())],
        )]);

        let mut op = vec![(
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
        )];

        assert!(!OperationHandler::is_done(&op));

        let mut tx = TxDatagram::new(1);

        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, 0).is_ok());
        assert_eq!(op[0].0.num_frames, 2);
        assert_eq!(op[0].1.num_frames, 2);
        assert!(!OperationHandler::is_done(&op));

        op[0].0.pack_size = EC_OUTPUT_FRAME_SIZE - size_of::<Header>() - op[0].1.required_size;
        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, 0).is_ok());
        assert_eq!(op[0].0.num_frames, 1);
        assert_eq!(op[0].1.num_frames, 1);
        assert!(!OperationHandler::is_done(&op));

        op[0].0.pack_size = EC_OUTPUT_FRAME_SIZE - size_of::<Header>() - op[0].1.required_size + 1;
        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, 0).is_ok());
        assert_eq!(op[0].0.num_frames, 0);
        assert_eq!(op[0].1.num_frames, 1);
        assert!(!OperationHandler::is_done(&op));

        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, 0).is_ok());
        assert_eq!(op[0].0.num_frames, 0);
        assert_eq!(op[0].1.num_frames, 0);
        assert!(OperationHandler::is_done(&op));
    }

    #[test]
    fn test_first() {
        let geometry = Geometry::new(vec![Device::new(
            0,
            UnitQuaternion::identity(),
            vec![Transducer::new(0, Vector3::zeros())],
        )]);

        let mut op = vec![(
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
        )];

        assert!(!op[0].0.is_done());
        assert!(op[0].1.is_done());
        assert!(!OperationHandler::is_done(&op));

        let mut tx = TxDatagram::new(1);

        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, 0).is_ok());
        assert!(op[0].0.is_done());
        assert!(op[0].1.is_done());
        assert!(OperationHandler::is_done(&op));
    }

    #[test]
    fn test_second() {
        let geometry = Geometry::new(vec![Device::new(
            0,
            UnitQuaternion::identity(),
            vec![Transducer::new(0, Vector3::zeros())],
        )]);

        let mut op = vec![(
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
        )];

        assert!(op[0].0.is_done());
        assert!(!op[0].1.is_done());
        assert!(!OperationHandler::is_done(&op));

        let mut tx = TxDatagram::new(1);

        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, 0).is_ok());
        assert!(op[0].0.is_done());
        assert!(op[0].1.is_done());
        assert!(OperationHandler::is_done(&op));
    }

    #[test]
    fn test_broken_pack() {
        let geometry = Geometry::new(vec![Device::new(
            0,
            UnitQuaternion::identity(),
            vec![Transducer::new(0, Vector3::zeros())],
        )]);

        let mut op = vec![(
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
        )];

        let mut tx = TxDatagram::new(1);

        assert_eq!(
            Err(AUTDInternalError::NotSupported("test".to_owned())),
            OperationHandler::pack(&mut op, &geometry, &mut tx, 0)
        );

        op[0].0.broken = false;
        op[0].1.broken = true;

        assert_eq!(
            Err(AUTDInternalError::NotSupported("test".to_owned())),
            OperationHandler::pack(&mut op, &geometry, &mut tx, 0)
        );

        op[0].0.num_frames = 0;

        assert_eq!(
            Err(AUTDInternalError::NotSupported("test".to_owned())),
            OperationHandler::pack(&mut op, &geometry, &mut tx, 0)
        );

        op[0].0.broken = true;
        op[0].1.broken = false;

        op[0].0.num_frames = 1;
        op[0].1.num_frames = 0;

        assert_eq!(
            Err(AUTDInternalError::NotSupported("test".to_owned())),
            OperationHandler::pack(&mut op, &geometry, &mut tx, 0)
        );
    }

    #[test]
    fn test_finished() {
        let geometry = Geometry::new(vec![Device::new(
            0,
            UnitQuaternion::identity(),
            vec![Transducer::new(0, Vector3::zeros())],
        )]);

        let mut op = vec![(
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
        )];

        assert!(OperationHandler::is_done(&op));

        let mut tx = TxDatagram::new(1);

        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, 0).is_ok());
    }
}
