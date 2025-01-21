use autd3_core::derive::*;

use autd3_driver::firmware::fpga::Drive;

/// [`Gain`] that output uniform phase and intensity
#[derive(Gain, Clone, PartialEq, Debug)]
pub struct Uniform {
    /// The intensity of the gain.
    pub intensity: EmitIntensity,
    /// The phase of the gain.
    pub phase: Phase,
}

impl GainContext for Uniform {
    fn calc(&self, _: &Transducer) -> Drive {
        Drive {
            intensity: self.intensity,
            phase: self.phase,
        }
    }
}

impl GainContextGenerator for Uniform {
    type Context = Uniform;

    fn generate(&mut self, _: &Device) -> Self::Context {
        self.clone()
    }
}

impl Gain for Uniform {
    type G = Uniform;

    fn init(
        self,
        _geometry: &Geometry,
        _filter: Option<&HashMap<usize, BitVec>>,
        _option: &DatagramOption,
    ) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::create_geometry;

    use super::*;
    use autd3_driver::firmware::fpga::{EmitIntensity, Phase};
    use rand::Rng;

    #[test]
    fn test_uniform() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);

        let intensity = EmitIntensity(rng.gen());
        let phase = Phase(rng.gen());
        let g = Uniform { intensity, phase };

        let mut b = g.init(&geometry, None, &DatagramOption::default())?;
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                let d = d.calc(tr);
                assert_eq!(phase, d.phase);
                assert_eq!(intensity, d.intensity);
            });
        });
        Ok(())
    }
}
