use crate::geometry::{Device, Geometry};

/// A filter that represents which devices are enabled.
pub enum DeviceMask {
    /// All devices are enabled.
    AllEnabled,
    /// A filtered mask where each value represents whether the corresponding device is enabled.
    Masked(alloc::vec::Vec<bool>),
}

impl DeviceMask {
    /// Creates a [`DeviceMask`] where the value at each index is `f(&Device)`
    pub fn from_fn(geo: &Geometry, f: impl Fn(&Device) -> bool) -> Self {
        Self::Masked(geo.iter().map(f).collect())
    }

    /// Returns `true` if the `Device` enabled.
    pub fn is_enabled(&self, dev: &Device) -> bool {
        match self {
            Self::AllEnabled => true,
            Self::Masked(filter) => filter[dev.idx()],
        }
    }

    /// Sets the device enabled.
    pub fn set_enable(&mut self, dev: &Device) {
        if let Self::Masked(filter) = self {
            filter[dev.idx()] = true;
        }
    }
}
