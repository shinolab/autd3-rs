use super::Point3;

/// A ultrasound transducer.
#[derive(Clone, Debug, PartialEq)]
pub struct Transducer {
    pub(crate) idx: u8,
    pub(crate) dev_idx: u16,
    position: Point3,
}

impl Transducer {
    /// Creates a new [`Transducer`].
    #[must_use]
    pub const fn new(position: Point3) -> Self {
        Self {
            idx: 0,
            dev_idx: 0,
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

    /// Gets the position of the transducer.
    #[must_use]
    pub const fn position(&self) -> Point3 {
        self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idx() {
        let tr = Transducer::new(Point3::origin());
        assert_eq!(0, tr.idx());
        assert_eq!(0, tr.dev_idx());
    }
}
