// mod clear;
// mod clk;
// mod debug;
// mod force_fan;
mod gain;
// mod gpio_in;
// mod info;
// mod modulation;
mod null;
// mod phase_filter;
// mod pulse_width_encoder;
// mod reads_fpga_state;
// mod segment;
// mod silencer;
// mod stm;
// mod sync;

use std::collections::HashMap;

// pub use clear::*;
// pub use clk::*;
// pub use debug::*;
// pub use force_fan::*;
pub use gain::*;
// pub use gpio_in::*;
// pub use info::*;
// pub use modulation::*;
pub use null::*;
// pub use phase_filter::*;
// pub use pulse_width_encoder::*;
// pub use reads_fpga_state::*;
// pub use segment::*;
// pub use silencer::*;
// pub use stm::*;
// pub use sync::*;

use crate::{
    error::AUTDInternalError,
    firmware::cpu::{TxMessage, MSG_ID_MAX},
    geometry::{Device, Geometry},
};

#[repr(u8)]
pub enum TypeTag {
    NONE = 0x00,
    Clear = 0x01,
    Sync = 0x02,
    FirmwareVersion = 0x03,
    ConfigFPGAClk = 0x04,
    Modulation = 0x10,
    ModulationSwapSegment = 0x11,
    Silencer = 0x20,
    Gain = 0x30,
    GainSwapSegment = 0x31,
    FocusSTM = 0x40,
    GainSTM = 0x41,
    FocusSTMSwapSegment = 0x42,
    GainSTMSwapSegment = 0x43,
    ForceFan = 0x60,
    ReadsFPGAState = 0x61,
    ConfigPulseWidthEncoder = 0x70,
    PhaseFilter = 0x80,
    Debug = 0xF0,
    EmulateGPIOIn = 0xF1,
}

fn cast<T>(tx: &mut [u8]) -> &mut T {
    unsafe { (tx.as_mut_ptr() as *mut T).as_mut().unwrap() }
}

pub trait Operation {
    fn init(&mut self, device: &Device) -> Result<(), AUTDInternalError>;
    fn required_size(&self, device: &Device) -> usize;
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError>;
    fn is_done(&self, device: &Device) -> bool;
}

// GRCOV_EXCL_START
impl Operation for Box<dyn Operation> {
    fn init(&mut self, device: &Device) -> Result<(), AUTDInternalError> {
        self.as_mut().init(device)
    }

    fn required_size(&self, device: &Device) -> usize {
        self.as_ref().required_size(device)
    }

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        self.as_mut().pack(device, tx)
    }

    fn is_done(&self, device: &Device) -> bool {
        self.as_ref().is_done(device)
    }
}
// GRCOV_EXCL_STOP

pub struct OperationHandler {}

impl OperationHandler {
    pub fn is_finished(
        op1: &mut HashMap<usize, impl Operation>,
        op2: &mut HashMap<usize, impl Operation>,
        geometry: &Geometry,
    ) -> bool {
        geometry
            .devices()
            .all(|dev| op1[&dev.idx()].is_done(dev) && op2[&dev.idx()].is_done(dev))
    }

    pub fn init(
        op1: &mut impl Operation,
        op2: &mut impl Operation,
        device: &Device,
    ) -> Result<(), AUTDInternalError> {
        op1.init(device)?;
        op2.init(device)
    }

    pub fn pack(
        op1: &mut impl Operation,
        op2: &mut impl Operation,
        device: &Device,
        tx: &mut TxMessage,
    ) -> Result<(), AUTDInternalError> {
        match (op1.is_done(device), op2.is_done(device)) {
            (true, true) => Result::<_, AUTDInternalError>::Ok(()),
            (true, false) => Self::pack_dev(op2, device, tx).map(|_| Ok(()))?,
            (false, true) => Self::pack_dev(op1, device, tx).map(|_| Ok(()))?,
            (false, false) => {
                let op1_size = Self::pack_dev(op1, device, tx)?;
                let t = &mut tx.payload;
                if t.len() - op1_size >= op2.required_size(device) {
                    op2.pack(device, &mut t[op1_size..])?;
                    tx.header.slot_2_offset = op1_size as u16;
                }
                Ok(())
            }
        }
    }

