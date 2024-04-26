use autd3_driver::{defined::PI, derive::*};

use num::integer::gcd;

use super::sampling_mode::SamplingMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactFrequency;
impl SamplingMode for ExactFrequency {
    type D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration);
    fn calc(freq: f64, data: Self::D) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let (intensity, phase, offset, sampling_config) = data;

        let fd = freq * sampling_config.division() as f64;
        if fd.fract() > 1e-9 {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) cannot be output with the sampling config ({}).",
                freq, sampling_config
            )));
        }
        let fd = fd as u64;
        let fs = SamplingConfiguration::BASE_FREQUENCY as u64;

        let k = gcd(fs, fd);
        if k >= SamplingConfiguration::BASE_FREQUENCY as u64 / 2 {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) is equal to or greater than the Nyquist frequency ({})",
                freq,
                sampling_config.freq() / 2.
            )));
        }

        let n = fs / k;
        let rep = fd / k;

        let intensity = intensity.value() as f64;
        let phase = phase.radian();
        let offset = offset.value() as f64;
        Ok((0..n)
            .map(|i| {
                (((intensity / 2. * (2.0 * PI * (rep * i) as f64 / n as f64 + phase).sin())
                    + offset)
                    .round() as u8)
                    .into()
            })
            .collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NearestFrequency;
impl SamplingMode for NearestFrequency {
    type D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration);
    fn calc(freq: f64, data: Self::D) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let (intensity, phase, offset, sampling_config) = data;

        let sf = sampling_config.freq();
        if freq >= sf / 2. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) is equal to or greater than the Nyquist frequency ({})",
                freq,
                sampling_config.freq() / 2.
            )));
        }

        let n = (sf / freq).round() as usize;
        let intensity = intensity.value() as f64;
        let phase = phase.radian();
        let offset = offset.value() as f64;
        Ok((0..n)
            .map(|i| {
                (((intensity / 2. * (2.0 * PI * i as f64 / n as f64 + phase).sin()) + offset)
                    .round() as u8)
                    .into()
            })
            .collect())
    }
}

/// Sine wave modulation
#[derive(Modulation, Clone, PartialEq, Debug, Builder)]
pub struct Sine<S: SamplingMode<D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration)>> {
    #[get]
    freq: f64,
    #[getset]
    intensity: EmitIntensity,
    #[getset]
    phase: Phase,
    #[getset]
    offset: EmitIntensity,
    config: SamplingConfiguration,
    loop_behavior: LoopBehavior,
    __phantom: std::marker::PhantomData<S>,
}

impl Sine<ExactFrequency> {
    pub const fn new(freq: f64) -> Self {
        Self::with_freq_exact(freq)
    }

    pub const fn with_freq_exact(freq: f64) -> Sine<ExactFrequency> {
        Sine {
            freq,
            intensity: EmitIntensity::MAX,
            phase: Phase::new(0),
            offset: EmitIntensity::new(127),
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::Infinite,
            __phantom: std::marker::PhantomData,
        }
    }

    pub const fn with_freq_nearest(freq: f64) -> Sine<NearestFrequency> {
        Sine {
            freq,
            intensity: EmitIntensity::MAX,
            phase: Phase::new(0),
            offset: EmitIntensity::new(127),
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::Infinite,
            __phantom: std::marker::PhantomData,
        }
    }
}

