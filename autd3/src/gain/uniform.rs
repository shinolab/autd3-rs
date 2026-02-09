use autd3_core::derive::*;

/// [`Gain`] that output uniform phase and intensity
#[derive(Gain, Clone, Copy, PartialEq, Debug)]
pub struct Uniform {
    /// The intensity of all transducers.
    pub intensity: Intensity,
    /// The phase of all transducers.
    pub phase: Phase,
}

impl Uniform {
    /// Create a new [`Uniform`]
    #[must_use]
    pub const fn new(intensity: Intensity, phase: Phase) -> Self {
        Self { intensity, phase }
    }
}

impl GainCalculator<'_> for Uniform {
    fn calc(&self, _: &Transducer) -> Drive {
        Drive {
            intensity: self.intensity,
            phase: self.phase,
        }
    }
}

impl GainCalculatorGenerator<'_> for Uniform {
    type Calculator = Uniform;

    fn generate(&mut self, _: &Device) -> Self::Calculator {
        *self
    }
}

impl Gain<'_> for Uniform {
    type G = Uniform;

    fn init(self, _: &Geometry, _: &Environment, _: &TransducerMask) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::create_geometry;

    use super::*;
    use rand::RngExt;

    #[test]
    fn uniform() {
        let mut rng = rand::rng();

        let geometry = create_geometry(1);

        let intensity = Intensity(rng.random());
        let phase = Phase(rng.random());
        let g = Uniform::new(intensity, phase);

        let mut b = g;
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                let d = d.calc(tr);
                assert_eq!(phase, d.phase);
                assert_eq!(intensity, d.intensity);
            });
        });
    }
}
