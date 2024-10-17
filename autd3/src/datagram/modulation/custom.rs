use std::borrow::Borrow;

use autd3_driver::{defined::Freq, derive::*};

use super::resample::Resampler;

#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct Custom {
    buffer: Vec<u8>,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Custom {
    pub fn new<T: TryInto<SamplingConfig>>(
        buffer: impl IntoIterator<Item = impl Borrow<u8>>,
        config: T,
    ) -> Result<Self, T::Error> {
        Ok(Self {
            buffer: buffer.into_iter().map(|b| *b.borrow()).collect(),
            config: config.try_into()?,
            loop_behavior: LoopBehavior::infinite(),
        })
    }

    #[tracing::instrument(skip(buffer))]
    pub fn new_with_resample<T: TryInto<SamplingConfig> + std::fmt::Debug>(
        buffer: impl IntoIterator<Item = impl Borrow<u8>>,
        source: Freq<f32>,
        target: T,
        resampler: impl Resampler,
    ) -> Result<Self, T::Error> {
        let target = target.try_into()?;
        let buffer = resampler.resample(
            &buffer.into_iter().map(|b| *b.borrow()).collect::<Vec<_>>(),
            source,
            target,
        );
        Ok(Self {
            buffer,
            config: target,
            loop_behavior: LoopBehavior::infinite(),
        })
    }
}

impl Modulation for Custom {
    fn calc(self) -> Result<Vec<u8>, AUTDInternalError> {
        Ok(self.buffer.clone())
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::defined::kHz;
    use rand::Rng;

    use crate::modulation::resample::SincInterpolation;

    use super::*;

    #[test]
    fn new() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let test_buf = (0..2).map(|_| rng.gen()).collect::<Vec<_>>();
        let custom = Custom::new(&test_buf, 4 * kHz)?;

        assert_eq!(4. * kHz, custom.sampling_config().freq());

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
        #[case] resampler: impl Resampler,
    ) -> anyhow::Result<()> {
        let custom = Custom::new_with_resample(&buffer, source, target, resampler)?;
        assert_eq!(target, custom.sampling_config().freq());
        assert_eq!(expected, *custom.calc()?);
        Ok(())
    }
}
