mod error;

pub use error::GainError;

use crate::{
    datagram::DeviceMask,
    environment::Environment,
    firmware::Drive,
    firmware::{Segment, transition_mode::TransitionModeParams},
    geometry::{Device, Geometry, Transducer},
};

#[derive(Debug, Clone)]
/// A mask that represents which Transducers are enabled in a Device.
pub enum DeviceTransducerMask {
    /// All transducers are enabled.
    AllEnabled,
    /// All transducers are disabled.
    AllDisabled,
    /// A filtered mask where each bit represents whether the corresponding transducer is enabled.
    Masked(bit_vec::BitVec<u32>),
}

impl DeviceTransducerMask {
    /// Creates a [`DeviceTransducerMask`] from an iterator.
    pub fn from_fn(dev: &Device, f: impl Fn(&Transducer) -> bool) -> Self {
        Self::Masked(bit_vec::BitVec::from_iter(dev.iter().map(f)))
    }

    /// Returns `true` if the transducers is enabled.
    fn is_enabled(&self, tr: &Transducer) -> bool {
        match self {
            Self::AllEnabled => true,
            Self::AllDisabled => false,
            Self::Masked(bit_vec) => bit_vec[tr.idx()],
        }
    }

    fn is_enabled_device(&self) -> bool {
        match self {
            Self::AllEnabled => true,
            Self::AllDisabled => false,
            Self::Masked(_) => true,
        }
    }

    fn num_enabled_transducers(&self, dev: &Device) -> usize {
        match self {
            Self::AllEnabled => dev.num_transducers(),
            Self::AllDisabled => 0,
            Self::Masked(bit_vec) => bit_vec.count_ones() as _,
        }
    }
}

#[derive(Debug)]
/// A filter that represents which transducers are enabled.
pub enum TransducerMask {
    /// All transducers are enabled.
    AllEnabled,
    /// A filtered mask where each value represents the enabled transducers for the corresponding device.
    Masked(Vec<DeviceTransducerMask>),
}

impl TransducerMask {
    /// Creates a [`TransducerMask`].
    pub fn new<T>(v: T) -> Self
    where
        T: IntoIterator<Item = DeviceTransducerMask>,
    {
        Self::Masked(v.into_iter().collect())
    }

    /// Creates a [`TransducerMask`] from a function that maps each [`Device`] to a [`DeviceTransducerMask`].
    pub fn from_fn(geo: &Geometry, f: impl Fn(&Device) -> DeviceTransducerMask) -> Self {
        Self::Masked(geo.iter().map(f).collect())
    }

    /// Returns `true` if all transducers are enabled.
    pub const fn is_all_enabled(&self) -> bool {
        matches!(self, Self::AllEnabled)
    }

    /// Returns `true` if the [`Device`] is enabled.
    pub fn is_enabled_device(&self, dev: &Device) -> bool {
        match self {
            Self::AllEnabled => true,
            Self::Masked(filter) => filter[dev.idx()].is_enabled_device(),
        }
    }

    /// Returns `true` if the [`Transducer`] is enabled.
    pub fn is_enabled(&self, tr: &Transducer) -> bool {
        match self {
            Self::AllEnabled => true,
            Self::Masked(filter) => filter[tr.dev_idx()].is_enabled(tr),
        }
    }

    /// Returns the number of enabled devices.
    pub fn num_enabled_devices(&self, geometry: &Geometry) -> usize {
        match self {
            Self::AllEnabled => geometry.num_devices(),
            Self::Masked(filter) => geometry
                .iter()
                .filter(|dev| filter[dev.idx()].is_enabled_device())
                .count(),
        }
    }

    /// Returns the number of enabled transducers for the given [`Device`].
    pub fn num_enabled_transducers(&self, dev: &Device) -> usize {
        match self {
            TransducerMask::AllEnabled => dev.num_transducers(),
            TransducerMask::Masked(filter) => filter[dev.idx()].num_enabled_transducers(dev),
        }
    }
}

impl From<&DeviceMask> for TransducerMask {
    fn from(filter: &DeviceMask) -> Self {
        match filter {
            DeviceMask::AllEnabled => Self::AllEnabled,
            DeviceMask::Masked(filter) => Self::Masked(
                filter
                    .iter()
                    .map(|enable| {
                        if *enable {
                            DeviceTransducerMask::AllEnabled
                        } else {
                            DeviceTransducerMask::AllDisabled
                        }
                    })
                    .collect(),
            ),
        }
    }
}

/// A trait to calculate the phase and intensity for [`Gain`].
///
/// [`Gain`]: crate::gain::Gain
pub trait GainCalculator<'a>: Send + Sync {
    /// Calculates the phase and intensity for the transducer.
    #[must_use]
    fn calc(&self, tr: &'a Transducer) -> Drive;
}

impl<'a> GainCalculator<'a> for Box<dyn GainCalculator<'a>> {
    fn calc(&self, tr: &'a Transducer) -> Drive {
        self.as_ref().calc(tr)
    }
}

/// A trait for generating a calculator for the gain operation.
pub trait GainCalculatorGenerator<'a> {
    /// The type of the calculator that actually performs the calculation.
    type Calculator: GainCalculator<'a>;

    /// Generate a calculator for the given device.
    #[must_use]
    fn generate(&mut self, device: &'a Device) -> Self::Calculator;
}

/// Trait for calculating the phase/amplitude of each transducer.
///
/// See also [`Gain`] derive macro.
///
/// [`Gain`]: autd3_derive::Gain
pub trait Gain<'a>: core::fmt::Debug + Sized {
    /// The type of the calculator generator.
    type G: GainCalculatorGenerator<'a>;

    /// Initialize the gain and generate the calculator generator.
    ///
    /// `filter` is a hash map that holds a bit vector representing the indices of the enabled transducers for each device index.
    /// If `filter` is `None`, all transducers are enabled.
    fn init(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &TransducerMask,
    ) -> Result<Self::G, GainError>;
}

#[doc(hidden)]
pub struct GainOperationGenerator<'a, G> {
    pub generator: G,
    pub segment: Segment,
    pub transition_params: TransitionModeParams,
    pub __phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a, C: GainCalculatorGenerator<'a>> GainOperationGenerator<'a, C> {
    pub fn new<G: Gain<'a, G = C>>(
        gain: G,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
        segment: Segment,
        transition_params: TransitionModeParams,
    ) -> Result<Self, GainError> {
        Ok(Self {
            generator: gain.init(geometry, env, &TransducerMask::from(filter))?,
            segment,
            transition_params,
            __phantom: core::marker::PhantomData,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
/// The result of the [`Gain`] inspection.
pub struct GainInspectionResult {
    /// The type name of the gain.
    pub name: String,
    /// The data of the gain.
    pub data: Vec<Drive>,
}
