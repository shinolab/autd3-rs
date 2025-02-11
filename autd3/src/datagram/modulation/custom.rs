use std::fmt::Debug;

use autd3_core::derive::*;
use derive_new::new;

///[`Modulation`] to use arbitrary modulation data
#[derive(Modulation, Clone, Debug, new)]
pub struct Custom<Config, E>
where
    E: Debug,
    SamplingConfigError: From<E>,
    Config: TryInto<SamplingConfig, Error = E> + Debug + Copy,
{
    /// The modulation data.
    pub buffer: Vec<u8>,
    /// The sampling configuration of the modulation data.
    pub sampling_config: Config,
}

impl<Config, E> Modulation for Custom<Config, E>
where
    E: Debug,
    SamplingConfigError: From<E>,
    Config: TryInto<SamplingConfig, Error = E> + Debug + Copy,
{
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        Ok(self.buffer)
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
