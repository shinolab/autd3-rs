#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod acoustics;
pub mod autd3_device;
pub mod datagram;
pub mod defined;
pub mod error;
pub mod ethercat;
pub mod firmware;
pub mod geometry;
pub mod link;

#[cfg(feature = "async-trait")]
pub use async_trait::async_trait;

#[cfg(feature = "derive")]
pub mod derive {
    pub use crate::{
        datagram::{
            DatagramS, Gain, GainCache, GainFilter, GainTransform, IntoGainCache,
            IntoGainTransform, IntoModulationCache, IntoModulationTransform, IntoRadiationPressure,
            Modulation, ModulationCache, ModulationProperty, ModulationTransform,
            RadiationPressure,
        },
        error::AUTDInternalError,
        firmware::{
            fpga::{
                Drive, EmitIntensity, LoopBehavior, Phase, Rad, SamplingConfiguration, Segment,
                TransitionMode, FPGA_CLK_FREQ, SAMPLING_FREQ_DIV_MIN,
            },
            operation::{GainOp, ModulationOp, NullOp, Operation},
        },
        geometry::{Device, Geometry, Transducer},
    };
    pub use autd3_derive::{Builder, Gain, Modulation};
}
