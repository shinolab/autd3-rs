mod cache;
mod radiation_pressure;
mod transform;

use std::time::Duration;

pub use cache::Cache as ModulationCache;
pub use cache::IntoCache as IntoModulationCache;
pub use radiation_pressure::IntoRadiationPressure;
pub use radiation_pressure::RadiationPressure;
pub use transform::IntoTransform as IntoModulationTransform;
pub use transform::Transform as ModulationTransform;

use crate::defined::DEFAULT_TIMEOUT;
use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
        operation::{ModulationOp, NullOp},
    },
    geometry::{Device, Geometry},
};

use super::DatagramST;

pub type ModCalcFn<'a> =
    Box<dyn Fn(&Device) -> Box<dyn ExactSizeIterator<Item = u8> + 'a> + Send + Sync + 'a>;

pub trait ModulationProperty {
    fn sampling_config(&self) -> SamplingConfig;
    fn loop_behavior(&self) -> LoopBehavior;
}

#[allow(clippy::len_without_is_empty)]
pub trait Modulation: ModulationProperty {
    fn calc<'a>(&'a self, geometry: &'a Geometry) -> Result<ModCalcFn<'a>, AUTDInternalError>;
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
    fn calc<'a>(&'a self, geometry: &'a Geometry) -> Result<ModCalcFn<'a>, AUTDInternalError> {
        self.as_ref().calc(geometry)
    }
}

impl<'a> DatagramST<'a> for Box<dyn Modulation> {
    type O1 = ModulationOp<Box<dyn ExactSizeIterator<Item = u8> + 'a>>;
    type O2 = NullOp;

    fn operation_with_segment(
        &'a self,
        geometry: &'a Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError> {
        let f = self.calc(geometry)?;
        let sampling_config = self.sampling_config();
        let rep = self.loop_behavior().rep;
        Ok(move |dev| {
            (
                Self::O1::new(f(dev), sampling_config, rep, segment, transition_mode),
                Self::O2::default(),
            )
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
        fn calc<'a>(&'a self, _: &'a Geometry) -> Result<ModCalcFn<'a>, AUTDInternalError> {
            Ok(Box::new(move |_| Box::new(self.buf.iter().copied())))
        }
    }
}
