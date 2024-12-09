use autd3_driver::{derive::*, firmware::fpga::Drive};
use derive_new::new;

/// `Gain` that output uniform phase and intensity
#[derive(Gain, Clone, PartialEq, Debug, Builder, new)]
pub struct Uniform {
    #[get]
    #[new(into)]
    /// The drive of all transducers
    drive: Drive,
}

impl GainContext for Uniform {
    fn calc(&self, _: &Transducer) -> Drive {
        self.drive
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
        _filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDInternalError> {
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

        let intensity = EmitIntensity::new(rng.gen());
        let phase = Phase::new(rng.gen());
        let g = Uniform::new((intensity, phase));

        assert_eq!(intensity, g.drive().intensity());
        assert_eq!(phase, g.drive().phase());

        let mut b = g.init(&geometry, None)?;
        geometry.iter().for_each(|dev| {
            let d = b.generate(dev);
            dev.iter().for_each(|tr| {
                let d = d.calc(tr);
                assert_eq!(phase, d.phase());
                assert_eq!(intensity, d.intensity());
            });
        });
        Ok(())
    }
}
