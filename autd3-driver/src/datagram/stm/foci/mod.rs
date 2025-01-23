mod implement;

use std::{fmt::Debug, time::Duration};

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

use autd3_core::{
    defined::DEFAULT_TIMEOUT,
    derive::{DatagramL, DatagramOption},
};
use derive_more::{Deref, DerefMut};

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

/// [`Datagram`] to produce STM by foci.
#[derive(Clone, Deref, DerefMut, Debug, PartialEq)]
pub struct FociSTM<const N: usize, T: FociSTMGenerator<N>, C> {
    #[deref]
    #[deref_mut]
    /// The sequence of foci.
    pub foci: T,
    /// The STM configuration.
    pub config: C,
}

impl<const N: usize, T: FociSTMGenerator<N>> FociSTM<N, T, Freq<f32>> {
    /// Convert to STM with the closest frequency among the possible frequencies.
    pub fn into_nearest(self) -> FociSTM<N, T, FreqNearest> {
        FociSTM {
            foci: self.foci,
            config: FreqNearest(self.config),
        }
    }
}

impl<const N: usize, T: FociSTMGenerator<N>> FociSTM<N, T, Duration> {
    /// Convert to STM with the closest frequency among the possible period.
    pub fn into_nearest(self) -> FociSTM<N, T, PeriodNearest> {
        FociSTM {
            foci: self.foci,
            config: PeriodNearest(self.config),
        }
    }
}

impl<const N: usize, T: FociSTMGenerator<N>, C: Into<STMConfig> + Copy> FociSTM<N, T, C> {
    /// The sampling configuration of the STM.
    pub fn sampling_config(&self) -> Result<SamplingConfig, AUTDDriverError> {
        let size = self.foci.len();
        let stm_config: STMConfig = self.config.into();
        stm_config.into_sampling_config(size)
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

impl<const N: usize, G: FociSTMGenerator<N>, C: Into<STMConfig> + Debug> DatagramL
    for FociSTM<N, G, C>
{
    type G = FociSTMOperationGenerator<N, G::T>;
    type Error = AUTDDriverError;

    fn operation_generator_with_loop_behavior(
        self,
        _: &Geometry,
        _: bool,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
        loop_behavior: LoopBehavior,
    ) -> Result<Self::G, Self::Error> {
        let size = self.foci.len();
        let stm_config: STMConfig = self.config.into();
        let sampling_config = stm_config.into_sampling_config(size)?;
        Ok(FociSTMOperationGenerator {
            gen: self.foci.init()?,
            size,
            config: sampling_config,
            loop_behavior,
            segment,
            transition_mode,
        })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption {
            parallel_threshold: if self.foci.len() * N >= 4000 {
                4
            } else {
                usize::MAX
            },
            timeout: DEFAULT_TIMEOUT,
        }
    }
}
