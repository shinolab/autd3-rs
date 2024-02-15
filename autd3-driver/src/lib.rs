#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod acoustics;
pub mod autd3_device;
pub mod common;
pub mod cpu;
pub mod datagram;
pub mod defined;
pub mod error;
pub mod firmware_version;
pub mod fpga;
pub mod geometry;
pub mod link;
pub mod operation;
pub mod osal_timer;
pub mod sync_mode;
pub mod timer_strategy;

#[cfg(feature = "async-trait")]
pub use async_trait::async_trait;

#[cfg(feature = "derive")]
pub mod derive {
    pub use crate::{
        common::{Drive, EmitIntensity, LoopBehavior, Phase, Rad, SamplingConfiguration},
        cpu::Segment,
        datagram::{
            DatagramS, Gain, GainCache, GainFilter, GainTransform, IntoGainCache,
            IntoGainTransform, IntoModulationCache, IntoModulationTransform, IntoRadiationPressure,
            Modulation, ModulationCache, ModulationProperty, ModulationTransform,
            RadiationPressure,
        },
        defined::float,
        error::AUTDInternalError,
        fpga::{FPGA_CLK_FREQ, SAMPLING_FREQ_DIV_MIN},
        geometry::{Device, Geometry, Transducer},
        operation::{GainOp, ModulationOp, NullOp, Operation},
    };
    pub use autd3_derive::{Gain, Modulation};
}
