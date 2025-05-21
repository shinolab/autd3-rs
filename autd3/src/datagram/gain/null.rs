use autd3_core::derive::*;
use autd3_driver::firmware::fpga::Drive;

/// [`Gain`] that output nothing
#[derive(Gain, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Null;

impl Null {
    /// Create a new [`Null`]
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl GainCalculator for Null {
    fn calc(&self, _: &Transducer) -> Drive {
        Drive::NULL
    }
}

impl GainCalculatorGenerator for Null {
    type Calculator = Null;

    fn generate(&mut self, _: &Device) -> Self::Calculator {
        Null {}
    }
}

impl Gain for Null {
    type G = Null;

    fn init(self, _: &Geometry, _: Option<&HashMap<usize, BitVec>>) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::create_geometry;

    use super::*;

    #[test]
    fn test_null() {
        let geometry = create_geometry(1);

        let g = Null::new();
        let mut b = g;
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                assert_eq!(Drive::NULL, d.calc(tr));
            });
        });
    }
}
