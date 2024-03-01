use autd3_driver::derive::*;

use num::integer::gcd;

use super::sampling_mode::SamplingMode;

/// Square wave modulation
#[derive(Modulation, Clone, PartialEq, Debug, Builder)]
pub struct Square {
    #[get]
    freq: float,
    #[getset]
    low: EmitIntensity,
    #[getset]
    high: EmitIntensity,
    #[getset]
    duty: float,
    #[getset]
    mode: SamplingMode,
    config: SamplingConfiguration,
    loop_behavior: LoopBehavior,
}

impl Square {
    /// constructor.
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of the square wave \[Hz\]
    ///
    pub const fn new(freq: float) -> Self {
        Self {
            freq,
            low: EmitIntensity::MIN,
            high: EmitIntensity::MAX,
            duty: 0.5,
            config: SamplingConfiguration::FREQ_4K_HZ,
            mode: SamplingMode::ExactFrequency,
            loop_behavior: LoopBehavior::Infinite,
        }
    }
}

impl Modulation for Square {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        if !(0.0..=1.0).contains(&self.duty) {
            return Err(AUTDInternalError::ModulationError(
                "duty must be in range from 0 to 1".to_string(),
            ));
        }

        let (d, n) = match self.mode {
            SamplingMode::ExactFrequency => {
                if self.sampling_config().frequency().fract() != 0.0 {
                    return Err(AUTDInternalError::ModulationError(
                        "Sampling frequency must be integer".to_string(),
                    ));
                }
                if self.freq.fract() != 0.0 {
                    return Err(AUTDInternalError::ModulationError(
                        "Frequency must be integer".to_string(),
                    ));
                }
                let sf = self.sampling_config().frequency() as usize;
                let freq = (self.freq as usize).clamp(1, sf / 2);
                let k = gcd(sf, freq);
                (freq / k, sf / k)
            }
            SamplingMode::SizeOptimized => {
                let sf = self.sampling_config().frequency();
                let freq = self.freq.clamp(0., sf / 2.);
                (1, (sf / freq).round() as usize)
            }
        };

        Ok((0..d)
            .map(|i| (n + i) / d)
            .flat_map(|size| {
                let n_high = (size as float * self.duty) as usize;
                vec![self.high; n_high]
                    .into_iter()
                    .chain(vec![self.low; size - n_high])
            })
            .collect())
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
        let m = Square::new(150.).with_cache();
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
        let m = Square::new(150.).with_mode(SamplingMode::SizeOptimized);
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
        let m = Square::new(100.);
        assert_eq!(100., m.freq());
        assert_eq!(EmitIntensity::MAX, m.high());
        assert_eq!(EmitIntensity::MIN, m.low());
        assert_eq!(SamplingMode::ExactFrequency, m.mode());

        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "Frequency must be integer".to_string()
            )),
            Square::new(100.1).calc()
        );

        assert!(Square::new(100.1)
            .with_mode(SamplingMode::SizeOptimized)
            .calc()
            .is_ok());
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
