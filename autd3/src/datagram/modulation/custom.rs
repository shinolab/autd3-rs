use std::{fmt::Debug, rc::Rc};

use autd3_core::{defined::Freq, derive::*, resampler::Resampler};

#[derive(Clone, Debug)]
pub struct CustomOption {
    resampler: Option<(Freq<f32>, Rc<dyn Resampler>)>,
}

impl Default for CustomOption {
    fn default() -> Self {
        Self { resampler: None }
    }
}

///[`Modulation`] to use arbitrary modulation data
#[derive(Modulation, Clone, Debug)]
pub struct Custom<Config, E>
where
    E: Debug,
    SamplingConfigError: From<E>,
    Config: TryInto<SamplingConfig, Error = E> + Debug + Copy,
{
    pub buffer: Vec<u8>,
    pub sampling_config: Config,
    pub option: CustomOption,
}

impl Custom<Freq<f32>, SamplingConfigError> {
    /// Create new [`Custom`] modulation with resampling
    ///
    /// # Examples
    ///
    /// ```
    /// use autd3::prelude::*;
    /// use autd3::modulation::Custom;
    /// use autd3::core::resampler::SincInterpolation;
    ///
    /// Custom {
    ///     buffer: vec![0x00, 0xFF],
    ///     sampling_config: 2.0 * kHz,
    ///     option: Default::default(),
    /// }.with_resample(4 * kHz, SincInterpolation::default());
    /// ```
    pub fn with_resample<T, E>(self, target: T, resampler: impl Resampler + 'static) -> Custom<T, E>
    where
        E: Debug,
        SamplingConfigError: From<E>,
        T: TryInto<SamplingConfig, Error = E> + Debug + Copy,
    {
        let source = self.sampling_config;
        Custom {
            buffer: self.buffer,
            sampling_config: target,
            option: CustomOption {
                resampler: Some((source, Rc::new(resampler))),
                ..self.option
            },
        }
    }
}

impl<Config, E> Modulation for Custom<Config, E>
where
    E: Debug,
    SamplingConfigError: From<E>,
    Config: TryInto<SamplingConfig, Error = E> + Debug + Copy,
{
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        Ok(if let Some((source, resampler)) = self.option.resampler {
            resampler.resample(
                &self.buffer,
                source,
                self.sampling_config
                    .try_into()
                    .map_err(SamplingConfigError::from)?,
            )
        } else {
            self.buffer
        })
    }

    fn sampling_config(&self) -> Result<SamplingConfig, ModulationError> {
        Ok(self
            .sampling_config
            .try_into()
            .map_err(SamplingConfigError::from)?)
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::defined::kHz;
    use rand::Rng;

    use autd3_core::resampler::SincInterpolation;

    use super::*;

    #[test]
    fn new() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let test_buf = (0..2).map(|_| rng.gen()).collect::<Vec<_>>();
        let custom = Custom {
            buffer: test_buf.clone(),
            sampling_config: 4. * kHz,
            option: CustomOption::default(),
        };

        let d = custom.calc()?;
        assert_eq!(test_buf, *d);

        Ok(())
    }

    #[rstest::rstest]
    #[case(vec![127, 217, 255, 217, 127, 37, 0, 37], vec![127, 255, 127, 0], 2.0 * kHz, 4.0 * kHz, SincInterpolation::default())]
    #[case(vec![127, 255, 127, 0], vec![127, 217, 255, 217, 127, 37, 0, 37], 8.0 * kHz, 4.0 * kHz, SincInterpolation::default())]
    #[test]
    fn new_with_resample(
        #[case] expected: Vec<u8>,
        #[case] buffer: Vec<u8>,
        #[case] source: Freq<f32>,
        #[case] target: Freq<f32>,
        #[case] resampler: impl Resampler + 'static,
    ) -> anyhow::Result<()> {
        let custom = Custom {
            buffer: buffer,
            sampling_config: source,
            option: CustomOption::default(),
        }
        .with_resample(target, resampler);
        assert_eq!(expected, *custom.calc()?);
        Ok(())
    }
}
