mod implement;

use super::sampling_config::*;
use crate::{
    datagram::*,
    defined::Freq,
    firmware::{
        fpga::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
        operation::FociSTMOp,
    },
};

pub use crate::firmware::operation::FociSTMContext;

use autd3_core::datagram::DatagramS;
use autd3_derive::Builder;
use derive_more::{Deref, DerefMut};
use silencer::HasSamplingConfig;

/// A trait to generate the [`FociSTMContext`].
#[allow(clippy::len_without_is_empty)]
pub trait FociSTMContextGenerator<const N: usize>: std::fmt::Debug {
    /// [`FociSTMContext`] that generates the sequence of foci.
    type Context: FociSTMContext<N>;

    /// generates the context.
    fn generate(&mut self, device: &Device) -> Self::Context;
}

/// A trait to generate the [`FociSTMContextGenerator`].
#[allow(clippy::len_without_is_empty)]
pub trait FociSTMGenerator<const N: usize>: std::fmt::Debug {
    /// The type of the context generator.
    type T: FociSTMContextGenerator<N>;

    /// Initializes and returns the context generator.
    fn init(self) -> Result<Self::T, AUTDDriverError>;

    /// Returns the length of the sequence of foci.
    fn len(&self) -> usize;
}

/// A trait to convert to [`FociSTMGenerator`].
pub trait IntoFociSTMGenerator<const N: usize> {
    /// The type of the generator.
    type G: FociSTMGenerator<N>;

    /// Converts to [`FociSTMGenerator`].
    fn into(self) -> Self::G;
}

/// [`Datagram`] to produce STM by foci.
#[derive(Clone, Builder, Deref, DerefMut, Debug)]
pub struct FociSTM<const N: usize, G: FociSTMGenerator<N>> {
    #[deref]
    #[deref_mut]
    gen: G,
    #[get]
    #[set]
    /// The loop behavior of the STM.
    loop_behavior: LoopBehavior,
    #[get]
    /// The sampling configuration of the STM.
    sampling_config: SamplingConfig,
}

impl<const N: usize, G: FociSTMGenerator<N>> FociSTM<N, G> {
    /// Creates a new [`FociSTM`].
    ///
    /// # Errors
    ///
    /// Returns [`AUTDDriverError::SamplingConfig`] or [`AUTDDriverError::STMPeriodInvalid`] if the frequency or period cannot be set strictly.
    pub fn new(
        config: impl Into<STMConfig>,
        iter: impl IntoFociSTMGenerator<N, G = G>,
    ) -> Result<Self, AUTDDriverError> {
        Self::new_from_sampling_config(config.into(), iter)
    }

    /// Creates a new [`FociSTM`] with the nearest frequency or period to the specified value of the possible values.
    pub fn new_nearest(
        config: impl Into<STMConfigNearest>,
        iter: impl IntoFociSTMGenerator<N, G = G>,
    ) -> Self {
        Self::new_from_sampling_config(config.into(), iter).unwrap()
    }

    fn new_from_sampling_config(
        config: impl IntoSamplingConfigSTM,
        iter: impl IntoFociSTMGenerator<N, G = G>,
    ) -> Result<Self, AUTDDriverError> {
        let gen = iter.into();
        Ok(Self {
            sampling_config: config.into_sampling_config(gen.len())?,
            gen,
            loop_behavior: LoopBehavior::infinite(),
        })
    }

    /// Returns the frequency of the STM.
    ///
    /// # Example
    ///
    /// ```
    /// # use autd3_driver::datagram::FociSTM;
    /// # use autd3_driver::defined::Hz;
    /// # use autd3_driver::firmware::fpga::SamplingConfig;
    /// # use autd3_driver::geometry::Point3;
    /// # use autd3_driver::error::AUTDDriverError;
    /// # fn main() -> Result<(), AUTDDriverError> {
    /// let stm = FociSTM::new(1.0 * Hz, vec![Point3::origin(), Point3::origin()])?;
    /// assert_eq!(1.0 * Hz, stm.freq());
    ///
    /// let stm = FociSTM::new(SamplingConfig::new(1.0 * Hz)?, vec![Point3::origin(), Point3::origin()])?;
    /// assert_eq!(0.5 * Hz, stm.freq());
    /// # Ok(())
    /// # }
    /// ```
    pub fn freq(&self) -> Freq<f32> {
        self.sampling_config().freq() / self.gen.len() as f32
    }

    /// Returns the period of the STM.
    ///
    /// # Example
    ///
    /// ```
    /// # use autd3_driver::datagram::FociSTM;
    /// # use autd3_driver::defined::Hz;
    /// # use autd3_driver::firmware::fpga::SamplingConfig;
    /// # use autd3_driver::geometry::Point3;
    /// # use autd3_driver::error::AUTDDriverError;
    /// # fn main() -> Result<(), AUTDDriverError> {
    /// let stm = FociSTM::new(1.0 * Hz, vec![Point3::origin(), Point3::origin()])?;
    /// assert_eq!(std::time::Duration::from_secs(1), stm.period());
    ///
    /// let stm = FociSTM::new(std::time::Duration::from_secs(1), vec![Point3::origin(), Point3::origin()])?;
    /// assert_eq!(std::time::Duration::from_secs(1), stm.period());
    ///
    /// let stm = FociSTM::new(SamplingConfig::new(1.0 * Hz)?, vec![Point3::origin(), Point3::origin()])?;
    /// assert_eq!(std::time::Duration::from_secs(2), stm.period());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(not(feature = "dynamic_freq"))]
    pub fn period(&self) -> std::time::Duration {
        self.sampling_config().period() * self.gen.len() as u32
    }
}

impl<const N: usize, G: FociSTMGenerator<N>> HasSamplingConfig for FociSTM<N, G> {
    fn intensity(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }

    fn phase(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }
}

pub struct FociSTMOperationGenerator<const N: usize, G: FociSTMContextGenerator<N>> {
    gen: G,
    size: usize,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<const N: usize, G: FociSTMContextGenerator<N>> OperationGenerator
    for FociSTMOperationGenerator<N, G>
{
    type O1 = FociSTMOp<N, G::Context>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.gen.generate(device),
                self.size,
                self.config,
                self.loop_behavior,
                self.segment,
                self.transition_mode,
            ),
            Self::O2 {},
        )
    }
}

impl<const N: usize, G: FociSTMGenerator<N>> DatagramS for FociSTM<N, G> {
    type G = FociSTMOperationGenerator<N, G::T>;
    type Error = AUTDDriverError;

    fn operation_generator_with_segment(
        self,
        _geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, Self::Error> {
        let size = self.gen.len();
        Ok(FociSTMOperationGenerator {
            gen: self.gen.init()?,
            size,
            config: self.sampling_config,
            loop_behavior: self.loop_behavior,
            segment,
            transition_mode,
        })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        if self.gen.len() * N >= 4000 {
            None
        } else {
            Some(usize::MAX)
        }
    }
}
