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

#[cfg(not(feature = "dynamic_freq"))]
#[inline]
pub const fn get_ultrasound_freq() -> defined::Freq<u32> {
    defined::FREQ_40K
}

#[cfg(all(feature = "dynamic_freq", feature = "test"))]
mod dynamic {
    #[inline]
    pub fn get_ultrasound_freq() -> super::defined::Freq<u32> {
        std::env::var("AUTD3_ULTRASOUND_FREQ")
            .map(|freq| {
                freq.parse::<u32>()
                    .map(|freq| freq * super::defined::Hz)
                    .unwrap_or(super::defined::FREQ_40K)
            })
            .unwrap_or(super::defined::FREQ_40K)
    }
}

#[cfg(all(feature = "dynamic_freq", not(feature = "test")))]
mod dynamic {
    use std::sync::Once;

    use crate::defined::{Hz, FREQ_40K};

    static mut VAL: super::defined::Freq<u32> = FREQ_40K;
    static FREQ: Once = Once::new();

    #[inline]
    pub fn get_ultrasound_freq() -> super::defined::Freq<u32> {
        unsafe {
            FREQ.call_once(|| {
                VAL = match std::env::var("AUTD3_ULTRASOUND_FREQ") {
                    Ok(freq) => match freq.parse::<u32>() {
                        Ok(freq) => {
                            tracing::info!("Set ultrasound frequency to {} Hz.", freq);
                            freq * Hz
                        }
                        Err(_) => {
                            tracing::error!(
                                "Invalid ultrasound frequency ({} Hz), fallback to 40 kHz.",
                                freq
                            );
                            FREQ_40K
                        }
                    },
                    Err(_) => {
                        tracing::warn!("Environment variable AUTD3_ULTRASOUND_FREQ is not set, fallback to 40 kHz.");
                        FREQ_40K
                    }
                };
            });
            VAL
        }
    }
}

#[cfg(feature = "dynamic_freq")]
pub use dynamic::get_ultrasound_freq;

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
            SAMPLING_FREQ_DIV_MIN,
        },
        geometry::{Device, Geometry, Transducer},
    };
    pub use autd3_derive::{Builder, Gain, Modulation};
    pub use itertools::Itertools;
    pub use std::collections::HashMap;
    pub use tracing;
    pub use tynm;
}
