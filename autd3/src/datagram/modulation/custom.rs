use autd3_driver::derive::*;

#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct Custom {
    buffer: Vec<u8>,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Custom {
    pub fn new(buffer: Vec<u8>, config: SamplingConfig) -> Self {
        Self {
            buffer,
            config,
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    // GRCOV_EXCL_START
    #[deprecated(note = "Do not change the sampling configuration", since = "25.3.0")]
    pub fn with_sampling_config(self, config: SamplingConfig) -> Self {
        Self { config, ..self }
    }
    // GRCOV_EXCL_STOP
}
impl Modulation for Custom {
    fn calc(&self, _: &Geometry) -> ModulationCalcResult {
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

    use crate::tests::create_geometry;

    use super::*;

    #[test]
    fn test_custom() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(2);

        let test_buf = (0..2).map(|_| rng.gen()).collect::<Vec<_>>();
        let custom = Custom::new(test_buf.clone(), SamplingConfig::Division(5120));

        let d = custom.calc(&geometry)?;
        assert_eq!(d, test_buf);

        Ok(())
    }
}
