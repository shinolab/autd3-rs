use autd3_driver::derive::*;

#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct Custom {
    buffer: Vec<u8>,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Custom {
    pub fn new(
        buffer: Vec<u8>,
        config: impl IntoSamplingConfig,
    ) -> Result<Self, AUTDInternalError> {
        Ok(Self {
            buffer,
            config: config.into_sampling_config()?,
            loop_behavior: LoopBehavior::infinite(),
        })
    }

    pub fn new_nearest(buffer: Vec<u8>, config: impl IntoSamplingConfigNearest) -> Self {
        Self {
            buffer,
            config: config.into_sampling_config_nearest(),
            loop_behavior: LoopBehavior::infinite(),
        }
    }
}
impl Modulation for Custom {
    fn calc(&self) -> ModulationCalcResult {
        Ok(self.buffer.clone())
    }

    #[tracing::instrument(level = "debug", skip(self, _geometry), fields(%self.config, %self.loop_behavior))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use autd3_driver::defined::kHz;
    use rand::Rng;

    use super::*;

    #[test]
    fn new() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let test_buf = (0..2).map(|_| rng.gen()).collect::<Vec<_>>();
        let custom = Custom::new(test_buf.clone(), 4 * kHz)?;

        assert_eq!(4. * kHz, custom.sampling_config().freq());

        let d = custom.calc()?;
        assert_eq!(d, test_buf);

        Ok(())
    }

    #[test]
    fn new_nearest() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let test_buf = (0..2).map(|_| rng.gen()).collect::<Vec<_>>();
        let custom = Custom::new_nearest(test_buf.clone(), 4 * kHz);

        assert_eq!(4. * kHz, custom.sampling_config().freq());

        let d = custom.calc()?;
        assert_eq!(d, test_buf);

        Ok(())
    }
}
