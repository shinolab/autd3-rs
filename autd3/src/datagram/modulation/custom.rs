use autd3_driver::derive::*;

#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct Custom {
    buffer: Vec<u8>,
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
}
impl Modulation for Custom {
    fn calc(&self, _: &Geometry) -> ModulationCalcResult {
        Ok(self.buffer.clone())
    }
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
