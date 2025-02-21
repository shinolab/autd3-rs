use std::fmt::Debug;

use autd3_core::derive::*;

///[`Modulation`] to use arbitrary modulation data
#[derive(Modulation, Clone, Debug)]
pub struct Custom<Config>
where
    Config: Into<SamplingConfig> + Debug + Copy,
{
    /// The modulation data.
    pub buffer: Vec<u8>,
    /// The sampling configuration of the modulation data.
    pub sampling_config: Config,
}

impl<Config> Custom<Config>
where
    Config: Into<SamplingConfig> + Debug + Copy,
{
    /// Create a new [`Custom`].
    #[must_use]
    pub const fn new(buffer: Vec<u8>, sampling_config: Config) -> Self {
        Self {
            buffer,
            sampling_config,
        }
    }
}

impl<Config> Modulation for Custom<Config>
where
    Config: Into<SamplingConfig> + Debug + Copy,
{
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        Ok(self.buffer)
    }

    fn sampling_config(&self) -> SamplingConfig {
        self.sampling_config.into()
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::defined::kHz;
    use rand::Rng;

    use super::*;

    #[test]
    fn new() -> anyhow::Result<()> {
        let mut rng = rand::rng();

        let test_buf = (0..2).map(|_| rng.random()).collect::<Vec<_>>();
        let custom = Custom {
            buffer: test_buf.clone(),
            sampling_config: 4. * kHz,
        };

        let d = custom.calc()?;
        assert_eq!(test_buf, *d);

        Ok(())
    }
}
