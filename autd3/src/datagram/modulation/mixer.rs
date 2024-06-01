use super::{sampling_mode::SamplingMode, sine::Sine};

use autd3_driver::derive::*;

use derive_more::Deref;
use num::integer::lcm;

#[derive(Modulation, Clone, PartialEq, Debug, Deref)]
pub struct Mixer<S: SamplingMode> {
    #[no_change]
    config: SamplingConfig,
    #[deref]
    components: Vec<Sine<S>>,
    loop_behavior: LoopBehavior,
}

impl<S: SamplingMode> Mixer<S> {
    pub fn new(componens: impl IntoIterator<Item = Sine<S>>) -> Result<Self, AUTDInternalError> {
        let components = componens.into_iter().collect::<Vec<_>>();
        if components.is_empty() {
            return Err(AUTDInternalError::ModulationError(
                "Components must not be empty".to_string(),
            ));
        }
        let config = components[0].sampling_config();
        if !components.iter().all(|c| c.sampling_config() == config) {
            return Err(AUTDInternalError::ModulationError(
                "All components must have the same sampling configuration".to_string(),
            ));
        }
        Ok(Self {
            config,
            components,
            loop_behavior: LoopBehavior::infinite(),
        })
    }
}

impl<S: SamplingMode> Modulation for Mixer<S> {
    fn calc(&self, geometry: &Geometry) -> ModulationCalcResult {
        let buffers = self
            .components
            .iter()
            .map(|c| c.calc(geometry))
            .map(|v| {
                v.map(|v| {
                    v.into_iter()
                        .map(|x| x as f32 / u8::MAX as f32)
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(buffers
            .iter()
            .fold(
                vec![1.0; buffers.iter().fold(1, |acc, x| lcm(acc, x.len()))],
                |acc, x| {
                    acc.iter()
                        .zip(x.iter().cycle())
                        .map(|(&a, &b)| a * b)
                        .collect::<Vec<_>>()
                },
            )
            .iter()
            .map(|x| (x * u8::MAX as f32) as u8)
            .collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use crate::{modulation::sampling_mode::ExactFreq, tests::create_geometry};

    use super::*;

    use autd3_driver::{defined::Hz, defined::PI};

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(1);

        let f0 = Sine::new(50. * Hz).with_phase(PI / 2.0 * rad);
        let f1 = Sine::new(100. * Hz).with_phase(PI / 3.0 * rad);
        let f2 = Sine::new(150. * Hz).with_phase(PI / 4.0 * rad);
        let f3 = Sine::new(200. * Hz);
        let f4 = Sine::new(250. * Hz);

        let f0_buf = &f0.calc(&geometry)?;
        let f1_buf = &f1.calc(&geometry)?;
        let f2_buf = &f2.calc(&geometry)?;
        let f3_buf = &f3.calc(&geometry)?;
        let f4_buf = &f4.calc(&geometry)?;

        let f = Mixer::new([f0, f1, f2, f3, f4])?;

        assert_eq!(f.sampling_config(), SamplingConfig::Division(5120));
        assert_eq!(f[0].freq(), 50. * Hz);
        assert_eq!(f[0].phase(), PI / 2.0 * rad);
        assert_eq!(f[1].freq(), 100. * Hz);
        assert_eq!(f[1].phase(), PI / 3.0 * rad);
        assert_eq!(f[2].freq(), 150. * Hz);
        assert_eq!(f[2].phase(), PI / 4.0 * rad);
        assert_eq!(f[3].freq(), 200. * Hz);
        assert_eq!(f[3].phase(), 0.0 * rad);
        assert_eq!(f[4].freq(), 250. * Hz);
        assert_eq!(f[4].phase(), 0.0 * rad);

        let buf = &f.calc(&geometry)?;

        (0..buf.len()).for_each(|i| {
            assert_eq!(
                buf[i],
                ((f0_buf[i % f0_buf.len()] as f32 / u8::MAX as f32
                    * f1_buf[i % f1_buf.len()] as f32
                    / u8::MAX as f32
                    * f2_buf[i % f2_buf.len()] as f32
                    / u8::MAX as f32
                    * f3_buf[i % f3_buf.len()] as f32
                    / u8::MAX as f32
                    * f4_buf[i % f4_buf.len()] as f32
                    / u8::MAX as f32)
                    * u8::MAX as f32) as u8
            );
        });

        Ok(())
    }

    #[test]
    fn mismatch_sampling_config() {
        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "All components must have the same sampling configuration".to_string()
            )),
            Mixer::new([
                Sine::new(50. * Hz),
                Sine::new(50. * Hz).with_sampling_config(SamplingConfig::Freq(1000 * Hz)),
            ])
        );
    }

    #[test]
    fn empty_components() {
        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "Components must not be empty".to_string()
            )),
            Mixer::<ExactFreq>::new(vec![])
        );
    }
}
