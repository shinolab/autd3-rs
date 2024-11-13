mod implement;

use super::sampling_config::*;
use crate::{
    datagram::*,
    defined::Freq,
    derive::*,
    firmware::{cpu::GainSTMMode, operation::GainSTMOp},
};

pub use crate::firmware::operation::GainSTMContext;

use derive_more::{Deref, DerefMut};
use silencer::WithSampling;

pub trait GainSTMContextGenerator {
    type Gain: GainContextGenerator;
    type Context: GainSTMContext<Context = <Self::Gain as GainContextGenerator>::Context>;

    fn generate(&mut self, device: &Device) -> Self::Context;
}

#[allow(clippy::len_without_is_empty)]
pub trait GainSTMGenerator: std::fmt::Debug {
    type T: GainSTMContextGenerator;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::T, AUTDInternalError>;
    fn len(&self) -> usize;
}
pub trait IntoGainSTMGenerator {
    type G: GainSTMGenerator;

    fn into(self) -> Self::G;
}

#[derive(Builder, Clone, Debug, Deref, DerefMut)]
pub struct GainSTM<G: GainSTMGenerator> {
    #[deref]
    #[deref_mut]
    gen: G,
    #[get]
    #[set]
    loop_behavior: LoopBehavior,
    #[get]
    sampling_config: SamplingConfig,
    #[get]
    #[set]
    mode: GainSTMMode,
}

impl<G: GainSTMGenerator> WithSampling for GainSTM<G> {
    fn sampling_config_intensity(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }

    fn sampling_config_phase(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }
}

impl<G: GainSTMGenerator> GainSTM<G> {
    pub fn new<T: IntoGainSTMGenerator<G = G>>(
        config: impl Into<STMConfig>,
        iter: T,
    ) -> Result<Self, AUTDInternalError> {
        Self::new_from_sampling_config(config.into(), iter)
    }

    pub fn new_nearest<T: IntoGainSTMGenerator<G = G>>(
        config: impl Into<STMConfigNearest>,
        iter: T,
    ) -> Result<Self, AUTDInternalError> {
        Self::new_from_sampling_config(config.into(), iter)
    }

    fn new_from_sampling_config<S, T: IntoGainSTMGenerator<G = G>>(
        config: S,
        iter: T,
    ) -> Result<Self, AUTDInternalError>
    where
        SamplingConfig: TryFrom<(S, usize), Error = AUTDInternalError>,
    {
        let gen = iter.into();
        Ok(Self {
            sampling_config: (config, gen.len()).try_into()?,
            loop_behavior: LoopBehavior::infinite(),
            mode: GainSTMMode::PhaseIntensityFull,
            gen,
        })
    }

    pub fn freq(&self) -> Freq<f32> {
        self.sampling_config().freq() / self.gen.len() as f32
    }

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
    ) -> Result<Self::G, AUTDInternalError> {
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
