use std::time::Duration;

use super::{Modulation, ModulationOperationGenerator, ModulationProperty};
use crate::{
    defined::DEFAULT_TIMEOUT,
    derive::DatagramS,
    error::AUTDInternalError,
    firmware::fpga::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
    geometry::Geometry,
};

#[cfg(not(feature = "lightweight"))]
type BoxedFmt = Box<dyn Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result>;
#[cfg(feature = "lightweight")]
type BoxedFmt = Box<dyn Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result + Send + Sync>;

pub struct BoxedModulation {
    dbg: BoxedFmt,
    #[cfg(not(feature = "lightweight"))]
    gen: Box<dyn FnOnce() -> Result<Vec<u8>, AUTDInternalError>>,
    #[cfg(feature = "lightweight")]
    gen: Box<dyn FnOnce() -> Result<Vec<u8>, AUTDInternalError> + Send + Sync>,
    sampling_config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl std::fmt::Debug for BoxedModulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.dbg)(f)
    }
}

impl ModulationProperty for BoxedModulation {
    fn sampling_config(&self) -> SamplingConfig {
        self.sampling_config
    }

    fn loop_behavior(&self) -> LoopBehavior {
        self.loop_behavior
    }
}

impl Modulation for BoxedModulation {
    fn calc(self) -> Result<Vec<u8>, AUTDInternalError> {
        (self.gen)()
    }
}

// GRCOV_EXCL_START
impl DatagramS for BoxedModulation {
    type G = ModulationOperationGenerator;

    fn operation_generator_with_segment(
        self,
        _: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        let config = self.sampling_config();
        let loop_behavior = self.loop_behavior();
        let g = self.calc()?;
        tracing::trace!("Modulation buffer: {:?}", g);
        Ok(Self::G {
            g: std::sync::Arc::new(g),
            config,
            loop_behavior,
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
// GRCOV_EXCL_STOP

pub trait IntoBoxedModulation {
    fn into_boxed(self) -> BoxedModulation;
}

#[cfg(not(feature = "lightweight"))]
impl<M: Modulation> IntoBoxedModulation for M
where
    M: 'static,
{
    fn into_boxed(self) -> BoxedModulation {
        let sampling_config = self.sampling_config();
        let loop_behavior = self.loop_behavior();
        let m = std::rc::Rc::new(std::cell::RefCell::new(Some(self)));
        BoxedModulation {
            dbg: Box::new({
                let m = m.clone();
                move |f| m.borrow().as_ref().unwrap().fmt(f)
            }),
            sampling_config,
            loop_behavior,
            gen: Box::new(move || m.take().unwrap().calc()),
        }
    }
}

#[cfg(feature = "lightweight")]
impl<M: Modulation> IntoBoxedModulation for M
where
    M: Send + Sync + 'static,
{
    fn into_boxed(self) -> BoxedModulation {
        let sampling_config = self.sampling_config();
        let loop_behavior = self.loop_behavior();
        let m = std::sync::Arc::new(std::sync::Mutex::new(Some(self)));
        BoxedModulation {
            dbg: Box::new({
                let m = m.clone();
                move |f| m.lock().unwrap().as_ref().unwrap().fmt(f)
            }),
            sampling_config,
            loop_behavior,
            gen: Box::new(move || m.lock().unwrap().take().unwrap().calc()),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{datagram::modulation::tests::TestModulation, derive::*};

    #[test]
    fn test() {
        let m = TestModulation {
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        };

        let mb = m.clone().into_boxed();

        assert_eq!(format!("{:?}", m), format!("{:?}", mb));
        assert_eq!(SamplingConfig::FREQ_4K, mb.sampling_config());
        assert_eq!(LoopBehavior::infinite(), mb.loop_behavior());
        assert_eq!(Ok(vec![0; 2]), mb.calc());
    }
}
