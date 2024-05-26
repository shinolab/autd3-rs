use autd3_driver::{defined::Freq, derive::*};

use super::sampling_mode::{ExactFreq, NearestFreq, SamplingMode, SamplingModeInference};

#[derive(Modulation, Clone, PartialEq, Debug, Builder)]
pub struct Square<S: SamplingMode> {
    #[get]
    freq: S::T,
    #[getset]
    low: EmitIntensity,
    #[getset]
    high: EmitIntensity,
    #[getset]
    duty: f64,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    __phantom: std::marker::PhantomData<S>,
}

impl Square<ExactFreq> {
    pub const fn new<S: SamplingModeInference>(freq: S) -> Square<S::T> {
        Square {
            freq,
            low: EmitIntensity::MIN,
            high: EmitIntensity::MAX,
            duty: 0.5,
            config: SamplingConfig::Division(5120),
            loop_behavior: LoopBehavior::infinite(),
            __phantom: std::marker::PhantomData,
        }
    }

    pub const fn with_freq_nearest(freq: Freq<f64>) -> Square<NearestFreq> {
        Square {
            freq,
            low: EmitIntensity::MIN,
            high: EmitIntensity::MAX,
            duty: 0.5,
            config: SamplingConfig::Division(5120),
            loop_behavior: LoopBehavior::infinite(),
            __phantom: std::marker::PhantomData,
        }
    }
}

impl<S: SamplingMode> Modulation for Square<S> {
    fn calc(&self, geometry: &Geometry) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        if !(0.0..=1.0).contains(&self.duty) {
            return Err(AUTDInternalError::ModulationError(
                "duty must be in range from 0 to 1".to_string(),
            ));
        }

        let (n, rep) = S::validate(self.freq, self.config, geometry.ultrasound_freq())?;
        Ok((0..rep)
            .map(|i| (n + i) / rep)
            .flat_map(|size| {
                let n_high = (size as f64 * self.duty) as usize;
                vec![self.high; n_high]
                    .into_iter()
                    .chain(vec![self.low; size as usize - n_high])
            })
            .map(EmitIntensity::from)
            .collect())
    }
}

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
        assert_eq!(EmitIntensity::MIN, m.low());
        assert_eq!(EmitIntensity::MAX, m.high());
        assert_eq!(0.5, m.duty());
        assert_eq!(SamplingConfig::Division(5120), m.sampling_config());
        assert_eq!(
            expect.map(|v| v.into_iter().map(EmitIntensity::from).collect()),
            m.calc(&geometry)
        );
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
        #[case] freq: Freq<f64>,
    ) {
        let geometry = create_geometry(1);
        let m = Square::with_freq_nearest(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(EmitIntensity::MIN, m.low());
        assert_eq!(EmitIntensity::MAX, m.high());
        assert_eq!(0.5, m.duty());
        assert_eq!(SamplingConfig::Division(5120), m.sampling_config());

        assert_eq!(
            expect.map(|v| v.into_iter().map(EmitIntensity::from).collect()),
            m.calc(&geometry)
        );
    }

    #[test]
    fn with_low() -> anyhow::Result<()> {
        let geometry = create_geometry(1);
        let m = Square::new(150. * Hz).with_low(u8::MAX);
        assert_eq!(EmitIntensity::MAX, m.low());
        assert!(m.calc(&geometry)?.iter().all(|&x| x == EmitIntensity::MAX));

        Ok(())
    }

    #[test]
    fn with_high() -> anyhow::Result<()> {
        let geometry = create_geometry(1);
        let m = Square::new(150. * Hz).with_high(u8::MIN);
        assert_eq!(EmitIntensity::MIN, m.high());
        assert!(m.calc(&geometry)?.iter().all(|&x| x == EmitIntensity::MIN));

        Ok(())
    }

    #[test]
    fn with_duty() -> anyhow::Result<()> {
        let geometry = create_geometry(1);
        let m = Square::new(150. * Hz).with_duty(0.0);
        assert_eq!(m.duty(), 0.0);
        assert!(m.calc(&geometry)?.iter().all(|&x| x == EmitIntensity::MIN));

        let m = Square::new(150. * Hz).with_duty(1.0);
        assert_eq!(m.duty(), 1.0);
        assert!(m.calc(&geometry)?.iter().all(|&x| x == EmitIntensity::MAX));

        Ok(())
    }

    #[test]
    fn duty_out_of_range() {
        let geometry = create_geometry(1);
        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "duty must be in range from 0 to 1".to_string()
            )),
            Square::new(150. * Hz).with_duty(-0.1).calc(&geometry)
        );

        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "duty must be in range from 0 to 1".to_string()
            )),
            Square::new(150. * Hz).with_duty(1.1).calc(&geometry)
        );
    }

    #[test]
    fn test_square_derive() {
        let m = Square::new(150. * Hz);
        assert_eq!(m, m.clone());
    }
}
