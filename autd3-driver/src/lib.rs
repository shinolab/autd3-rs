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

/// Utilities for user-defined `Gain` and `Modulation`.
#[cfg(feature = "derive")]
pub mod derive {
    pub use crate::{
        datagram::{
            Datagram, DatagramS, Gain, GainContextGenerator, GainOperationGenerator, Modulation,
            ModulationOperationGenerator, ModulationProperty,
        },
        defined::DEFAULT_TIMEOUT,
        error::AUTDDriverError,
        firmware::{
            fpga::{
                Drive, EmitIntensity, LoopBehavior, Phase, SamplingConfig, Segment, TransitionMode,
            },
            operation::GainContext,
        },
        geometry::{Device, Geometry, Transducer},
    };
    pub use autd3_derive::{Builder, Gain, Modulation};
    pub use bit_vec::BitVec;
    pub use std::{collections::HashMap, sync::Arc};
    pub use tracing;
}
