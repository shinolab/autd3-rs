mod implement;

use std::collections::HashMap;

use super::sampling_config::*;
pub use crate::firmware::operation::GainSTMContext;
use crate::{
    datagram::*,
    defined::Freq,
    firmware::{
        cpu::GainSTMMode,
        fpga::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
        operation::GainSTMOp,
    },
};

use autd3_derive::Builder;
use bit_vec::BitVec;
use derive_more::{Deref, DerefMut};
use silencer::HasSamplingConfig;

/// A trait to generate the [`GainSTMContext`].
pub trait GainSTMContextGenerator {
    /// The element type of the gain sequence.
    type Gain: GainContextGenerator;
    /// [`GainSTMContext`] that generates the sequence of [`Gain`].
    type Context: GainSTMContext<Context = <Self::Gain as GainContextGenerator>::Context>;

    /// generates the context.
    fn generate(&mut self, device: &Device) -> Self::Context;
}

/// A trait to generate the [`GainSTMContextGenerator`].
#[allow(clippy::len_without_is_empty)]
pub trait GainSTMGenerator: std::fmt::Debug {
    /// The type of the context generator.
    type T: GainSTMContextGenerator;

    /// Initializes and returns the context generator.
    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::T, AUTDDriverError>;
    /// Returns the length of the sequence of gains.
    fn len(&self) -> usize;
}

/// A trait to convert to [`GainSTMGenerator`].
pub trait IntoGainSTMGenerator {
    /// The type of the generator.
    type G: GainSTMGenerator;

    /// Converts to [`GainSTMGenerator`].
    fn into(self) -> Self::G;
}

/// [`Datagram`] to produce STM by [`Gain`].
#[derive(Builder, Clone, Debug, Deref, DerefMut)]
pub struct GainSTM<G: GainSTMGenerator> {
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
    #[get]
    #[set]
    /// The mode of the STM.
    mode: GainSTMMode,
}

impl<G: GainSTMGenerator> HasSamplingConfig for GainSTM<G> {
    fn intensity(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }

    fn phase(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }
}

impl<G: GainSTMGenerator> GainSTM<G> {
    /// Creates a new [`GainSTM`].
    ///
    /// # Errors
    ///
    /// Returns [`AUTDDriverError::SamplingFreqOutOfRangeF`], [`AUTDDriverError::SamplingFreqInvalidF`], or [`AUTDDriverError::STMPeriodInvalid`] if the frequency or period cannot be set strictly.
    pub fn new<T: IntoGainSTMGenerator<G = G>>(
        config: impl Into<STMConfig>,
        iter: T,
    ) -> Result<Self, AUTDDriverError> {
        Self::new_from_sampling_config(config.into(), iter)
    }

    /// Creates a new [`GainSTM`] with the nearest frequency or period to the specified value.
    pub fn new_nearest<T: IntoGainSTMGenerator<G = G>>(
        config: impl Into<STMConfigNearest>,
        iter: T,
    ) -> Self {
        Self::new_from_sampling_config(config.into(), iter).unwrap()
    }

    fn new_from_sampling_config<S, T: IntoGainSTMGenerator<G = G>>(
        config: S,
        iter: T,
    ) -> Result<Self, AUTDDriverError>
    where
        SamplingConfig: TryFrom<(S, usize), Error = AUTDDriverError>,
    {
        let gen = iter.into();
        Ok(Self {
            sampling_config: (config, gen.len()).try_into()?,
            loop_behavior: LoopBehavior::infinite(),
            mode: GainSTMMode::PhaseIntensityFull,
            gen,
        })
    }

    /// Returns the frequency of the STM. See also [`FociSTM::freq`].
    ///
    /// [`FociSTM::freq`]: crate::datagram::FociSTM::freq
    pub fn freq(&self) -> Freq<f32> {
        self.sampling_config().freq() / self.gen.len() as f32
    }

    /// Returns the period of the STM. See also [`FociSTM::period`].
    ///
    /// [`FociSTM::period`]: crate::datagram::FociSTM::period
    pub fn period(&self) -> Duration {
        self.sampling_config().period() * self.gen.len() as u32
    }
}

pub struct GainSTMOperationGenerator<T: GainSTMContextGenerator> {
    g: T,
    size: usize,
    mode: GainSTMMode,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<T: GainSTMContextGenerator> OperationGenerator for GainSTMOperationGenerator<T> {
    type O1 = GainSTMOp<<T::Gain as GainContextGenerator>::Context, T::Context>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.g.generate(device),
                self.size,
                self.mode,
                self.config,
                self.loop_behavior,
                self.segment,
                self.transition_mode,
            ),
            Self::O2::new(),
        )
    }
}

impl<I: GainSTMGenerator> DatagramS for GainSTM<I> {
    type G = GainSTMOperationGenerator<I::T>;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDDriverError> {
        let size = self.gen.len();
        let config = self.sampling_config;
        let loop_behavior = self.loop_behavior;
        let mode = self.mode;
        let initializer = self.gen;
        Ok(GainSTMOperationGenerator {
            g: initializer.init(geometry, None)?,
            size,
            config,
            mode,
            loop_behavior,
            segment,
            transition_mode,
        })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        None
    }
}
