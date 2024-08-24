pub mod acoustics;
pub mod autd3_device;
pub mod datagram;
pub mod defined;
pub mod error;
pub mod ethercat;
pub mod firmware;
pub mod geometry;
pub mod link;
pub mod utils;

#[cfg(feature = "async-trait")]
pub use async_trait::async_trait;

#[cfg(feature = "derive")]
pub mod derive {
    pub use crate::{
        datagram::{
            Datagram, DatagramS, DatagramST, Gain, GainCalcResult, GainOperationGenerator,
            Modulation, ModulationCalcResult, ModulationOperationGenerator, ModulationProperty,
        },
        defined::DEFAULT_TIMEOUT,
        error::AUTDInternalError,
        firmware::fpga::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
        geometry::Geometry,
    };
    pub use autd3_derive::{Builder, Gain, Modulation};
}
