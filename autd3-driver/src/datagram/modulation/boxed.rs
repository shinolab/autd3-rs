use std::mem::MaybeUninit;

use autd3_derive::Modulation;

use super::{Modulation, ModulationOperationGenerator, ModulationProperty};
use crate::derive::*;

pub trait DModulation {
    fn dyn_calc(&mut self) -> Result<Vec<u8>, AUTDInternalError>;
    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<
        #[cfg(not(feature = "lightweight"))] T: Modulation,
        #[cfg(feature = "lightweight")] T: Modulation + Send + Sync,
    > DModulation for MaybeUninit<T>
{
    fn dyn_calc(&mut self) -> Result<Vec<u8>, AUTDInternalError> {
        let mut tmp: MaybeUninit<T> = MaybeUninit::uninit();
        std::mem::swap(&mut tmp, self);
        let g = unsafe { tmp.assume_init() };
        g.calc()
    }

    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { self.assume_init_ref() }.fmt(f)
    }
}

#[derive(Modulation)]
pub struct BoxedModulation {
    m: Box<dyn DModulation>,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

#[cfg(feature = "lightweight")]
unsafe impl Send for BoxedModulation {}
#[cfg(feature = "lightweight")]
unsafe impl Sync for BoxedModulation {}

impl std::fmt::Debug for BoxedModulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.m.as_ref().dyn_fmt(f)
    }
}

impl Modulation for BoxedModulation {
    fn calc(self) -> Result<Vec<u8>, AUTDInternalError> {
        let Self { mut m, .. } = self;
        m.dyn_calc()
    }
}

pub trait IntoBoxedModulation {
    fn into_boxed(self) -> BoxedModulation;
}

impl<
        #[cfg(not(feature = "lightweight"))] M: Modulation + 'static,
        #[cfg(feature = "lightweight")] M: Modulation + Send + Sync + 'static,
    > IntoBoxedModulation for M
{
    fn into_boxed(self) -> BoxedModulation {
        let config = self.sampling_config();
        let loop_behavior = self.loop_behavior();
        BoxedModulation {
            m: Box::new(MaybeUninit::new(self)),
            config,
            loop_behavior,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::datagram::modulation::tests::TestModulation;

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
