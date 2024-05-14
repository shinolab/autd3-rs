use std::ops::Deref;

use super::{sampling_mode::SamplingMode, sine::Sine};

use autd3_driver::derive::*;

use num::integer::lcm;

/// Multi-freq sine wave modulation
#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct Fourier<S: SamplingMode> {
    #[no_change]
    config: SamplingConfig,
    components: Vec<Sine<S>>,
    loop_behavior: LoopBehavior,
}

impl<S: SamplingMode> Fourier<S> {
    pub fn new(sine: Sine<S>) -> Self {
        Self {
            config: sine.sampling_config(),
            components: vec![sine],
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    /// Add a sine wave component
    ///
    /// # Arguments
    /// - `sine` - [Sine] modulation
    ///
    pub fn add_component(mut self, sine: Sine<S>) -> Self {
        self.components.push(sine);
        self
    }

    /// Add sine wave components from iterator
    ///
    /// # Arguments
    /// - `iter` - Iterator of [Sine] modulation
    ///
    pub fn add_components_from_iter(mut self, iter: impl IntoIterator<Item = Sine<S>>) -> Self {
        self.components.extend(iter);
        self
    }
}

impl<S: SamplingMode> From<Sine<S>> for Fourier<S> {
    fn from(sine: Sine<S>) -> Self {
        Self::new(sine)
    }
}

impl<S: SamplingMode> Deref for Fourier<S> {
    type Target = [Sine<S>];

    fn deref(&self) -> &Self::Target {
        &self.components
    }
}

impl<S: SamplingMode> std::ops::Add<Sine<S>> for Fourier<S> {
    type Output = Self;

    fn add(self, rhs: Sine<S>) -> Self::Output {
        self.add_component(rhs)
    }
}

impl<S: SamplingMode> std::ops::Add<Sine<S>> for Sine<S> {
    type Output = Fourier<S>;

    fn add(self, rhs: Sine<S>) -> Self::Output {
        Fourier::from(self).add_component(rhs)
    }
}

impl<S: SamplingMode> Modulation for Fourier<S> {
    fn calc(&self, geometry: &Geometry) -> Result<Vec<u8>, AUTDInternalError> {
        if !self
            .components
            .iter()
            .all(|c| c.sampling_config() == self.sampling_config())
        {
            return Err(AUTDInternalError::ModulationError(
                "All components must have the same sampling configuration".to_string(),
            ));
        }

        let buffers = self
            .components
            .iter()
            .map(|c| c.calc(geometry))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(buffers
            .iter()
            .fold(
                vec![0usize; buffers.iter().fold(1, |acc, x| lcm(acc, x.len()))],
                |acc, x| {
                    acc.iter()
                        .zip(x.iter().cycle())
                        .map(|(a, &b)| a + b as usize)
                        .collect::<Vec<_>>()
                },
            )
            .iter()
            .map(|x| (x / self.components.len()) as u8)
            .collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::create_geometry;

    use super::*;

    use autd3_driver::{defined::Hz, defined::PI};

    #[test]
    fn test_fourier() -> anyhow::Result<()> {
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

        let f = (f0 + f1).add_component(f2).add_components_from_iter([f3]) + f4;

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
                ((f0_buf[i % f0_buf.len()] as usize
                    + f1_buf[i % f1_buf.len()] as usize
                    + f2_buf[i % f2_buf.len()] as usize
                    + f3_buf[i % f3_buf.len()] as usize
                    + f4_buf[i % f4_buf.len()] as usize)
                    / 5) as u8
            );
        });

        Ok(())
    }

    #[test]
    fn mismatch_sampling_config() {
        let geometry = create_geometry(1);

        let f = Fourier::new(Sine::new(50. * Hz))
            + Sine::new(50. * Hz).with_sampling_config(SamplingConfig::Freq(1000 * Hz));

        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "All components must have the same sampling configuration".to_string()
            )),
            f.calc(&geometry)
        );
    }
}
