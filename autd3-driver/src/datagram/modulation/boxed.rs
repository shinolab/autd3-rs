use std::mem::MaybeUninit;

use autd3_core::derive::*;
use autd3_derive::Modulation;

/// A dyn-compatible version of [`Modulation`].
pub trait DModulation {
    fn dyn_calc(&mut self) -> Result<Vec<u8>, ModulationError>;
    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<
        #[cfg(not(feature = "lightweight"))] T: Modulation,
        #[cfg(feature = "lightweight")] T: Modulation + Send + Sync,
    > DModulation for MaybeUninit<T>
{
    fn dyn_calc(&mut self) -> Result<Vec<u8>, ModulationError> {
        let mut tmp: MaybeUninit<T> = MaybeUninit::uninit();
        std::mem::swap(&mut tmp, self);
        // SAFETY: This function is called only once from `Modulation::calc`.
        let g = unsafe { tmp.assume_init() };
        g.calc()
    }

    fn dyn_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: This function is never called after `dyn_init`.
        unsafe { self.assume_init_ref() }.fmt(f)
    }
}

/// Boxed [`Modulation`].
///
/// This provides the ability to wrap any [`Modulation`] in a common type.
#[derive(Modulation)]
pub struct BoxedModulation {
    m: Box<dyn DModulation>,
    sampling_config: Result<SamplingConfig, ModulationError>,
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
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        let Self { mut m, .. } = self;
        m.dyn_calc()
    }

    fn sampling_config(&self) -> Result<SamplingConfig, ModulationError> {
        self.sampling_config.clone()
    }
}

/// Trait to convert [`Modulation`] to [`BoxedModulation`].
pub trait IntoBoxedModulation {
    /// Convert [`Modulation`] to [`BoxedModulation`]
    fn into_boxed(self) -> BoxedModulation;
}

impl<
        #[cfg(not(feature = "lightweight"))] M: Modulation + 'static,
        #[cfg(feature = "lightweight")] M: Modulation + Send + Sync + 'static,
    > IntoBoxedModulation for M
{
    fn into_boxed(self) -> BoxedModulation {
        let sampling_config = self.sampling_config();
        BoxedModulation {
            m: Box::new(MaybeUninit::new(self)),
            sampling_config,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::datagram::modulation::tests::TestModulation;

    #[test]
    fn boxed_modulation_unsafe() {
        let m = TestModulation {
            sampling_config: SamplingConfig::DIV_10,
        };

        let mb = m.clone().into_boxed();

        assert_eq!(format!("{:?}", m), format!("{:?}", mb));
        assert_eq!(Ok(SamplingConfig::DIV_10), mb.sampling_config());
        assert_eq!(Ok(vec![0; 2]), mb.calc());
    }
}
