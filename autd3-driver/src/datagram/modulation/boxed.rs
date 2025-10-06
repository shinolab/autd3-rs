use std::mem::MaybeUninit;

use autd3_core::derive::*;

/// A dyn-compatible version of [`Modulation`].
pub trait DModulation {
    fn dyn_calc(&mut self) -> Result<Vec<u8>, ModulationError>;
}

impl<T: Modulation> DModulation for MaybeUninit<T> {
    fn dyn_calc(&mut self) -> Result<Vec<u8>, ModulationError> {
        let mut tmp: MaybeUninit<T> = MaybeUninit::uninit();
        std::mem::swap(&mut tmp, self);
        // SAFETY: This function is called only once from `Modulation::calc`.
        let g = unsafe { tmp.assume_init() };
        g.calc()
    }
}

/// Boxed [`Modulation`].
///
/// This provides the ability to wrap any [`Modulation`] in a common type.
#[derive(Modulation)]
pub struct BoxedModulation {
    m: Box<dyn DModulation>,
    sampling_config: SamplingConfig,
}

impl BoxedModulation {
    /// Creates a new [`BoxedModulation`].
    pub fn new<M: Modulation + 'static>(m: M) -> BoxedModulation {
        let sampling_config = m.sampling_config();
        BoxedModulation {
            m: Box::new(MaybeUninit::new(m)),
            sampling_config,
        }
    }
}

impl Modulation for BoxedModulation {
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        let Self { mut m, .. } = self;
        m.dyn_calc()
    }

    fn sampling_config(&self) -> SamplingConfig {
        self.sampling_config
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::datagram::modulation::tests::TestModulation;

    #[test]
    fn boxed_modulation_unsafe() {
        let m = TestModulation {
            sampling_config: SamplingConfig::FREQ_4K,
        };

        let mb = BoxedModulation::new(m.clone());

        assert_eq!(SamplingConfig::FREQ_4K, mb.sampling_config());
        assert_eq!(Ok(vec![0; 2]), mb.calc());
    }
}
