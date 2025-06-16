use crate::geometry::{Device, Geometry};

/// A filter that represents which devices are enabled.
pub struct DeviceFilter(pub(crate) Option<smallvec::SmallVec<[bool; 32]>>);

impl DeviceFilter {
    /// Returns a new `DeviceFilter` that enables all devices.
    pub const fn all_enabled() -> Self {
        Self(None)
    }

    /// Creates a `DeviceFilter` where the value at each index is `f(&Device)`
    pub fn from_fn(geo: &Geometry, f: impl Fn(&Device) -> bool) -> Self {
        Self(Some(geo.iter().map(f).collect()))
    }

    /// Returns `true` if the `Device` enabled.
    pub fn is_enabled(&self, dev: &Device) -> bool {
        self.0.as_ref().is_none_or(|f| f[dev.idx()])
    }

    /// Sets the device at `idx` to enabled.
    pub fn set_enable(&mut self, idx: usize) {
        if let Some(ref mut filter) = self.0 {
            filter[idx] = true;
        }
    }
}
