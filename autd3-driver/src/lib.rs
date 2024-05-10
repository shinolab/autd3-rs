pub mod acoustics;
pub mod autd3_device;
pub mod datagram;
pub mod defined;
pub mod error;
pub mod ethercat;
pub mod firmware;
pub mod freq;
pub mod geometry;
pub mod link;
pub mod utils;

#[cfg(feature = "async-trait")]
pub use async_trait::async_trait;

#[cfg(feature = "derive")]
pub mod derive {
    pub use crate::{
        datagram::{
            Datagram, DatagramS, DatagramST, Gain, GainCache, GainFilter, GainTransform,
            IntoGainCache, IntoGainTransform, IntoModulationCache, IntoModulationTransform,
            IntoRadiationPressure, Modulation, ModulationCache, ModulationProperty,
            ModulationTransform, RadiationPressure,
        },
        defined::DEFAULT_TIMEOUT,
        error::AUTDInternalError,
        firmware::{
            fpga::{
                Drive, EmitIntensity, LoopBehavior, Phase, Rad, SamplingConfig, Segment,
                TransitionMode, SAMPLING_FREQ_DIV_MIN,
            },
            operation::{GainOp, ModulationOp, NullOp, Operation},
        },
        geometry::{Device, Geometry, Transducer},
    };
    pub use autd3_derive::{Builder, Gain, Modulation};
    pub use std::collections::HashMap;
}
