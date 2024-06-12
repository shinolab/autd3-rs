use autd3_driver::{defined::Freq, derive::*};

use super::sampling_mode::{ExactFreq, NearestFreq, SamplingMode, SamplingModeInference};

#[derive(Modulation, Clone, PartialEq, Builder)]
pub struct Square<S: SamplingMode> {
    #[get]
    freq: S::T,
    #[getset]
    low: u8,
    #[getset]
    high: u8,
    #[getset]
    duty: f32,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    __phantom: std::marker::PhantomData<S>,
}

impl Square<ExactFreq> {
    pub const fn new<S: SamplingModeInference>(freq: S) -> Square<S::T> {
        Square {
            freq,
            low: u8::MIN,
            high: u8::MAX,
            duty: 0.5,
            config: SamplingConfig::Division(5120),
            loop_behavior: LoopBehavior::infinite(),
            __phantom: std::marker::PhantomData,
        }
    }

    pub const fn with_freq_nearest(freq: Freq<f32>) -> Square<NearestFreq> {
        Square {
            freq,
            low: u8::MIN,
            high: u8::MAX,
            duty: 0.5,
            config: SamplingConfig::Division(5120),
            loop_behavior: LoopBehavior::infinite(),
            __phantom: std::marker::PhantomData,
        }
    }
}

impl<S: SamplingMode> Modulation for Square<S> {
    fn calc(&self, geometry: &Geometry) -> ModulationCalcResult {
        if !(0.0..=1.0).contains(&self.duty) {
            return Err(AUTDInternalError::ModulationError(
                "duty must be in range from 0 to 1".to_string(),
            ));
        }

        let (n, rep) = S::validate(self.freq, self.config, geometry.ultrasound_freq())?;
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

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::info!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

// TODO: add Debug to SamplingMode and use derive(Debug)
// GRCOV_EXCL_START
impl<S: SamplingMode> std::fmt::Debug for Square<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Square")
            .field("freq", &self.freq)
            .field("low", &self.low)
            .field("high", &self.high)
            .field("duty", &self.duty)
            .field("config", &self.config)
            .field("loop_behavior", &self.loop_behavior)
            .finish()
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use autd3_driver::defined::Hz;

    use crate::tests::create_geometry;

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
        Err(AUTDInternalError::ModulationError("Frequency (150.01 Hz) cannot be output with the sampling config (Division(5120)).".to_owned())),
        150.01*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (2000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        2000.*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (2000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        2000*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (4000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        4000.*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (4000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        4000*Hz
    )]
    fn with_freq_float_exact(
        #[case] expect: Result<Vec<u8>, AUTDInternalError>,
        #[case] freq: impl SamplingModeInference,
    ) {
        let geometry = create_geometry(1);
        let m = Square::new(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(u8::MIN, m.low());
        assert_eq!(u8::MAX, m.high());
        assert_eq!(0.5, m.duty());
        assert_eq!(SamplingConfig::Division(5120), m.sampling_config());
        assert_eq!(expect, m.calc(&geometry));
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
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (2000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        2000.*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (4000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        4000.*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (-0.1 Hz) must be positive".to_owned())),
        -0.1*Hz
    )]
    fn with_freq_nearest(
        #[case] expect: Result<Vec<u8>, AUTDInternalError>,
        #[case] freq: Freq<f32>,
    ) {
        let geometry = create_geometry(1);
        let m = Square::with_freq_nearest(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(u8::MIN, m.low());
        assert_eq!(u8::MAX, m.high());
        assert_eq!(0.5, m.duty());
        assert_eq!(SamplingConfig::Division(5120), m.sampling_config());

        assert_eq!(expect, m.calc(&geometry));
    }

    #[test]
    fn with_low() -> anyhow::Result<()> {
        let geometry = create_geometry(1);
        let m = Square::new(150. * Hz).with_low(u8::MAX);
        assert_eq!(u8::MAX, m.low());
        assert!(m.calc(&geometry)?.iter().all(|&x| x == u8::MAX));

        Ok(())
    }

    #[test]
    fn with_high() -> anyhow::Result<()> {
        let geometry = create_geometry(1);
        let m = Square::new(150. * Hz).with_high(u8::MIN);
        assert_eq!(u8::MIN, m.high());
        assert!(m.calc(&geometry)?.iter().all(|&x| x == u8::MIN));

        Ok(())
    }

    #[test]
    fn with_duty() -> anyhow::Result<()> {
        let geometry = create_geometry(1);
        let m = Square::new(150. * Hz).with_duty(0.0);
        assert_eq!(m.duty(), 0.0);
        assert!(m.calc(&geometry)?.iter().all(|&x| x == u8::MIN));

        let m = Square::new(150. * Hz).with_duty(1.0);
        assert_eq!(m.duty(), 1.0);
        assert!(m.calc(&geometry)?.iter().all(|&x| x == u8::MAX));

        Ok(())
    }

    #[test]
    fn duty_out_of_range() {
        let geometry = create_geometry(1);
        assert_eq!(
            Some(AUTDInternalError::ModulationError(
                "duty must be in range from 0 to 1".to_string()
            )),
            Square::new(150. * Hz).with_duty(-0.1).calc(&geometry).err()
        );

        assert_eq!(
            Some(AUTDInternalError::ModulationError(
                "duty must be in range from 0 to 1".to_string()
            )),
            Square::new(150. * Hz).with_duty(1.1).calc(&geometry).err()
        );
    }
}
