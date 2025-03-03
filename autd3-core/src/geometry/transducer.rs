use getset::Getters;

use super::{Isometry, Point3};

/// A ultrasound transducer.
#[derive(Clone, Debug, PartialEq, Getters)]
pub struct Transducer {
    pub(crate) idx: u8,
    pub(crate) dev_idx: u16,
    #[getset(get = "pub")]
    /// The position of the transducer.
    position: Point3,
}

impl Transducer {
    /// Creates a new [`Transducer`].
    #[must_use]
    pub const fn new(idx: u8, dev_idx: u16, position: Point3) -> Self {
        Self {
            idx,
            dev_idx,
            position,
        }
    }

    /// Gets the local index of the transducer.
    #[must_use]
    pub const fn idx(&self) -> usize {
        self.idx as _
    }

    /// Gets the index of the device to which this transducer belongs.
    #[must_use]
    pub const fn dev_idx(&self) -> usize {
        self.dev_idx as _
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idx() {
        let tr = Transducer::new(1, 2, Point3::origin());
        assert_eq!(1, tr.idx());
        assert_eq!(2, tr.dev_idx());
    }
}
