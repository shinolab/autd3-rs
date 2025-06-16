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
    pub fn new(
        buffer: impl IntoIterator<Item = impl std::borrow::Borrow<u8>>,
        sampling_config: Config,
    ) -> Self {
        Self {
            buffer: buffer.into_iter().map(|v| *v.borrow()).collect(),
            sampling_config,
        }
    }
}

impl<Config> Modulation for Custom<Config>
where
    Config: Into<SamplingConfig> + Debug + Copy,
{
    fn calc(self, _: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
        Ok(self.buffer)
    }

    fn sampling_config(&self) -> SamplingConfig {
        self.sampling_config.into()
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::common::kHz;
    use rand::Rng;

    use super::*;

    #[test]
    fn new() -> anyhow::Result<()> {
        let mut rng = rand::rng();

        let test_buf = (0..2).map(|_| rng.random()).collect::<Vec<u8>>();
        let custom = Custom::new(test_buf.clone(), 4. * kHz);

        let d = custom.calc(&FirmwareLimits::unused())?;
        assert_eq!(test_buf, *d);

        Ok(())
    }
}
