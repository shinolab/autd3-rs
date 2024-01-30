mod clear;
mod debug;
mod force_fan;
pub mod gain;
mod info;
mod mod_delay;
mod modulation;
mod null;
mod reads_fpga_state;
mod silencer;
pub mod stm;
mod sync;

pub use clear::*;
pub use debug::*;
pub use force_fan::*;
pub use gain::*;
pub use info::*;
pub use mod_delay::*;
pub use modulation::*;
pub use null::*;
pub use reads_fpga_state::*;
pub use silencer::*;
pub use stm::*;
pub use sync::*;

use crate::{
    cpu::{TxDatagram, MSG_ID_MAX},
    error::AUTDInternalError,
    geometry::{Device, Geometry},
};

#[repr(u8)]
pub enum TypeTag {
    NONE = 0x00,
    Clear = 0x01,
    Sync = 0x02,
    FirmwareInfo = 0x03,
    Modulation = 0x10,
    ConfigureModDelay = 0x11,
    Silencer = 0x20,
    Gain = 0x30,
    FocusSTM = 0x40,
    GainSTM = 0x50,
    ForceFan = 0x60,
    ReadsFPGAState = 0x61,
    Debug = 0xF0,
}

fn cast<T>(tx: &mut [u8]) -> &mut T {
    unsafe { (tx.as_mut_ptr() as *mut T).as_mut().unwrap() }
}

pub trait Operation {
    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError>;
    fn required_size(&self, device: &Device) -> usize;
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError>;
    fn commit(&mut self, device: &Device);
    fn remains(&self, device: &Device) -> usize;
}

impl Operation for Box<dyn Operation> {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.as_mut().init(geometry)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn required_size(&self, device: &Device) -> usize {
        self.as_ref().required_size(device)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        self.as_mut().pack(device, tx)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn commit(&mut self, device: &Device) {
        self.as_mut().commit(device)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn remains(&self, device: &Device) -> usize {
        self.as_ref().remains(device)
    }
}

pub struct OperationHandler {}

impl OperationHandler {
    pub fn is_finished(
        op1: &mut impl Operation,
        op2: &mut impl Operation,
        geometry: &Geometry,
    ) -> bool {
        geometry
            .devices()
            .all(|dev| op1.remains(dev) == 0 && op2.remains(dev) == 0)
    }

    pub fn init(
        op1: &mut impl Operation,
        op2: &mut impl Operation,
        geometry: &Geometry,
    ) -> Result<(), AUTDInternalError> {
        op1.init(geometry)?;
        op2.init(geometry)
    }

    pub fn pack(
        op1: &mut impl Operation,
        op2: &mut impl Operation,
        geometry: &Geometry,
        tx: &mut TxDatagram,
    ) -> Result<(), AUTDInternalError> {
        geometry
            .devices()
            .map(|dev| match (op1.remains(dev), op2.remains(dev)) {
                (0, 0) => unreachable!(),
                (0, _) => {
                    Self::pack_dev(op2, dev, tx)?;
                    Ok(())
                }
                (_, 0) => {
                    Self::pack_dev(op1, dev, tx)?;
                    Ok(())
                }
                _ => {
                    let op1_size = Self::pack_dev(op1, dev, tx)?;
                    let t = tx.payload_mut(dev.idx());
                    if t.len() - op1_size >= op2.required_size(dev) {
                        op2.pack(dev, &mut t[op1_size..])?;
                        op2.commit(dev);
                        tx.header_mut(dev.idx()).slot_2_offset = op1_size as u16;
                    }
                    Ok(())
                }
            })
            .collect::<Result<Vec<_>, AUTDInternalError>>()?;
        Ok(())
    }

