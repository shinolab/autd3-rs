mod error;

use std::collections::HashMap;

pub use error::GainError;

use crate::{
    datagram::DeviceFilter,
    environment::Environment,
    firmware::Drive,
    firmware::{Segment, transition_mode::TransitionModeParams},
    geometry::{Device, Geometry, Transducer},
};

#[derive(Debug)]
/// A filter that represents which transducers are enabled.
pub struct TransducerFilter(Option<HashMap<usize, Option<bit_vec::BitVec<u32>>>>);

impl TransducerFilter {
    #[doc(hidden)]
    pub const fn new(filter: HashMap<usize, Option<bit_vec::BitVec<u32>>>) -> Self {
        Self(Some(filter))
    }

    /// Returns a new `TransducerFilter` that enables all transducers.
    pub const fn all_enabled() -> Self {
        Self(None)
    }

    /// Creates a `DeviceFilter` where the value at each index is `f(&Device)`
    pub fn from_fn<FT: Fn(&Transducer) -> bool>(
        geo: &Geometry,
        f: impl Fn(&Device) -> Option<FT>,
    ) -> Self {
        Self(Some(HashMap::from_iter(geo.iter().filter_map(|dev| {
            f(dev).map(|f| {
                (
                    dev.idx(),
                    Some(bit_vec::BitVec::from_fn(dev.num_transducers(), |idx| {
                        f(&dev[idx])
                    })),
                )
            })
        }))))
    }

    /// Returns `true` i
    pub const fn is_all_enabled(&self) -> bool {
        self.0.is_none()
    }

    /// Returns `true` if the `Device` is enabled.
    pub fn is_enabled_device(&self, dev: &Device) -> bool {
        self.0.as_ref().is_none_or(|f| f.contains_key(&dev.idx()))
    }

    /// Returns `true` if the `Transducer` is enabled.
    pub fn is_enabled(&self, tr: &Transducer) -> bool {
        self.0.as_ref().is_none_or(|f| {
            f.get(&tr.dev_idx())
                .map(|f| f.as_ref().map(|f| f[tr.idx()]).unwrap_or(true))
                .unwrap_or(false)
        })
    }

    /// Returns the number of enabled devices.
    pub fn num_enabled_devices(&self, geometry: &Geometry) -> usize {
        self.0.as_ref().map_or(geometry.num_devices(), |f| {
            geometry
                .iter()
                .filter(|dev| f.contains_key(&dev.idx()))
                .count()
        })
    }

    /// Returns the number of enabled transducers for the given `Device`.
    pub fn num_enabled_transducers(&self, dev: &Device) -> usize {
        self.0.as_ref().map_or(dev.num_transducers(), |f| {
            f.get(&dev.idx()).map_or(0, |filter| {
                filter
                    .as_ref()
                    .map_or(dev.num_transducers(), |f| f.count_ones() as usize)
            })
        })
    }
}

impl From<&DeviceFilter> for TransducerFilter {
    fn from(filter: &DeviceFilter) -> Self {
        if let Some(filter) = filter.0.as_ref() {
            Self(Some(
                filter
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, enable)| enable.then_some((idx, None)))
                    .collect(),
            ))
        } else {
            Self(None)
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
pub trait Gain<'a>: std::fmt::Debug + Sized {
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
        filter: &TransducerFilter,
    ) -> Result<Self::G, GainError>;
}

#[doc(hidden)]
pub struct GainOperationGenerator<'a, G> {
    pub generator: G,
    pub segment: Segment,
    pub transition_params: TransitionModeParams,
    pub __phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, C: GainCalculatorGenerator<'a>> GainOperationGenerator<'a, C> {
    pub fn new<G: Gain<'a, G = C>>(
        gain: G,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        segment: Segment,
        transition_params: TransitionModeParams,
    ) -> Result<Self, GainError> {
        Ok(Self {
            generator: gain.init(geometry, env, &TransducerFilter::from(filter))?,
            segment,
            transition_params,
            __phantom: std::marker::PhantomData,
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
