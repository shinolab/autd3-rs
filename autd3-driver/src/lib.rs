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

#[cfg(all(feature = "dynamic_freq", not(any(feature = "test", feature = "capi"))))]
mod dynamic {
    use std::sync::Once;

    use crate::defined::FREQ_40K;

    static mut VAL: super::defined::Freq<u32> = FREQ_40K;
    static FREQ: Once = Once::new();

    #[inline]
    pub fn set_ultrasound_freq(freq: super::defined::Freq<u32>) {
        unsafe {
            FREQ.call_once(|| {
                VAL = freq;
            });
        }
    }

    #[inline]
    pub fn get_ultrasound_freq() -> super::defined::Freq<u32> {
        unsafe {
            #[cfg(not(feature = "capi"))]
            if !FREQ.is_completed() {
                panic!("Set ultrasound frequency first by `set_ultrasound_freq`");
            }
            VAL
        }
    }
}

#[cfg(all(feature = "dynamic_freq", any(feature = "test", feature = "capi")))]
mod dynamic {
    use std::{cell::Cell, sync::Once};

    use crate::defined::FREQ_40K;

    thread_local! {
        static LOCAL_VAL: Cell<super::defined::Freq<u32>> = Cell::new(FREQ_40K);
        static FREQ: Once = Once::new();
    }

    #[inline]
    pub fn set_ultrasound_freq(freq: super::defined::Freq<u32>) {
        FREQ.with(|f| {
            f.call_once(|| {
                LOCAL_VAL.set(freq);
            })
        });
    }

    #[inline]
    pub fn get_ultrasound_freq() -> super::defined::Freq<u32> {
        LOCAL_VAL.get()
    }
}

#[cfg(feature = "dynamic_freq")]
pub use dynamic::{get_ultrasound_freq, set_ultrasound_freq};

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
