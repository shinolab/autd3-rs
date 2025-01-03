mod clear;
mod cpu_gpio_out;
mod debug;
mod force_fan;
mod gain;
mod gpio_in;
mod info;
mod modulation;
mod null;
mod phase_corr;
mod pulse_width_encoder;
mod reads_fpga_state;
mod segment;
mod silencer;
mod stm;
mod sync;

pub(crate) use clear::*;
pub(crate) use cpu_gpio_out::*;
pub(crate) use debug::*;
pub(crate) use force_fan::*;
pub use gain::GainContext;
pub(crate) use gain::*;
pub(crate) use gpio_in::*;
pub use info::FirmwareVersionType;
pub(crate) use info::*;
pub(crate) use modulation::*;
pub(crate) use null::*;
pub(crate) use phase_corr::*;
pub(crate) use pulse_width_encoder::*;
pub(crate) use reads_fpga_state::*;
pub use segment::SwapSegment;
pub(crate) use segment::*;
pub(crate) use silencer::*;
pub(crate) use stm::*;
pub use stm::{ControlPoint, ControlPoints, FociSTMContext, GainSTMContext};
pub(crate) use sync::*;
use zerocopy::{Immutable, IntoBytes};

use crate::{
    error::AUTDDriverError,
    firmware::cpu::{TxMessage, MSG_ID_MAX},
    geometry::{Device, Geometry},
};

use rayon::prelude::*;

#[derive(PartialEq, Debug, IntoBytes, Immutable)]
#[repr(u8)]
#[non_exhaustive]
pub(crate) enum TypeTag {
    Clear = 0x01,
    Sync = 0x02,
    FirmwareVersion = 0x03,
    Modulation = 0x10,
    ModulationSwapSegment = 0x11,
    Silencer = 0x21,
    Gain = 0x30,
    GainSwapSegment = 0x31,
    GainSTM = 0x41,
    FociSTM = 0x42,
    GainSTMSwapSegment = 0x43,
    FociSTMSwapSegment = 0x44,
    ForceFan = 0x60,
    ReadsFPGAState = 0x61,
    ConfigPulseWidthEncoder = 0x71,
    PhaseCorrection = 0x80,
    Debug = 0xF0,
    EmulateGPIOIn = 0xF1,
    CpuGPIOOut = 0xF2,
}

#[doc(hidden)]
pub trait Operation: Send + Sync {
    fn required_size(&self, device: &Device) -> usize;
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError>;
    fn is_done(&self) -> bool;
}

#[doc(hidden)]
pub trait OperationGenerator {
    type O1: Operation;
    type O2: Operation;
    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2);
}

impl Operation for Box<dyn Operation> {
    fn required_size(&self, device: &Device) -> usize {
        self.as_ref().required_size(device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        self.as_mut().pack(device, tx)
    }

    fn is_done(&self) -> bool {
        self.as_ref().is_done()
    }
}

impl Default for Box<dyn Operation> {
    fn default() -> Self {
        Box::new(NullOp::new())
    }
}

#[doc(hidden)]
pub struct OperationHandler {}

impl OperationHandler {
    pub fn generate<G: OperationGenerator>(mut gen: G, geometry: &Geometry) -> Vec<(G::O1, G::O2)> {
        geometry.devices().map(|dev| gen.generate(dev)).collect()
    }

    pub fn is_done(operations: &[(impl Operation, impl Operation)]) -> bool {
        operations.iter().all(|op| op.0.is_done() && op.1.is_done())
    }

    pub fn pack(
        operations: &mut [(impl Operation, impl Operation)],
        geometry: &Geometry,
        tx: &mut [TxMessage],
        parallel: bool,
    ) -> Result<(), AUTDDriverError> {
        if parallel {
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
    ) -> Result<(), AUTDDriverError> {
        match (op1.is_done(), op2.is_done()) {
            (true, true) => Result::<_, AUTDDriverError>::Ok(()),
            (true, false) => Self::pack_op(op2, dev, tx).map(|_| Ok(()))?,
            (false, true) => Self::pack_op(op1, dev, tx).map(|_| Ok(()))?,
            (false, false) => {
                let op1_size = Self::pack_op(op1, dev, tx)?;
                if tx.payload().len() - op1_size >= op2.required_size(dev) {
                    op2.pack(dev, &mut tx.payload_mut()[op1_size..])?;
                    tx.header_mut().slot_2_offset = op1_size as u16;
                }
                Ok(())
            }
        }
    }

    fn pack_op(
        op: &mut impl Operation,
        dev: &Device,
        tx: &mut TxMessage,
    ) -> Result<usize, AUTDDriverError> {
        tx.header_mut().msg_id += 1;
        tx.header_mut().msg_id &= MSG_ID_MAX;
        tx.header_mut().slot_2_offset = 0;
        op.pack(dev, tx.payload_mut())
    }
}

#[inline(always)]
pub(crate) fn write_to_tx<T: IntoBytes + Immutable>(tx: &mut [u8], data: T) {
    tx[..size_of::<T>()].copy_from_slice(data.as_bytes());
}

#[cfg(test)]
pub(crate) mod tests {

