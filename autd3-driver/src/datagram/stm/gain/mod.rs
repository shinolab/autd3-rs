mod implement;

use std::{collections::HashMap, fmt::Debug, time::Duration};

use super::sampling_config::*;
pub use crate::firmware::operation::GainSTMIterator;
use crate::{
    datagram::*,
    defined::Freq,
    firmware::{
        cpu::GainSTMMode,
        fpga::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
        operation::GainSTMOp,
    },
};

use autd3_core::{
    defined::DEFAULT_TIMEOUT,
    derive::{DatagramL, DatagramOption},
    gain::{BitVec, GainCalculatorGenerator, GainError},
};
use derive_more::{Deref, DerefMut};
use derive_new::new;

/// A trait to generate the [`GainSTMIterator`].
pub trait GainSTMIteratorGenerator {
    /// The element type of the gain sequence.
    type Gain: GainCalculatorGenerator;
    /// [`GainSTMIterator`] that generates the sequence of [`Gain`].
    ///
    /// [`Gain`]: autd3_core::gain::Gain
    type Iterator: GainSTMIterator<Calculator = <Self::Gain as GainCalculatorGenerator>::Calculator>;

    /// generates the iterator.
    fn generate(&mut self, device: &Device) -> Self::Iterator;
}

/// A trait to generate the [`GainSTMIteratorGenerator`].
#[allow(clippy::len_without_is_empty)]
pub trait GainSTMGenerator: std::fmt::Debug {
    /// The type of the iterator generator.
    type T: GainSTMIteratorGenerator;

    /// Initializes and returns the iterator generator.
    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
        parallel: bool,
    ) -> Result<Self::T, GainError>;
    /// Returns the length of the sequence of gains.
    fn len(&self) -> usize;
}

/// The option for the [`GainSTM`].
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct GainSTMOption {
    /// The mode of the STM. The default is [`GainSTMMode::PhaseIntensityFull`].
    pub mode: GainSTMMode,
}

impl Default for GainSTMOption {
    fn default() -> Self {
        Self {
            mode: GainSTMMode::PhaseIntensityFull,
        }
    }
}

/// [`Datagram`] to produce STM by [`Gain`].
///
/// [`Gain`]: autd3_core::gain::Gain
#[derive(Clone, Debug, Deref, DerefMut, new)]
pub struct GainSTM<T: GainSTMGenerator, C> {
    #[deref]
    #[deref_mut]
    /// The sequence of [`Gain`]s.
    ///
    /// [`Gain`]: autd3_core::gain::Gain
    pub gains: T,
    /// The STM configuration.
    pub config: C,
    /// The STM option.
    pub option: GainSTMOption,
}

impl<T: GainSTMGenerator> GainSTM<T, Freq<f32>> {
    /// Convert to STM with the closest frequency among the possible frequencies.
    pub fn into_nearest(self) -> GainSTM<T, FreqNearest> {
        GainSTM {
            gains: self.gains,
            config: FreqNearest(self.config),
            option: self.option,
        }
    }
}

impl<T: GainSTMGenerator> GainSTM<T, Duration> {
    /// Convert to STM with the closest frequency among the possible period.
    pub fn into_nearest(self) -> GainSTM<T, PeriodNearest> {
        GainSTM {
            gains: self.gains,
            config: PeriodNearest(self.config),
            option: self.option,
        }
    }
}

impl<T: GainSTMGenerator, C: Into<STMConfig> + Copy> GainSTM<T, C> {
    /// The sampling configuration of the STM.
    pub fn sampling_config(&self) -> Result<SamplingConfig, AUTDDriverError> {
        let size = self.gains.len();
        let stm_config: STMConfig = self.config.into();
        stm_config.into_sampling_config(size)
    }
}

pub struct GainSTMOperationGenerator<T: GainSTMIteratorGenerator> {
    g: T,
    size: usize,
    mode: GainSTMMode,
    sampling_config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<T: GainSTMIteratorGenerator> OperationGenerator for GainSTMOperationGenerator<T> {
    type O1 = GainSTMOp<<T::Gain as GainCalculatorGenerator>::Calculator, T::Iterator>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.g.generate(device),
                self.size,
                self.mode,
                self.sampling_config,
                self.loop_behavior,
                self.segment,
                self.transition_mode,
            ),
            Self::O2 {},
        )
    }
}

impl<T: GainSTMGenerator, C: Into<STMConfig> + Debug> DatagramL for GainSTM<T, C> {
    type G = GainSTMOperationGenerator<T::T>;
    type Error = AUTDDriverError;

    fn operation_generator_with_loop_behavior(
        self,
        geometry: &Geometry,
        parallel: bool,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
        loop_behavior: LoopBehavior,
    ) -> Result<Self::G, Self::Error> {
        let size = self.gains.len();
        let stm_config: STMConfig = self.config.into();
        let sampling_config = stm_config.into_sampling_config(size)?;
        let GainSTMOption { mode } = self.option;
        let gains = self.gains;
        Ok(GainSTMOperationGenerator {
            g: gains.init(geometry, None, parallel)?,
            size,
            sampling_config,
            mode,
            loop_behavior,
            segment,
            transition_mode,
        })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption {
            parallel_threshold: 4,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}