impl<S: SamplingMode<D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration)>> Modulation
    for Sine<S>
{
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        if self.freq < 0. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) must be positive",
                self.freq
            )));
        }

        S::calc(
            self.freq,
            (self.intensity, self.phase, self.offset, self.config),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(vec![
            127, 157, 185, 210, 230, 245, 253, 254, 248, 236, 217, 194, 166, 137, 107, 78, 52, 30,
            13, 3, 0, 3, 13, 30, 52, 78, 107, 137, 166, 194, 217, 236, 248, 254, 253, 245, 230,
            210, 185, 157, 127, 97, 69, 44, 24, 9, 1, 0, 6, 18, 37, 60, 88, 117, 147, 176, 202,
            224, 241, 251, 255, 251, 241, 224, 202, 176, 147, 117, 88, 60, 37, 18, 6, 0, 1, 9, 24,
            44, 69, 97,
        ]),
        150.
    )]
    #[case(
        Ok(vec![127, 166, 202, 230, 248, 255, 248, 230, 202, 166, 127, 88, 52, 24, 6, 0, 6, 24, 52, 88]),
        200.
    )]
    #[case(
        Ok(vec![
            127, 247, 208, 61, 2, 108, 239, 221, 78, 0, 90, 229, 233, 96, 0, 72, 217, 242, 115, 3,
            56, 203, 249, 133, 9, 41, 187, 253, 152, 18, 28, 170, 255, 170, 28, 18, 152, 253, 187,
            41, 9, 133, 249, 203, 56, 3, 115, 242, 217, 72, 0, 96, 233, 229, 90, 0, 78, 221, 239,
            108, 2, 61, 208, 247, 127, 7, 46, 193, 252, 146, 15, 33, 176, 254, 164, 25, 21, 158,
            254, 182, 37, 12, 139, 251, 198, 51, 5, 121, 245, 213, 67, 1, 102, 236, 226, 84, 0, 84,
            226, 236, 102, 1, 67, 213, 245, 121, 5, 51, 198, 251, 139, 12, 37, 182, 254, 158, 21,
            25, 164, 254, 176, 33, 15, 146, 252, 193, 46, 7
        ]),
        781.25
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (2000) is equal to or greater than the Nyquist frequency (2000)".to_owned())),
        2000.
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (4000) is equal to or greater than the Nyquist frequency (2000)".to_owned())),
        4000.
    )]
    fn with_freq_exact(#[case] expect: Result<Vec<u8>, AUTDInternalError>, #[case] freq: f64) {
        let m = Sine::with_freq_exact(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(EmitIntensity::MAX, m.intensity());
        assert_eq!(EmitIntensity::MAX / 2, m.offset());
        assert_eq!(Phase::new(0), m.phase());
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
            127, 156, 184, 209, 229, 244, 253, 254, 249, 237, 220, 197, 171, 142, 112, 83, 57, 34,
            17, 5, 0, 1, 10, 25, 45, 70, 98,
        ]),
        150.
    )]
    #[case(
        Ok(vec![127, 166, 202, 230, 248, 255, 248, 230, 202, 166, 127, 88, 52, 24, 6, 0, 6, 24, 52, 88]),
        200.
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (2000) is equal to or greater than the Nyquist frequency (2000)".to_owned())),
        2e3
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (4000) is equal to or greater than the Nyquist frequency (2000)".to_owned())),
        4e3
    )]
    fn with_freq_nearest(#[case] expect: Result<Vec<u8>, AUTDInternalError>, #[case] freq: f64) {
        let m = Sine::with_freq_nearest(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(EmitIntensity::MAX, m.intensity());
        assert_eq!(EmitIntensity::MAX / 2, m.offset());
        assert_eq!(Phase::new(0), m.phase());
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
                "Frequency (-0.1) must be positive".to_string()
            )),
            Sine::with_freq_nearest(-0.1).calc()
        );
    }

    #[test]
    fn test_sine_with_param() {
        let m = Sine::new(100.)
            .with_intensity(EmitIntensity::MAX / 2)
            .with_offset(EmitIntensity::MAX / 4)
            .with_phase(PI / 4.0 * Rad)
            .with_sampling_config(SamplingConfiguration::from_freq_nearest(10.1).unwrap());
        assert_eq!(EmitIntensity::MAX / 2, m.intensity);
        assert_eq!(EmitIntensity::MAX / 4, m.offset);
        assert_eq!(PI / 4.0 * Rad, m.phase);
        assert_eq!(
            SamplingConfiguration::from_freq_nearest(10.1).unwrap(),
            m.config
        );
    }

    #[test]
    fn test_sine_derive() {
        let m = Sine::new(150.);
        assert_eq!(m, m.clone());
    }
}
