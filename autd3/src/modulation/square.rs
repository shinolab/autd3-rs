use autd3_driver::derive::*;

use num::integer::gcd;

use super::sampling_mode::SamplingMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactFrequency;
impl SamplingMode for ExactFrequency {
    type F = usize;
    type D = (EmitIntensity, EmitIntensity, f64, SamplingConfiguration);
    fn calc(freq: Self::F, data: Self::D) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let (low, high, duty, sampling_config) = data;

        if sampling_config.frequency().fract() != 0.0 {
            return Err(AUTDInternalError::ModulationError(
                "Sampling frequency must be integer".to_string(),
            ));
        }
        let sf = sampling_config.frequency() as usize;
        let freq = freq.clamp(1, sf / 2);
        let k = gcd(sf, freq);
        let d = freq / k;
        let n = sf / k;

        Ok((0..d)
            .map(|i| (n + i) / d)
            .flat_map(|size| {
                let n_high = (size as f64 * duty) as usize;
                vec![high; n_high]
                    .into_iter()
                    .chain(vec![low; size - n_high])
            })
            .collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SizeOptimized;
impl SamplingMode for SizeOptimized {
    type F = f64;
    type D = (EmitIntensity, EmitIntensity, f64, SamplingConfiguration);
    fn calc(freq: Self::F, data: Self::D) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let (low, high, duty, sampling_config) = data;

        let sf = sampling_config.frequency();
        let freq = freq.clamp(0., sf / 2.);
        let n = (sf / freq).round() as usize;
        let n_high = (n as f64 * duty) as usize;
        Ok(vec![high; n_high]
            .into_iter()
            .chain(vec![low; n - n_high])
            .collect())
    }
}

pub trait FrequencyType: Copy {
    type S: SamplingMode<F = Self, D = (EmitIntensity, EmitIntensity, f64, SamplingConfiguration)>;
}
impl FrequencyType for usize {
    type S = ExactFrequency;
}
impl FrequencyType for f64 {
    type S = SizeOptimized;
}

/// Square wave modulation
#[derive(Modulation, Clone, PartialEq, Debug, Builder)]
pub struct Square<F: FrequencyType> {
    #[get]
    freq: F,
    #[getset]
    low: EmitIntensity,
    #[getset]
    high: EmitIntensity,
    #[getset]
    duty: f64,
    config: SamplingConfiguration,
    loop_behavior: LoopBehavior,
}

impl<F: FrequencyType> Square<F> {
    /// constructor.
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of the square wave \[Hz\]
    ///
    pub const fn new(freq: F) -> Self {
        Self {
            freq,
            low: EmitIntensity::MIN,
            high: EmitIntensity::MAX,
            duty: 0.5,
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::Infinite,
        }
    }
}

impl<F: FrequencyType> Modulation for Square<F> {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        if !(0.0..=1.0).contains(&self.duty) {
            return Err(AUTDInternalError::ModulationError(
                "duty must be in range from 0 to 1".to_string(),
            ));
        }
        F::S::calc(self.freq, (self.low, self.high, self.duty, self.config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square() -> anyhow::Result<()> {
        let expect = [
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let m = Square::new(150).with_cache();
        assert_eq!(SamplingConfiguration::FREQ_4K_HZ, m.sampling_config());
        assert_eq!(
            expect
                .into_iter()
                .map(EmitIntensity::new)
                .collect::<Vec<_>>(),
            m.calc()?
        );

        Ok(())
    }

    #[test]
    fn test_square_with_size_opt() -> anyhow::Result<()> {
        let expect = [
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let m = Square::new(150.);
        assert_eq!(SamplingConfiguration::FREQ_4K_HZ, m.sampling_config());
        assert_eq!(
            expect
                .into_iter()
                .map(EmitIntensity::new)
                .collect::<Vec<_>>(),
            m.calc()?
        );

        Ok(())
    }

    #[test]
    fn test_square_new() {
        let m = Square::new(100);
        assert_eq!(100, m.freq());
        assert_eq!(EmitIntensity::MAX, m.high());
        assert_eq!(EmitIntensity::MIN, m.low());

        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "Sampling frequency must be integer".to_string()
            )),
            Square::new(100)
                .with_sampling_config(SamplingConfiguration::from_frequency(10.1).unwrap())
                .calc()
        );
    }

    #[test]
    fn test_square_with_low() -> anyhow::Result<()> {
        let m = Square::new(150.).with_low(EmitIntensity::MAX);
        assert_eq!(EmitIntensity::MAX, m.low());
        assert!(m.calc()?.iter().all(|&a| a == EmitIntensity::MAX));

        Ok(())
    }

    #[test]
    fn test_square_with_high() -> anyhow::Result<()> {
        let m = Square::new(150.).with_high(EmitIntensity::MIN);
        assert_eq!(EmitIntensity::MIN, m.high());
        assert!(m.calc()?.iter().all(|&a| a == EmitIntensity::MIN));

        Ok(())
    }

    #[test]
    fn test_square_with_duty() -> anyhow::Result<()> {
        let m = Square::new(150.).with_duty(0.0);
        assert_eq!(m.duty(), 0.0);
        assert!(m.calc()?.iter().all(|&a| a == EmitIntensity::MIN));

        let m = Square::new(150.).with_duty(1.0);
        assert_eq!(m.duty(), 1.0);
        assert!(m.calc()?.iter().all(|&a| a == EmitIntensity::MAX));

        Ok(())
    }

    #[test]
    fn test_square_with_duty_out_of_range() {
        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "duty must be in range from 0 to 1".to_string()
            )),
            Square::new(150.).with_duty(-0.1).calc()
        );

        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "duty must be in range from 0 to 1".to_string()
            )),
            Square::new(150.).with_duty(1.1).calc()
        );
    }

    #[test]
    fn test_square_derive() {
        let m = Square::new(150.);
        assert_eq!(m, m.clone());
    }
}
