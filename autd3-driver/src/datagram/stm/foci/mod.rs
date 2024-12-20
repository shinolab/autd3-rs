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

use autd3_derive::Builder;
use derive_more::{Deref, DerefMut};
use silencer::WithSampling;

#[allow(clippy::len_without_is_empty)]
pub trait FociSTMContextGenerator<const N: usize>: std::fmt::Debug {
    type Context: FociSTMContext<N>;
    fn generate(&mut self, device: &Device) -> Self::Context;
}

#[allow(clippy::len_without_is_empty)]
pub trait FociSTMGenerator<const N: usize>: std::fmt::Debug {
    type T: FociSTMContextGenerator<N>;

    fn init(self) -> Result<Self::T, AUTDDriverError>;
    fn len(&self) -> usize;
}

pub trait IntoFociSTMGenerator<const N: usize> {
    type G: FociSTMGenerator<N>;

    fn into(self) -> Self::G;
}

#[derive(Clone, Builder, Deref, DerefMut, Debug)]
pub struct FociSTM<const N: usize, G: FociSTMGenerator<N>> {
    #[deref]
    #[deref_mut]
    gen: G,
    #[get]
    #[set]
    loop_behavior: LoopBehavior,
    #[get]
    sampling_config: SamplingConfig,
}

impl<const N: usize, G: FociSTMGenerator<N>> WithSampling for FociSTM<N, G> {
    fn sampling_config_intensity(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }

    fn sampling_config_phase(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }
}

impl<const N: usize, G: FociSTMGenerator<N>> FociSTM<N, G> {
    pub fn new(
        config: impl Into<STMConfig>,
        iter: impl IntoFociSTMGenerator<N, G = G>,
    ) -> Result<Self, AUTDDriverError> {
        Self::new_from_sampling_config(config.into(), iter)
    }

    pub fn new_nearest(
        config: impl Into<STMConfigNearest>,
        iter: impl IntoFociSTMGenerator<N, G = G>,
    ) -> Result<Self, AUTDDriverError> {
        Self::new_from_sampling_config(config.into(), iter)
    }

    fn new_from_sampling_config<T>(
        config: T,
        iter: impl IntoFociSTMGenerator<N, G = G>,
    ) -> Result<Self, AUTDDriverError>
    where
        SamplingConfig: TryFrom<(T, usize), Error = AUTDDriverError>,
    {
        let gen = iter.into();
        Ok(Self {
            sampling_config: (config, gen.len()).try_into()?,
            gen,
            loop_behavior: LoopBehavior::infinite(),
        })
    }

    pub fn freq(&self) -> Freq<f32> {
        self.sampling_config().freq() / self.gen.len() as f32
    }

    pub fn period(&self) -> Duration {
        self.sampling_config().period() * self.gen.len() as u32
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
            Self::O2::new(),
        )
    }
}

impl<const N: usize, G: FociSTMGenerator<N>> DatagramS for FociSTM<N, G> {
    type G = FociSTMOperationGenerator<N, G::T>;

    fn operation_generator_with_segment(
        self,
        _geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDDriverError> {
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