    use std::mem::size_of;

    use zerocopy::FromZeros;

    use crate::{
        ethercat::EC_OUTPUT_FRAME_SIZE,
        firmware::cpu::Header,
        geometry::{Point3, Transducer, UnitQuaternion},
    };

    use super::*;

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

        fn pack(&mut self, _: &Device, _: &mut [u8]) -> Result<usize, AUTDDriverError> {
            if self.broken {
                return Err(AUTDDriverError::LinkError("test".to_owned()));
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
    fn test(#[case] parallel: bool) {
        use crate::geometry::Point3;

        let geometry = Geometry::new(
            vec![Device::new(
                0,
                UnitQuaternion::identity(),
                vec![Transducer::new(0, 0, Point3::origin())],
            )],
            4,
        );

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

        let mut tx = vec![TxMessage::new_zeroed(); 1];

        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, parallel).is_ok());
        assert_eq!(op[0].0.num_frames, 2);
        assert_eq!(op[0].1.num_frames, 2);
        assert!(!OperationHandler::is_done(&op));

        op[0].0.pack_size = EC_OUTPUT_FRAME_SIZE - size_of::<Header>() - op[0].1.required_size;
        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, parallel).is_ok());
        assert_eq!(op[0].0.num_frames, 1);
        assert_eq!(op[0].1.num_frames, 1);
        assert!(!OperationHandler::is_done(&op));

        op[0].0.pack_size = EC_OUTPUT_FRAME_SIZE - size_of::<Header>() - op[0].1.required_size + 1;
        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, parallel).is_ok());
        assert_eq!(op[0].0.num_frames, 0);
        assert_eq!(op[0].1.num_frames, 1);
        assert!(!OperationHandler::is_done(&op));

        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, parallel).is_ok());
        assert_eq!(op[0].0.num_frames, 0);
        assert_eq!(op[0].1.num_frames, 0);
        assert!(OperationHandler::is_done(&op));
    }

    #[test]
    fn test_first() {
        let geometry = Geometry::new(
            vec![Device::new(
                0,
                UnitQuaternion::identity(),
                vec![Transducer::new(0, 0, Point3::origin())],
            )],
            4,
        );

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

        let mut tx = vec![TxMessage::new_zeroed(); 1];

        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, false).is_ok());
        assert!(op[0].0.is_done());
        assert!(op[0].1.is_done());
        assert!(OperationHandler::is_done(&op));
    }

    #[test]
    fn test_second() {
        let geometry = Geometry::new(
            vec![Device::new(
                0,
                UnitQuaternion::identity(),
                vec![Transducer::new(0, 0, Point3::origin())],
            )],
            4,
        );

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

        let mut tx = vec![TxMessage::new_zeroed(); 1];

        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, false).is_ok());
        assert!(op[0].0.is_done());
        assert!(op[0].1.is_done());
        assert!(OperationHandler::is_done(&op));
    }

    #[test]
    fn test_broken_pack() {
        let geometry = Geometry::new(
            vec![Device::new(
                0,
                UnitQuaternion::identity(),
                vec![Transducer::new(0, 0, Point3::origin())],
            )],
            4,
        );

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

        let mut tx = vec![TxMessage::new_zeroed(); 1];

        assert_eq!(
            Err(AUTDDriverError::LinkError("test".to_owned())),
            OperationHandler::pack(&mut op, &geometry, &mut tx, false)
        );

        op[0].0.broken = false;
        op[0].1.broken = true;

        assert_eq!(
            Err(AUTDDriverError::LinkError("test".to_owned())),
            OperationHandler::pack(&mut op, &geometry, &mut tx, false)
        );

        op[0].0.num_frames = 0;

        assert_eq!(
            Err(AUTDDriverError::LinkError("test".to_owned())),
            OperationHandler::pack(&mut op, &geometry, &mut tx, false)
        );

        op[0].0.broken = true;
        op[0].1.broken = false;

        op[0].0.num_frames = 1;
        op[0].1.num_frames = 0;

        assert_eq!(
            Err(AUTDDriverError::LinkError("test".to_owned())),
            OperationHandler::pack(&mut op, &geometry, &mut tx, false)
        );
    }

    #[test]
    fn test_finished() {
        let geometry = Geometry::new(
            vec![Device::new(
                0,
                UnitQuaternion::identity(),
                vec![Transducer::new(0, 0, Point3::origin())],
            )],
            4,
        );

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

        let mut tx = vec![TxMessage::new_zeroed(); 1];

        assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, false).is_ok());
    }

    #[test]
    fn msg_id() {
        let geometry = Geometry::new(
            vec![Device::new(
                0,
                UnitQuaternion::identity(),
                vec![Transducer::new(0, 0, Point3::origin())],
            )],
            4,
        );

        let mut tx = vec![TxMessage::new_zeroed(); 1];

        for i in 0..=MSG_ID_MAX {
            assert_eq!(i, tx[0].header().msg_id);
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
            assert!(OperationHandler::pack(&mut op, &geometry, &mut tx, false).is_ok());
        }
        assert_eq!(0, tx[0].header().msg_id);
    }
}
