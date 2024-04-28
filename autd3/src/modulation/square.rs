use autd3_driver::derive::*;

use super::sampling_mode::{ExactFrequency, NearestFrequency, SamplingMode};

/// Square wave modulation
#[derive(Modulation, Clone, PartialEq, Debug, Builder)]
pub struct Square<S: SamplingMode> {
    #[get]
    freq: f64,
    #[getset]
    low: EmitIntensity,
    #[getset]
    high: EmitIntensity,
    #[getset]
    duty: f64,
    config: SamplingConfiguration,
    loop_behavior: LoopBehavior,
    __phantom: std::marker::PhantomData<S>,
}

impl Square<ExactFrequency> {
    pub const fn new(freq: f64) -> Self {
        Self::with_freq_exact(freq)
    }

    /// constructor.
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of the square wave \[Hz\]
    ///
    pub const fn with_freq_exact(freq: f64) -> Self {
        Self {
            freq,
            low: EmitIntensity::MIN,
            high: EmitIntensity::MAX,
            duty: 0.5,
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::infinite(),
            __phantom: std::marker::PhantomData,
        }
    }

    /// constructor.
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of the square wave \[Hz\]
    ///
    pub const fn with_freq_nearest(freq: f64) -> Square<NearestFrequency> {
        Square {
            freq,
            low: EmitIntensity::MIN,
            high: EmitIntensity::MAX,
            duty: 0.5,
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::infinite(),
            __phantom: std::marker::PhantomData,
        }
    }
}

impl<S: SamplingMode> Modulation for Square<S> {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        if self.freq < 0. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}Hz) must be positive",
                self.freq
            )));
        }
        if self.freq >= self.config.freq() / 2. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}Hz) is equal to or greater than the Nyquist frequency ({}Hz)",
                self.freq,
                self.config.freq() / 2.
            )));
        }

        if !(0.0..=1.0).contains(&self.duty) {
            return Err(AUTDInternalError::ModulationError(
                "duty must be in range from 0 to 1".to_string(),
            ));
        }

        let (n, rep) = S::validate(self.freq, self.config)?;

        Ok((0..rep)
            .map(|i| (n + i) / rep)
            .flat_map(|size| {
                let n_high = (size as f64 * self.duty) as usize;
                vec![self.high; n_high]
                    .into_iter()
                    .chain(vec![self.low; size as usize - n_high])
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(vec![
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]),
        150.
    )]
    #[case(
        Ok(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        200.
    )]
    #[case(
        Ok(vec![
            255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255,
            0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0,
            255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255,
            0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0,
            255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255, 0, 0, 0, 255, 255,
            255, 0, 0, 0, 255, 255, 255, 0, 0, 0, 255, 255, 255, 0, 0, 0
        ]),
        781.25
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (150.01Hz) cannot be output with the sampling config (4000Hz).".to_owned())),
        150.01
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (2000Hz) is equal to or greater than the Nyquist frequency (2000Hz)".to_owned())),
        2000.
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (4000Hz) is equal to or greater than the Nyquist frequency (2000Hz)".to_owned())),
        4000.
    )]
    fn with_freq_exact(#[case] expect: Result<Vec<u8>, AUTDInternalError>, #[case] freq: f64) {
        let m = Square::with_freq_exact(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(EmitIntensity::MIN, m.low());
        assert_eq!(EmitIntensity::MAX, m.high());
        assert_eq!(0.5, m.duty());
        assert_eq!(SamplingConfiguration::FREQ_4K_HZ, m.sampling_config());

        assert_eq!(
            expect.map(|v| v.into_iter().map(EmitIntensity::new).collect::<Vec<_>>()),
            m.calc()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(vec![
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ]),
        150.
    )]
    #[case(
        Ok(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        200.
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (2000Hz) is equal to or greater than the Nyquist frequency (2000Hz)".to_owned())),
        2000.
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (4000Hz) is equal to or greater than the Nyquist frequency (2000Hz)".to_owned())),
        4000.
    )]
    fn with_freq_nearest(#[case] expect: Result<Vec<u8>, AUTDInternalError>, #[case] freq: f64) {
        let m = Square::with_freq_nearest(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(EmitIntensity::MIN, m.low());
        assert_eq!(EmitIntensity::MAX, m.high());
        assert_eq!(0.5, m.duty());
        assert_eq!(SamplingConfiguration::FREQ_4K_HZ, m.sampling_config());

        assert_eq!(
            expect.map(|v| v.into_iter().map(EmitIntensity::new).collect::<Vec<_>>()),
            m.calc()
        );
    }

    #[test]
    fn freq_must_be_positive() {
        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "Frequency (-0.1Hz) must be positive".to_string()
            )),
            Square::with_freq_nearest(-0.1).calc()
        );
    }

    #[test]
    fn with_low() -> anyhow::Result<()> {
        let m = Square::new(150.).with_low(EmitIntensity::MAX);
        assert_eq!(EmitIntensity::MAX, m.low());
        assert!(m.calc()?.iter().all(|&a| a == EmitIntensity::MAX));

        Ok(())
    }

    #[test]
    fn with_high() -> anyhow::Result<()> {
        let m = Square::new(150.).with_high(EmitIntensity::MIN);
        assert_eq!(EmitIntensity::MIN, m.high());
        assert!(m.calc()?.iter().all(|&a| a == EmitIntensity::MIN));

        Ok(())
    }

    #[test]
    fn with_duty() -> anyhow::Result<()> {
        let m = Square::new(150.).with_duty(0.0);
        assert_eq!(m.duty(), 0.0);
        assert!(m.calc()?.iter().all(|&a| a == EmitIntensity::MIN));

        let m = Square::new(150.).with_duty(1.0);
        assert_eq!(m.duty(), 1.0);
        assert!(m.calc()?.iter().all(|&a| a == EmitIntensity::MAX));

        Ok(())
    }

    #[test]
    fn duty_out_of_range() {
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
