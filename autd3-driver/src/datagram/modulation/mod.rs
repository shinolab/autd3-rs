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

use super::DatagramST;

pub type ModulationCalcResult = Result<Vec<u8>, AUTDInternalError>;

pub trait ModulationProperty {
    fn sampling_config(&self) -> SamplingConfig;
    fn loop_behavior(&self) -> LoopBehavior;
}

#[allow(clippy::len_without_is_empty)]
pub trait Modulation: ModulationProperty {
    fn calc(&self, geometry: &Geometry) -> ModulationCalcResult;
}

// GRCOV_EXCL_START
impl ModulationProperty for Box<dyn Modulation> {
    fn sampling_config(&self) -> SamplingConfig {
        self.as_ref().sampling_config()
    }

    fn loop_behavior(&self) -> LoopBehavior {
        self.as_ref().loop_behavior()
    }
}

impl Modulation for Box<dyn Modulation> {
    fn calc(&self, geometry: &Geometry) -> ModulationCalcResult {
        self.as_ref().calc(geometry)
    }
}

pub struct ModulationOperationGenerator {
    #[allow(clippy::type_complexity)]
    pub g: Arc<Vec<u8>>,
    pub config: SamplingConfig,
    pub rep: u32,
    pub segment: Segment,
    pub transition_mode: Option<TransitionMode>,
}

impl OperationGenerator for ModulationOperationGenerator {
    type O1 = ModulationOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        let d = self.g.clone();
        Ok((
            ModulationOp::new(d, self.config, self.rep, self.segment, self.transition_mode),
            NullOp::default(),
        ))
    }
}

impl<'a> DatagramST<'a> for Box<dyn Modulation> {
    type O1 = ModulationOp;
    type O2 = NullOp;
    type G = ModulationOperationGenerator;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        Ok(Self::G {
            g: Arc::new(self.calc(geometry)?),
            config: self.sampling_config(),
            rep: self.loop_behavior().rep,
            segment,
            transition_mode,
        })
    }

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derive::*;

    #[derive(Modulation, Clone, PartialEq, Debug)]
    pub struct TestModulation {
        pub buf: Vec<u8>,
        pub config: SamplingConfig,
        pub loop_behavior: LoopBehavior,
    }

    impl Modulation for TestModulation {
        fn calc(&self, _: &Geometry) -> ModulationCalcResult {
            Ok(self.buf.clone())
        }
    }
}
