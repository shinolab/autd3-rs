use alloc::vec::Vec;

use crate::{
    datagram::DeviceMask,
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
            Self::AllEnabled => true, // GRCOV_EXCL_LINE
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