    fn pack_dev(
        op: &mut impl Operation,
        dev: &Device,
        tx: &mut TxDatagram,
    ) -> Result<usize, AUTDInternalError> {
        let hedaer = tx.header_mut(dev.idx());
        hedaer.msg_id = if hedaer.msg_id == MSG_ID_MAX {
            0
        } else {
            hedaer.msg_id + 1
        };
        hedaer.slot_2_offset = 0;

        let t = tx.payload_mut(dev.idx());
        assert!(t.len() >= op.required_size(dev));
        let res = op.pack(dev, t)?;
        op.commit(dev);

        Ok(res)
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use crate::{
        common::{Drive, EmitIntensity, Phase},
        cpu::{Header, EC_OUTPUT_FRAME_SIZE},
        datagram::{Gain, GainFilter},
        geometry::{Transducer, UnitQuaternion, Vector3},
    };

    use super::*;

    #[derive(Clone)]
    pub struct TestGain {
        pub data: HashMap<usize, Vec<Drive>>,
    }

    impl Gain for TestGain {
        #[cfg_attr(coverage_nightly, coverage(off))]
        fn calc(
            &self,
            _geometry: &Geometry,
            _filter: GainFilter,
        ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
            Ok(self.data.clone())
        }
    }

    #[derive(Copy)]
    pub struct NullGain {}

    impl Clone for NullGain {
        #[cfg_attr(coverage_nightly, coverage(off))]
        fn clone(&self) -> Self {
            *self
        }
    }

    impl Gain for NullGain {
        #[cfg_attr(coverage_nightly, coverage(off))]
        fn calc(
            &self,
            geometry: &Geometry,
            filter: GainFilter,
        ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
            Ok(Self::transform(geometry, filter, |_, _| Drive {
                intensity: EmitIntensity::MAX,
                phase: Phase::new(2),
            }))
        }
    }

    #[derive(Copy)]
    pub struct ErrGain {}

    impl Clone for ErrGain {
        #[cfg_attr(coverage_nightly, coverage(off))]
        fn clone(&self) -> Self {
            *self
        }
    }

    impl Gain for ErrGain {
        #[cfg_attr(coverage_nightly, coverage(off))]
        fn calc(
            &self,
            _geometry: &Geometry,
            _filter: GainFilter,
        ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
            Err(AUTDInternalError::GainError("test".to_owned()))
        }
    }

    struct OperationMock {
        pub initialized: HashMap<usize, bool>,
        pub pack_size: HashMap<usize, usize>,
        pub required_size: HashMap<usize, usize>,
        pub num_frames: HashMap<usize, usize>,
        pub broken: bool,
    }

    impl Operation for OperationMock {
        fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
            if self.broken {
                return Err(AUTDInternalError::NotSupported("test".to_owned()));
            }
            self.initialized = geometry
                .devices()
                .map(|dev| (dev.idx(), true))
                .collect::<HashMap<_, _>>();
            Ok(())
        }

        fn required_size(&self, device: &Device) -> usize {
            self.required_size[&device.idx()]
        }

        fn pack(&mut self, device: &Device, _: &mut [u8]) -> Result<usize, AUTDInternalError> {
            if self.broken {
                return Err(AUTDInternalError::NotSupported("test".to_owned()));
            }
            Ok(self.pack_size[&device.idx()])
        }

        fn commit(&mut self, device: &Device) {
            *self.num_frames.get_mut(&device.idx()).unwrap() -= 1;
        }

        fn remains(&self, device: &Device) -> usize {
            self.num_frames[&device.idx()]
        }
    }

    #[test]
    fn op_handler_test() {
        let geometry = Geometry::new(vec![Device::new(
            0,
            vec![Transducer::new(
                0,
                Vector3::zeros(),
                UnitQuaternion::identity(),
            )],
        )]);

        let mut op1 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: false,
        };
        op1.pack_size.insert(0, 1);
        op1.required_size.insert(0, 2);
        op1.num_frames.insert(0, 3);

        let mut op2 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: false,
        };
        op2.pack_size.insert(0, 1);
        op2.required_size.insert(0, 2);
        op2.num_frames.insert(0, 3);

        OperationHandler::init(&mut op1, &mut op2, &geometry).unwrap();

        assert!(op1.initialized[&0]);
        assert!(op2.initialized[&0]);
        assert!(!OperationHandler::is_finished(
            &mut op1, &mut op2, &geometry
        ));

        let mut tx = TxDatagram::new(1);

        OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).unwrap();
        assert_eq!(op1.num_frames[&0], 2);
        assert_eq!(op2.num_frames[&0], 2);
        assert!(!OperationHandler::is_finished(
            &mut op1, &mut op2, &geometry
        ));

        op1.pack_size.insert(
            0,
            EC_OUTPUT_FRAME_SIZE - std::mem::size_of::<Header>() - op2.required_size[&0],
        );

        OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).unwrap();
        assert_eq!(op1.num_frames[&0], 1);
        assert_eq!(op2.num_frames[&0], 1);
        assert!(!OperationHandler::is_finished(
            &mut op1, &mut op2, &geometry
        ));

        op1.pack_size.insert(
            0,
            EC_OUTPUT_FRAME_SIZE - std::mem::size_of::<Header>() - op2.required_size[&0] + 1,
        );
        OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).unwrap();
        assert_eq!(op1.num_frames[&0], 0);
        assert_eq!(op2.num_frames[&0], 1);
        assert!(!OperationHandler::is_finished(
            &mut op1, &mut op2, &geometry
        ));

        OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).unwrap();
        assert_eq!(op1.num_frames[&0], 0);
        assert_eq!(op2.num_frames[&0], 0);
        assert!(OperationHandler::is_finished(&mut op1, &mut op2, &geometry));
    }

    #[test]
    fn op_handler_pack_first() {
        let geometry = Geometry::new(vec![Device::new(
            0,
            vec![Transducer::new(
                0,
                Vector3::zeros(),
                UnitQuaternion::identity(),
            )],
        )]);

        let mut op1 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: false,
        };
        op1.pack_size.insert(0, 0);
        op1.required_size.insert(0, 0);
        op1.num_frames.insert(0, 1);

        let mut op2 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: false,
        };
        op2.pack_size.insert(0, 0);
        op2.required_size.insert(0, 0);
        op2.num_frames.insert(0, 0);

        OperationHandler::init(&mut op1, &mut op2, &geometry).unwrap();

        assert_eq!(op1.remains(&geometry[0]), 1);
        assert_eq!(op2.remains(&geometry[0]), 0);
        assert!(!OperationHandler::is_finished(
            &mut op1, &mut op2, &geometry
        ));

        let mut tx = TxDatagram::new(1);

        assert!(OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).is_ok());

        assert_eq!(op1.remains(&geometry[0]), 0);
        assert_eq!(op2.remains(&geometry[0]), 0);
        assert!(OperationHandler::is_finished(&mut op1, &mut op2, &geometry));
    }

    #[test]
    fn op_handler_pack_second() {
        let geometry = Geometry::new(vec![Device::new(
            0,
            vec![Transducer::new(
                0,
                Vector3::zeros(),
                UnitQuaternion::identity(),
            )],
        )]);

        let mut op1 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: false,
        };
        op1.pack_size.insert(0, 0);
        op1.required_size.insert(0, 0);
        op1.num_frames.insert(0, 0);

        let mut op2 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: false,
        };
        op2.pack_size.insert(0, 0);
        op2.required_size.insert(0, 0);
        op2.num_frames.insert(0, 1);

        OperationHandler::init(&mut op1, &mut op2, &geometry).unwrap();

        assert_eq!(op1.remains(&geometry[0]), 0);
        assert_eq!(op2.remains(&geometry[0]), 1);
        assert!(!OperationHandler::is_finished(
            &mut op1, &mut op2, &geometry
        ));

        let mut tx = TxDatagram::new(1);

        assert!(OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).is_ok());

        assert_eq!(op1.remains(&geometry[0]), 0);
        assert_eq!(op2.remains(&geometry[0]), 0);
        assert!(OperationHandler::is_finished(&mut op1, &mut op2, &geometry));
    }

    #[test]
    fn op_handler_broken_init() {
        let geometry = Geometry::new(vec![Device::new(
            0,
            vec![Transducer::new(
                0,
                Vector3::zeros(),
                UnitQuaternion::identity(),
            )],
        )]);

        let mut op1 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: true,
        };
        op1.pack_size.insert(0, 0);
        op1.required_size.insert(0, 0);
        op1.num_frames.insert(0, 0);

        let mut op2 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: false,
        };
        op2.pack_size.insert(0, 0);
        op2.required_size.insert(0, 0);
        op2.num_frames.insert(0, 1);

        assert_eq!(
            OperationHandler::init(&mut op1, &mut op2, &geometry),
            Err(AUTDInternalError::NotSupported("test".to_owned()))
        );

        op1.broken = false;
        op2.broken = true;

        assert_eq!(
            OperationHandler::init(&mut op1, &mut op2, &geometry),
            Err(AUTDInternalError::NotSupported("test".to_owned()))
        );
    }

    #[test]
    fn op_handler_broken_pack() {
        let geometry = Geometry::new(vec![Device::new(
            0,
            vec![Transducer::new(
                0,
                Vector3::zeros(),
                UnitQuaternion::identity(),
            )],
        )]);

        let mut op1 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: false,
        };
        op1.pack_size.insert(0, 0);
        op1.required_size.insert(0, 0);
        op1.num_frames.insert(0, 1);

        let mut op2 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: false,
        };
        op2.pack_size.insert(0, 0);
        op2.required_size.insert(0, 0);
        op2.num_frames.insert(0, 1);

        OperationHandler::init(&mut op1, &mut op2, &geometry).unwrap();

        op1.broken = true;

        let mut tx = TxDatagram::new(1);

        assert_eq!(
            OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx),
            Err(AUTDInternalError::NotSupported("test".to_owned()))
        );

        op1.broken = false;
        op2.broken = true;

        assert_eq!(
            OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx),
            Err(AUTDInternalError::NotSupported("test".to_owned()))
        );

        op1.num_frames.insert(0, 0);
        assert_eq!(
            OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx),
            Err(AUTDInternalError::NotSupported("test".to_owned()))
        );

        op1.broken = true;
        op2.broken = false;

        op1.num_frames.insert(0, 1);
        op2.num_frames.insert(0, 0);
        assert_eq!(
            OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx),
            Err(AUTDInternalError::NotSupported("test".to_owned()))
        );
    }

    #[test]
    #[should_panic]
    fn op_handler_pack_finished() {
        let geometry = Geometry::new(vec![Device::new(
            0,
            vec![Transducer::new(
                0,
                Vector3::zeros(),
                UnitQuaternion::identity(),
            )],
        )]);

        let mut op1 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: false,
        };
        op1.pack_size.insert(0, 0);
        op1.required_size.insert(0, 0);
        op1.num_frames.insert(0, 0);

        let mut op2 = OperationMock {
            initialized: Default::default(),
            pack_size: Default::default(),
            required_size: Default::default(),
            num_frames: Default::default(),
            broken: false,
        };
        op2.pack_size.insert(0, 0);
        op2.required_size.insert(0, 0);
        op2.num_frames.insert(0, 0);

        OperationHandler::init(&mut op1, &mut op2, &geometry).unwrap();

        assert!(OperationHandler::is_finished(&mut op1, &mut op2, &geometry));

        let mut tx = TxDatagram::new(1);

        let _ = OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx);
    }
}
