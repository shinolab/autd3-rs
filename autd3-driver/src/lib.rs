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
            Datagram, DatagramS, DatagramST, Gain, GainCache, GainCalcResult,
            GainOperationGenerator, GainTransform, IntoGainCache, IntoGainTransform,
            IntoModulationCache, IntoModulationTransform, IntoRadiationPressure, Modulation,
            ModulationCache, ModulationCalcResult, ModulationOperationGenerator,
            ModulationProperty, ModulationTransform, RadiationPressure,
        },
        defined::{rad, DEFAULT_TIMEOUT},
        error::AUTDInternalError,
        firmware::fpga::{
            Drive, EmitIntensity, LoopBehavior, Phase, SamplingConfig, Segment, TransitionMode,
        },
        geometry::{Device, Geometry, Transducer},
    };
    pub use autd3_derive::{Builder, Gain, Modulation};
    pub use itertools::Itertools;
    pub use std::collections::HashMap;
    pub use tracing;
    pub use tynm;
}
