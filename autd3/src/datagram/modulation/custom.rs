use autd3_driver::derive::*;

#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct Custom {
    buffer: Vec<u8>,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Custom {
    pub fn new(buffer: Vec<u8>, config: impl Into<SamplingConfig>) -> Self {
        Self {
            buffer,
            config: config.into(),
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
    use rand::Rng;

    use super::*;

    #[test]
    fn test_custom() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let test_buf = (0..2).map(|_| rng.gen()).collect::<Vec<_>>();
        let custom = Custom::new(test_buf.clone(), SamplingConfig::FREQ_4K);

        let d = custom.calc()?;
        assert_eq!(d, test_buf);

        Ok(())
    }
}
