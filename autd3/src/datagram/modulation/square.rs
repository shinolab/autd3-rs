use autd3_driver::{defined::Freq, derive::*};

use super::sampling_mode::{ExactFreq, NearestFreq, SamplingMode, SamplingModeInference};

use derive_more::Debug;

/// Square wave modulation
#[derive(Modulation, Clone, PartialEq, Builder, Debug)]
pub struct Square<S: SamplingMode> {
    #[debug("{}({:?})", tynm::type_name::<S>(), self.freq)]
    freq: S::T,
    #[get]
    #[set]
    /// The low value of the modulation. The default value is [`u8::MIN`].
    low: u8,
    #[get]
    #[set]
    /// The high value of the modulation. The default value is [`u8::MAX`].
    high: u8,
    #[get]
    #[set]
    /// The duty ratio of the modulation, that is the ratio of high value to the period. The default value is `0.5`.
    duty: f32,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Square<ExactFreq> {
    /// Create new [`Square`] modulation with exact frequency.
    ///
    /// # Examples
    ///
    /// ```
    /// use autd3::prelude::*;
    ///
    /// Square::new(100 * Hz);
    /// // or
    /// Square::new(100. * Hz);
    /// ```
    pub const fn new<S: SamplingModeInference>(freq: S) -> Square<S::T> {
        Square {
            freq,
            low: u8::MIN,
            high: u8::MAX,
            duty: 0.5,
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    /// Create new [`Square`] modulation with exact frequency.
    pub const fn new_nearest(freq: Freq<f32>) -> Square<NearestFreq> {
        Square {
            freq,
            low: u8::MIN,
            high: u8::MAX,
            duty: 0.5,
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
    }
}

impl<S: SamplingMode> Square<S> {
    /// The frequency of the modulation.
    pub fn freq(&self) -> S::T {
        S::freq(self.freq, self.config)
    }
}

impl<S: SamplingMode> Modulation for Square<S> {
    fn calc(self) -> Result<Vec<u8>, AUTDDriverError> {
        if !(0.0..=1.0).contains(&self.duty) {
            return Err(AUTDDriverError::ModulationError(
                "duty must be in range from 0 to 1".to_string(),
            ));
        }

        let (n, rep) = S::validate(self.freq, self.config)?;
        let high = self.high;
        let low = self.low;
        let duty = self.duty;
        Ok((0..rep)
            .map(|i| (n + i) / rep)
            .flat_map(|size| {
                let n_high = (size as f32 * duty) as usize;
                vec![high; n_high]
                    .into_iter()
                    .chain(vec![low; size as usize - n_high])
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::defined::Hz;

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
        150.*Hz
    )]
    #[case(
        Ok(vec![
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]),
        150*Hz
    )]
    #[case(
        Ok(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        200.*Hz
    )]
    #[case(
        Ok(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        200*Hz
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
        781.25*Hz
    )]
    #[case(
        Err(AUTDDriverError::ModulationError("Frequency (150.01 Hz) cannot be output with the sampling config (SamplingConfig { division: 10 }).".to_owned())),
        150.01*Hz
    )]
    #[case(
        Err(AUTDDriverError::ModulationError("Frequency (2000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        2000.*Hz
    )]
    #[case(
        Err(AUTDDriverError::ModulationError("Frequency (2000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        2000*Hz
    )]
    #[case(
        Err(AUTDDriverError::ModulationError("Frequency (4000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        4000.*Hz
    )]
    #[case(
        Err(AUTDDriverError::ModulationError("Frequency (4000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        4000*Hz
    )]
    #[case(
        Err(AUTDDriverError::ModulationError("Frequency must not be zero. If intentional, Use `Static` instead.".to_owned())),
        0*Hz
    )]
    #[case(
        Err(AUTDDriverError::ModulationError("Frequency must not be zero. If intentional, Use `Static` instead.".to_owned())),
        0.*Hz
    )]
    fn with_freq_float_exact(
        #[case] expect: Result<Vec<u8>, AUTDDriverError>,
        #[case] freq: impl SamplingModeInference,
    ) {
        let m = Square::new(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(u8::MIN, m.low());
        assert_eq!(u8::MAX, m.high());
        assert_eq!(0.5, m.duty());
        assert_eq!(SamplingConfig::FREQ_4K, m.sampling_config());
        assert_eq!(expect, m.calc());
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(vec![
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ]),
        150.*Hz
    )]
    #[case(
        Ok(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        200.*Hz
    )]
    fn new_nearest(#[case] expect: Result<Vec<u8>, AUTDDriverError>, #[case] freq: Freq<f32>) {
        let m = Square::new_nearest(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(u8::MIN, m.low());
        assert_eq!(u8::MAX, m.high());
        assert_eq!(0.5, m.duty());
        assert_eq!(SamplingConfig::FREQ_4K, m.sampling_config());

        assert_eq!(expect, m.calc());
    }

    #[test]
    fn with_low() -> anyhow::Result<()> {
        let m = Square::new(150. * Hz).with_low(u8::MAX);
        assert_eq!(u8::MAX, m.low());
        assert!(m.calc()?.iter().all(|&x| x == u8::MAX));

        Ok(())
    }

    #[test]
    fn with_high() -> anyhow::Result<()> {
        let m = Square::new(150. * Hz).with_high(u8::MIN);
        assert_eq!(u8::MIN, m.high());
        assert!(m.calc()?.iter().all(|&x| x == u8::MIN));

        Ok(())
    }

    #[test]
    fn with_duty() -> anyhow::Result<()> {
        let m = Square::new(150. * Hz).with_duty(0.0);
        assert_eq!(m.duty(), 0.0);
        assert!(m.calc()?.iter().all(|&x| x == u8::MIN));

        let m = Square::new(150. * Hz).with_duty(1.0);
        assert_eq!(m.duty(), 1.0);
        assert!(m.calc()?.iter().all(|&x| x == u8::MAX));

        Ok(())
    }

    #[test]
    fn duty_out_of_range() {
        assert_eq!(
            Some(AUTDDriverError::ModulationError(
                "duty must be in range from 0 to 1".to_string()
            )),
            Square::new(150. * Hz).with_duty(-0.1).calc().err()
        );

        assert_eq!(
            Some(AUTDDriverError::ModulationError(
                "duty must be in range from 0 to 1".to_string()
            )),
            Square::new(150. * Hz).with_duty(1.1).calc().err()
        );
    }
}
