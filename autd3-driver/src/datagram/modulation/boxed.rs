use autd3_derive::Modulation;

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

#[derive(Modulation)]
pub struct BoxedModulation {
    dbg: BoxedFmt,
    #[cfg(not(feature = "lightweight"))]
    gen: Box<dyn FnOnce() -> Result<Vec<u8>, AUTDInternalError>>,
    #[cfg(feature = "lightweight")]
    gen: Box<dyn FnOnce() -> Result<Vec<u8>, AUTDInternalError> + Send + Sync>,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl std::fmt::Debug for BoxedModulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.dbg)(f)
    }
}

impl Modulation for BoxedModulation {
    fn calc(self) -> Result<Vec<u8>, AUTDInternalError> {
        (self.gen)()
    }
}

pub trait IntoBoxedModulation {
    fn into_boxed(self) -> BoxedModulation;
}

#[cfg(not(feature = "lightweight"))]
impl<M: Modulation> IntoBoxedModulation for M
where
    M: 'static,
{
    fn into_boxed(self) -> BoxedModulation {
        let config = self.sampling_config();
        let loop_behavior = self.loop_behavior();
        let m = std::rc::Rc::new(std::cell::RefCell::new(Some(self)));
        BoxedModulation {
            dbg: Box::new({
                let m = m.clone();
                move |f| m.borrow().as_ref().unwrap().fmt(f)
            }),
            config,
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
        let config = self.sampling_config();
        let loop_behavior = self.loop_behavior();
        let m = std::sync::Arc::new(std::sync::Mutex::new(Some(self)));
        BoxedModulation {
            dbg: Box::new({
                let m = m.clone();
                move |f| m.lock().unwrap().as_ref().unwrap().fmt(f)
            }),
            config,
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
