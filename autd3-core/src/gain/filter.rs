use crate::{
    datagram::DeviceMask,
    geometry::{Device, Geometry, Transducer},
};

#[derive(Debug, Clone, PartialEq)]
/// A mask that represents which [`Transducer`]s are enabled in a [`Device`].
pub enum DeviceTransducerMask {
    /// All transducers are enabled.
    AllEnabled,
    /// All transducers are disabled.
    AllDisabled,
    /// The transducers are enabled/disabled according to the mask.
    Masked(Vec<bool>),
}

impl DeviceTransducerMask {
    /// Creates a [`DeviceTransducerMask`] from an iterator.
    pub fn from_fn(dev: &Device, f: impl Fn(&Transducer) -> bool) -> Self {
        Self::Masked(Vec::from_iter(dev.iter().map(f)))
    }

    fn is_enabled(&self, tr: &Transducer) -> bool {
        match self {
            Self::AllEnabled => true,
            Self::AllDisabled => false,
            Self::Masked(mask) => mask[tr.idx()],
        }
    }

    fn has_enabled(&self) -> bool {
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
            Self::Masked(mask) => mask.iter().filter(|b| **b).count(),
        }
    }
}

#[derive(Debug)]
/// A filter that represents which [`Transducer`]s are enabled.
pub enum TransducerMask {
    /// All transducers are enabled.
    AllEnabled,
    /// The transducers are enabled/disabled according to the [`DeviceTransducerMask`] for each device.
    Masked(Vec<DeviceTransducerMask>),
}

impl TransducerMask {
    /// Creates a new [`TransducerMask`] from an iterator of [`DeviceTransducerMask`]s.
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

    /// Returns `true` if the [`Device`] has enabled transducers.
    pub fn has_enabled(&self, dev: &Device) -> bool {
        match self {
            Self::AllEnabled => true,
            Self::Masked(filter) => filter[dev.idx()].has_enabled(),
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
                .filter(|dev| filter[dev.idx()].has_enabled())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::tests::create_device;

    #[test]
    fn device_transducer_mask_from_fn() {
        let dev = create_device(10);
        let mask = DeviceTransducerMask::from_fn(&dev, |tr| tr.idx() % 2 == 0);
        assert!(
            dev.iter()
                .all(|tr| (tr.idx() % 2 == 0) == mask.is_enabled(tr))
        );
        assert_eq!(5, mask.num_enabled_transducers(&dev));
    }

    #[test]
    fn is_enabled_variants() {
        let dev = create_device(3);
        let mask_all = DeviceTransducerMask::AllEnabled;
        assert!(dev.iter().all(|tr| mask_all.is_enabled(tr)));

        let mask_none = DeviceTransducerMask::AllDisabled;
        assert!(dev.iter().all(|tr| !mask_none.is_enabled(tr)));

        let mask_some = DeviceTransducerMask::from_fn(&dev, |tr| tr.idx() == 1);
        assert!(mask_some.is_enabled(&dev[1]));
        assert!(!mask_some.is_enabled(&dev[0]));
        assert!(!mask_some.is_enabled(&dev[2]));
    }

    #[test]
    fn has_enabled_variants() {
        let dev = create_device(2);
        let mask_all = DeviceTransducerMask::AllEnabled;
        assert!(mask_all.has_enabled());

        let mask_none = DeviceTransducerMask::AllDisabled;
        assert!(!mask_none.has_enabled());

        let mask_masked_all_disabled = DeviceTransducerMask::from_fn(&dev, |_| false);
        assert!(mask_masked_all_disabled.has_enabled());

        let mask_masked_some = DeviceTransducerMask::from_fn(&dev, |tr| tr.idx() == 0);
        assert!(mask_masked_some.has_enabled());
    }

    #[test]
    fn num_enabled_transducers_variants() {
        let dev = create_device(4);

        let mask_all = DeviceTransducerMask::AllEnabled;
        assert_eq!(4, mask_all.num_enabled_transducers(&dev));

        let mask_none = DeviceTransducerMask::AllDisabled;
        assert_eq!(0, mask_none.num_enabled_transducers(&dev));

        let mask_some = DeviceTransducerMask::from_fn(&dev, |tr| tr.idx() < 2);
        assert_eq!(2, mask_some.num_enabled_transducers(&dev));
    }

    #[test]
    fn num_enabled_devices_variants() {
        let geometry = Geometry::new(vec![create_device(1), create_device(1), create_device(1)]);

        let mask_all = TransducerMask::AllEnabled;
        assert_eq!(3, mask_all.num_enabled_devices(&geometry));

        let mask_none = TransducerMask::Masked(vec![
            DeviceTransducerMask::AllDisabled,
            DeviceTransducerMask::AllDisabled,
            DeviceTransducerMask::AllDisabled,
        ]);
        assert_eq!(0, mask_none.num_enabled_devices(&geometry));

        let mask_some = TransducerMask::Masked(vec![
            DeviceTransducerMask::AllEnabled,
            DeviceTransducerMask::AllDisabled,
            DeviceTransducerMask::AllEnabled,
        ]);
        assert_eq!(2, mask_some.num_enabled_devices(&geometry));
    }
}
