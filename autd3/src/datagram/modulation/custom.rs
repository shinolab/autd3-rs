use std::sync::Arc;

use autd3_driver::derive::*;

#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct Custom {
    buffer: Arc<Vec<u8>>,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Custom {
    pub fn new<T: TryInto<SamplingConfig>>(
        buffer: Arc<Vec<u8>>,
        config: T,
    ) -> Result<Self, T::Error> {
        Ok(Self {
            buffer,
            config: config.try_into()?,
            loop_behavior: LoopBehavior::infinite(),
        })
    }
}
impl Modulation for Custom {
    fn calc(&self) -> ModulationCalcResult {
        Ok(self.buffer.clone())
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::defined::kHz;
    use rand::Rng;

    use super::*;

    #[test]
    fn new() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let test_buf = Arc::new((0..2).map(|_| rng.gen()).collect::<Vec<_>>());
        let custom = Custom::new(test_buf.clone(), 4 * kHz)?;

        assert_eq!(4. * kHz, custom.sampling_config().freq());

        let d = custom.calc()?;
        assert_eq!(d, test_buf);

        Ok(())
    }
}
