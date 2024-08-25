mod cache;
mod radiation_pressure;
mod transform;

use std::sync::Arc;
use std::time::Duration;

pub use cache::Cache as ModulationCache;
pub use cache::IntoCache as IntoModulationCache;
pub use radiation_pressure::IntoRadiationPressure;
pub use radiation_pressure::RadiationPressure;
pub use transform::IntoTransform as IntoModulationTransform;
pub use transform::Transform as ModulationTransform;

use crate::defined::DEFAULT_TIMEOUT;
use crate::firmware::operation::OperationGenerator;
use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
        operation::{ModulationOp, NullOp},
    },
    geometry::{Device, Geometry},
};

use super::silencer::WithSampling;
use super::DatagramST;

pub trait ModulationProperty {
    fn sampling_config(&self) -> SamplingConfig;
    fn loop_behavior(&self) -> LoopBehavior;
}

pub trait Modulation: ModulationProperty + std::fmt::Debug {
    fn calc(&self) -> Result<Arc<Vec<u8>>, AUTDInternalError>;
}

impl<M: Modulation> WithSampling for M {
    fn sampling_config_intensity(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config())
    }
    fn sampling_config_phase(&self) -> Option<SamplingConfig> {
        None
    }
}

pub struct ModulationOperationGenerator {
    pub g: Arc<Vec<u8>>,
    pub config: SamplingConfig,
    pub loop_behavior: LoopBehavior,
    pub segment: Segment,
    pub transition_mode: Option<TransitionMode>,
}

impl OperationGenerator for ModulationOperationGenerator {
    type O1 = ModulationOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        let d = self.g.clone();
        (
            ModulationOp::new(
                d,
                self.config,
                self.loop_behavior,
                self.segment,
                self.transition_mode,
            ),
            NullOp::default(),
        )
    }
}

// GRCOV_EXCL_START
impl<'a> ModulationProperty for Box<dyn Modulation + Send + Sync + 'a> {
    fn sampling_config(&self) -> SamplingConfig {
        self.as_ref().sampling_config()
    }

    fn loop_behavior(&self) -> LoopBehavior {
        self.as_ref().loop_behavior()
    }
}

impl<'a> Modulation for Box<dyn Modulation + Send + Sync + 'a> {
    fn calc(&self) -> Result<Arc<Vec<u8>>, AUTDInternalError> {
        self.as_ref().calc()
    }
}

impl<'a> DatagramST for Box<dyn Modulation + Send + Sync + 'a> {
    type G = ModulationOperationGenerator;

    fn operation_generator_with_segment(
        self,
        _: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        Ok(Self::G {
            g: self.calc()?,
            config: self.sampling_config(),
            loop_behavior: self.loop_behavior(),
            segment,
            transition_mode,
        })
    }

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}

#[cfg(feature = "capi")]
mod capi {
    use crate::derive::*;
    use std::sync::Arc;

    #[derive(Modulation, Debug)]
    struct NullModulation {
        config: SamplingConfig,
        loop_behavior: LoopBehavior,
    }

    impl Modulation for NullModulation {
        fn calc(&self) -> Result<Arc<Vec<u8>>, AUTDInternalError> {
            Ok(Arc::new(vec![]))
        }
    }

    impl<'a> Default for Box<dyn Modulation + Send + Sync + 'a> {
        fn default() -> Self {
            Box::new(NullModulation {
                config: SamplingConfig::FREQ_4K,
                loop_behavior: LoopBehavior::infinite(),
            })
        }
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::derive::*;

    #[derive(Modulation, Clone, PartialEq, Debug)]
    pub struct TestModulation {
        pub buf: Arc<Vec<u8>>,
        pub config: SamplingConfig,
        pub loop_behavior: LoopBehavior,
    }

    impl Modulation for TestModulation {
        fn calc(&self) -> Result<Arc<Vec<u8>>, AUTDInternalError> {
            Ok(self.buf.clone())
        }
    }
}
