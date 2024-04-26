use std::ops::Deref;

use super::{sampling_mode::SamplingMode, sine::Sine};

use autd3_driver::derive::*;

use num::integer::lcm;

/// Multi-freq sine wave modulation
#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct Fourier<
    S: SamplingMode<D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration)>,
> {
    #[no_change]
    config: SamplingConfiguration,
    components: Vec<Sine<S>>,
    loop_behavior: LoopBehavior,
}

impl<S: SamplingMode<D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration)>> Fourier<S> {
    pub fn new(sine: Sine<S>) -> Self {
        Self {
            config: sine.sampling_config(),
            components: vec![sine],
            loop_behavior: LoopBehavior::Infinite,
        }
    }

    /// Add a sine wave component
    ///
    /// # Arguments
    /// - `sine` - [Sine] modulation
    ///
    pub fn add_component(self, sine: Sine<S>) -> Self {
        let Self {
            mut components,
            config,
            loop_behavior,
        } = self;
        let config = SamplingConfiguration::from_division_raw(
            config.division().min(sine.sampling_config().division()),
        )
        .unwrap();
        components.push(sine.with_sampling_config(config));
        Self {
            components,
            config,
            loop_behavior,
        }
    }

    /// Add sine wave components from iterator
    ///
    /// # Arguments
    /// - `iter` - Iterator of [Sine] modulation
    ///
    pub fn add_components_from_iter(
        self,
        iter: impl IntoIterator<Item = impl Into<Sine<S>>>,
    ) -> Self {
        let Self {
            mut components,
            config,
            loop_behavior,
        } = self;
        let append = iter.into_iter().map(|m| m.into()).collect::<Vec<_>>();
        let freq_div = append.iter().fold(config.division(), |acc, m| {
            acc.min(m.sampling_config().division())
        });
        let config = SamplingConfiguration::from_division_raw(freq_div).unwrap();
        components.extend(append.into_iter().map(|m| m.with_sampling_config(config)));
        Self {
            components,
            config,
            loop_behavior,
        }
    }
}

impl<S: SamplingMode<D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration)>>
    From<Sine<S>> for Fourier<S>
{
    fn from(sine: Sine<S>) -> Self {
        Self::new(sine)
    }
}

impl<S: SamplingMode<D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration)>> Deref
    for Fourier<S>
{
    type Target = [Sine<S>];

    fn deref(&self) -> &Self::Target {
        &self.components
    }
}

impl<S: SamplingMode<D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration)>>
    std::ops::Add<Sine<S>> for Fourier<S>
{
    type Output = Self;

    fn add(self, rhs: Sine<S>) -> Self::Output {
        self.add_component(rhs)
    }
}

impl<S: SamplingMode<D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration)>>
    std::ops::Add<Sine<S>> for Sine<S>
{
    type Output = Fourier<S>;

    fn add(self, rhs: Sine<S>) -> Self::Output {
        Fourier::from(self).add_component(rhs)
    }
}

impl<S: SamplingMode<D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration)>> Modulation
    for Fourier<S>
{
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let buffers = self
            .components
            .iter()
            .map(|c| c.calc())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(buffers
            .iter()
            .fold(
                vec![0usize; buffers.iter().fold(1, |acc, x| lcm(acc, x.len()))],
                |acc, x| {
                    acc.iter()
                        .zip(x.iter().cycle())
                        .map(|(a, &b)| a + b.value() as usize)
                        .collect::<Vec<_>>()
                },
            )
            .iter()
            .map(|x| ((x / self.components.len()) as u8).into())
            .collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use autd3_driver::defined::PI;

    #[test]
    fn test_fourier() -> anyhow::Result<()> {
        let f0 = Sine::new(50.).with_phase(PI / 2.0 * Rad);
        let f1 = Sine::new(100.).with_phase(PI / 3.0 * Rad);
        let f2 = Sine::new(150.).with_phase(PI / 4.0 * Rad);
        let f3 = Sine::new(200.);
        let f4 = Sine::new(250.);

        let f0_buf = f0.calc()?;
        let f1_buf = f1.calc()?;
        let f2_buf = f2.calc()?;
        let f3_buf = f3.calc()?;
        let f4_buf = f4.calc()?;

        let f = (f0 + f1).add_component(f2).add_components_from_iter([f3]) + f4;

        assert_eq!(f.sampling_config(), SamplingConfiguration::FREQ_4K_HZ);
        assert_eq!(f[0].freq(), 50.);
        assert_eq!(f[0].phase(), PI / 2.0 * Rad);
        assert_eq!(f[1].freq(), 100.);
        assert_eq!(f[1].phase(), PI / 3.0 * Rad);
        assert_eq!(f[2].freq(), 150.);
        assert_eq!(f[2].phase(), PI / 4.0 * Rad);
        assert_eq!(f[3].freq(), 200.);
        assert_eq!(f[3].phase(), 0.0 * Rad);
        assert_eq!(f[4].freq(), 250.);
        assert_eq!(f[4].phase(), 0.0 * Rad);

        let buf = f.calc()?;

        (0..buf.len()).for_each(|i| {
            assert_eq!(
                buf[i].value(),
                ((f0_buf[i % f0_buf.len()].value() as usize
                    + f1_buf[i % f1_buf.len()].value() as usize
                    + f2_buf[i % f2_buf.len()].value() as usize
                    + f3_buf[i % f3_buf.len()].value() as usize
                    + f4_buf[i % f4_buf.len()].value() as usize)
                    / 5) as u8
            );
        });

        Ok(())
    }

    #[test]
    fn test_fourier_derive() {
        let f = Fourier::new(Sine::new(50.).with_phase(PI / 2.0 * Rad));
        assert_eq!(f, f.clone());
    }
}