    fn pack_dev(
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
        assert!(t.len() >= op.required_size(dev));
        op.pack(dev, t)
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use autd3_derive::Gain;

    use crate::{
        datagram::GainCalcFn,
        defined::FREQ_40K,
        derive::*,
        ethercat::EC_OUTPUT_FRAME_SIZE,
        firmware::cpu::Header,
        geometry::{UnitQuaternion, Vector3},
    };

    use super::*;

    pub(crate) fn parse_tx_as<T>(tx: &[u8]) -> T {
        unsafe {
            let ptr = tx.as_ptr() as *const T;
            ptr.read()
        }
    }

    // #[derive(Modulation, Clone)]
    // pub struct TestModulation {
    //     pub buf: Vec<EmitIntensity>,
    //     pub config: SamplingConfig,
    //     pub loop_behavior: LoopBehavior,
    // }

    // impl Modulation for TestModulation {
    //     // GRCOV_EXCL_START
    //     fn calc(&self, _: &Geometry) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
    //         Ok(self.buf.clone())
    //     }
    //     // GRCOV_EXCL_STOP
    // }

    #[derive(Gain, Clone)]
    pub struct TestGain {
        pub data: HashMap<usize, Vec<Drive>>,
    }

    impl Gain for TestGain {
        // GRCOV_EXCL_START
        fn calc(
            &self,
            _geometry: &Geometry,
            _filter: GainFilter,
        ) -> Result<GainCalcFn, AUTDInternalError> {
            Ok(Box::new(|dev| {
                let data = self.data[&dev.idx()].clone();
                Box::new(move |tr| data[tr.idx()])
            }))
        }
        // GRCOV_EXCL_STOP
    }

    #[derive(Gain, Clone, Copy)]
    pub struct NullGain {}

    // GRCOV_EXCL_START
    impl Gain for NullGain {
        fn calc<'a>(
            &'a self,
            _: &'a Geometry,
            filter: GainFilter<'a>,
        ) -> Result<GainCalcFn<'a>, AUTDInternalError> {
            Ok(Self::transform(
                filter,
                Box::new(|_| Box::new(|_| Drive::null())),
            ))
        }
    }
    // GRCOV_EXCL_STOP

    #[derive(Gain, Copy)]
    pub struct ErrGain {}

    impl Clone for ErrGain {
        // GRCOV_EXCL_START
        fn clone(&self) -> Self {
            *self
        }
        // GRCOV_EXCL_STOP
    }

    impl Gain for ErrGain {
        // GRCOV_EXCL_START
        fn calc(
            &self,
            _geometry: &Geometry,
            _filter: GainFilter,
        ) -> Result<GainCalcFn, AUTDInternalError> {
            Err(AUTDInternalError::GainError("test".to_owned()))
        }
        // GRCOV_EXCL_STOP
    }

    // struct OperationMock {
    //     pub initialized: HashMap<usize, bool>,
    //     pub pack_size: HashMap<usize, usize>,
    //     pub required_size: HashMap<usize, usize>,
    //     pub num_frames: HashMap<usize, usize>,
    //     pub broken: bool,
    // }

    // impl Operation for OperationMock {
    //     fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
    //         if self.broken {
    //             return Err(AUTDInternalError::NotSupported("test".to_owned()));
    //         }
    //         self.initialized = geometry
    //             .devices()
    //             .map(|dev| (dev.idx(), true))
    //             .collect::<HashMap<_, _>>();
    //         Ok(())
    //     }

    //     fn required_size(&self, device: &Device) -> usize {
    //         self.required_size[&device.idx()]
    //     }

    //     fn pack(&mut self, device: &Device, _: &mut [u8]) -> Result<usize, AUTDInternalError> {
    //         if self.broken {
    //             return Err(AUTDInternalError::NotSupported("test".to_owned()));
    //         }
    //         *self.num_frames.get_mut(&device.idx()).unwrap() -= 1;
    //         Ok(self.pack_size[&device.idx()])
    //     }

    //     fn is_done(&self, device: &Device) -> bool {
    //         self.num_frames[&device.idx()] == 0
    //     }
    // }

    // #[test]
    // fn test() {
    //     let geometry = Geometry::new(
    //         vec![Device::new(
    //             0,
    //             vec![Transducer::new(
    //                 0,
    //                 Vector3::zeros(),
    //                 UnitQuaternion::identity(),
    //             )],
    //         )],
    //         FREQ_40K,
    //     );

    //     let mut op1 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: false,
    //     };
    //     op1.pack_size.insert(0, 1);
    //     op1.required_size.insert(0, 2);
    //     op1.num_frames.insert(0, 3);

    //     let mut op2 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: false,
    //     };
    //     op2.pack_size.insert(0, 1);
    //     op2.required_size.insert(0, 2);
    //     op2.num_frames.insert(0, 3);

    //     OperationHandler::init(&mut op1, &mut op2, &geometry).unwrap();

    //     assert!(op1.initialized[&0]);
    //     assert!(op2.initialized[&0]);
    //     assert!(!OperationHandler::is_finished(
    //         &mut op1, &mut op2, &geometry
    //     ));

    //     let mut tx = TxDatagram::new(1);

    //     OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).unwrap();
    //     assert_eq!(op1.num_frames[&0], 2);
    //     assert_eq!(op2.num_frames[&0], 2);
    //     assert!(!OperationHandler::is_finished(
    //         &mut op1, &mut op2, &geometry
    //     ));

    //     op1.pack_size.insert(
    //         0,
    //         EC_OUTPUT_FRAME_SIZE - std::mem::size_of::<Header>() - op2.required_size[&0],
    //     );

    //     OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).unwrap();
    //     assert_eq!(op1.num_frames[&0], 1);
    //     assert_eq!(op2.num_frames[&0], 1);
    //     assert!(!OperationHandler::is_finished(
    //         &mut op1, &mut op2, &geometry
    //     ));

    //     op1.pack_size.insert(
    //         0,
    //         EC_OUTPUT_FRAME_SIZE - std::mem::size_of::<Header>() - op2.required_size[&0] + 1,
    //     );
    //     OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).unwrap();
    //     assert_eq!(op1.num_frames[&0], 0);
    //     assert_eq!(op2.num_frames[&0], 1);
    //     assert!(!OperationHandler::is_finished(
    //         &mut op1, &mut op2, &geometry
    //     ));

    //     OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).unwrap();
    //     assert_eq!(op1.num_frames[&0], 0);
    //     assert_eq!(op2.num_frames[&0], 0);
    //     assert!(OperationHandler::is_finished(&mut op1, &mut op2, &geometry));
    // }

    // #[test]
    // fn test_first() {
    //     let geometry = Geometry::new(
    //         vec![Device::new(
    //             0,
    //             vec![Transducer::new(
    //                 0,
    //                 Vector3::zeros(),
    //                 UnitQuaternion::identity(),
    //             )],
    //         )],
    //         FREQ_40K,
    //     );

    //     let mut op1 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: false,
    //     };
    //     op1.pack_size.insert(0, 0);
    //     op1.required_size.insert(0, 0);
    //     op1.num_frames.insert(0, 1);

    //     let mut op2 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: false,
    //     };
    //     op2.pack_size.insert(0, 0);
    //     op2.required_size.insert(0, 0);
    //     op2.num_frames.insert(0, 0);

    //     OperationHandler::init(&mut op1, &mut op2, &geometry).unwrap();

    //     assert!(!op1.is_done(&geometry[0]));
    //     assert!(op2.is_done(&geometry[0]));
    //     assert!(!OperationHandler::is_finished(
    //         &mut op1, &mut op2, &geometry
    //     ));

    //     let mut tx = TxDatagram::new(1);

    //     assert!(OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).is_ok());

    //     assert!(op1.is_done(&geometry[0]));
    //     assert!(op2.is_done(&geometry[0]));
    //     assert!(OperationHandler::is_finished(&mut op1, &mut op2, &geometry));
    // }

    // #[test]
    // fn test_second() {
    //     let geometry = Geometry::new(
    //         vec![Device::new(
    //             0,
    //             vec![Transducer::new(
    //                 0,
    //                 Vector3::zeros(),
    //                 UnitQuaternion::identity(),
    //             )],
    //         )],
    //         FREQ_40K,
    //     );

    //     let mut op1 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: false,
    //     };
    //     op1.pack_size.insert(0, 0);
    //     op1.required_size.insert(0, 0);
    //     op1.num_frames.insert(0, 0);

    //     let mut op2 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: false,
    //     };
    //     op2.pack_size.insert(0, 0);
    //     op2.required_size.insert(0, 0);
    //     op2.num_frames.insert(0, 1);

    //     OperationHandler::init(&mut op1, &mut op2, &geometry).unwrap();

    //     assert!(op1.is_done(&geometry[0]));
    //     assert!(!op2.is_done(&geometry[0]));
    //     assert!(!OperationHandler::is_finished(
    //         &mut op1, &mut op2, &geometry
    //     ));

    //     let mut tx = TxDatagram::new(1);

    //     assert!(OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx).is_ok());

    //     assert!(op1.is_done(&geometry[0]));
    //     assert!(op2.is_done(&geometry[0]));
    //     assert!(OperationHandler::is_finished(&mut op1, &mut op2, &geometry));
    // }

    // #[test]
    // fn test_init() {
    //     let geometry = Geometry::new(
    //         vec![Device::new(
    //             0,
    //             vec![Transducer::new(
    //                 0,
    //                 Vector3::zeros(),
    //                 UnitQuaternion::identity(),
    //             )],
    //         )],
    //         FREQ_40K,
    //     );

    //     let mut op1 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: true,
    //     };
    //     op1.pack_size.insert(0, 0);
    //     op1.required_size.insert(0, 0);
    //     op1.num_frames.insert(0, 0);

    //     let mut op2 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: false,
    //     };
    //     op2.pack_size.insert(0, 0);
    //     op2.required_size.insert(0, 0);
    //     op2.num_frames.insert(0, 1);

    //     assert_eq!(
    //         OperationHandler::init(&mut op1, &mut op2, &geometry),
    //         Err(AUTDInternalError::NotSupported("test".to_owned()))
    //     );

    //     op1.broken = false;
    //     op2.broken = true;

    //     assert_eq!(
    //         OperationHandler::init(&mut op1, &mut op2, &geometry),
    //         Err(AUTDInternalError::NotSupported("test".to_owned()))
    //     );
    // }

    // #[test]
    // fn test_broken_pack() {
    //     let geometry = Geometry::new(
    //         vec![Device::new(
    //             0,
    //             vec![Transducer::new(
    //                 0,
    //                 Vector3::zeros(),
    //                 UnitQuaternion::identity(),
    //             )],
    //         )],
    //         FREQ_40K,
    //     );

    //     let mut op1 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: false,
    //     };
    //     op1.pack_size.insert(0, 0);
    //     op1.required_size.insert(0, 0);
    //     op1.num_frames.insert(0, 1);

    //     let mut op2 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: false,
    //     };
    //     op2.pack_size.insert(0, 0);
    //     op2.required_size.insert(0, 0);
    //     op2.num_frames.insert(0, 1);

    //     OperationHandler::init(&mut op1, &mut op2, &geometry).unwrap();

    //     op1.broken = true;

    //     let mut tx = TxDatagram::new(1);

    //     assert_eq!(
    //         OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx),
    //         Err(AUTDInternalError::NotSupported("test".to_owned()))
    //     );

    //     op1.broken = false;
    //     op2.broken = true;

    //     assert_eq!(
    //         OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx),
    //         Err(AUTDInternalError::NotSupported("test".to_owned()))
    //     );

    //     op1.num_frames.insert(0, 0);
    //     assert_eq!(
    //         OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx),
    //         Err(AUTDInternalError::NotSupported("test".to_owned()))
    //     );

    //     op1.broken = true;
    //     op2.broken = false;

    //     op1.num_frames.insert(0, 1);
    //     op2.num_frames.insert(0, 0);
    //     assert_eq!(
    //         OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx),
    //         Err(AUTDInternalError::NotSupported("test".to_owned()))
    //     );
    // }

    // #[test]
    // fn test_finished() {
    //     let geometry = Geometry::new(
    //         vec![Device::new(
    //             0,
    //             vec![Transducer::new(
    //                 0,
    //                 Vector3::zeros(),
    //                 UnitQuaternion::identity(),
    //             )],
    //         )],
    //         FREQ_40K,
    //     );

    //     let mut op1 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: false,
    //     };
    //     op1.pack_size.insert(0, 0);
    //     op1.required_size.insert(0, 0);
    //     op1.num_frames.insert(0, 0);

    //     let mut op2 = OperationMock {
    //         initialized: Default::default(),
    //         pack_size: Default::default(),
    //         required_size: Default::default(),
    //         num_frames: Default::default(),
    //         broken: false,
    //     };
    //     op2.pack_size.insert(0, 0);
    //     op2.required_size.insert(0, 0);
    //     op2.num_frames.insert(0, 0);

    //     OperationHandler::init(&mut op1, &mut op2, &geometry).unwrap();

    //     assert!(OperationHandler::is_finished(&mut op1, &mut op2, &geometry));

    //     let mut tx = TxDatagram::new(1);

    //     let _ = OperationHandler::pack(&mut op1, &mut op2, &geometry, &mut tx);
    // }
}
