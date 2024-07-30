use autd3_driver::{
    defined::{Angle, Freq, PI},
    derive::*,
};

use super::sampling_mode::{ExactFreq, NearestFreq, SamplingMode, SamplingModeInference};

use derivative::Derivative;
use derive_more::Display;

#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Modulation, Clone, PartialEq, Builder, Display)]
#[display(
    fmt = "Sine<{}> {{ {}, {}±{}, {:?}, {:?}, {:?} }}",
    "tynm::type_name::<S>()",
    freq,
    offset,
    "*intensity as f32 / 2.",
    phase,
    config,
    loop_behavior
)]
pub struct Sine<S: SamplingMode> {
    freq: S::T,
    #[get]
    #[set]
    intensity: u8,
    #[get]
    #[set]
    phase: Angle,
    #[get]
    #[set]
    offset: u8,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    #[derivative(Debug = "ignore")]
    __phantom: std::marker::PhantomData<S>,
}

impl Sine<ExactFreq> {
    pub const fn new<S: SamplingModeInference>(freq: S) -> Sine<S::T> {
        Sine {
            freq,
            intensity: u8::MAX,
            phase: Angle::Rad(0.0),
            offset: u8::MAX / 2,
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
            __phantom: std::marker::PhantomData,
        }
    }

    pub const fn new_nearest(freq: Freq<f32>) -> Sine<NearestFreq> {
        Sine {
            freq,
            intensity: u8::MAX,
            phase: Angle::Rad(0.0),
            offset: u8::MAX / 2,
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
            __phantom: std::marker::PhantomData,
        }
    }
}

impl<S: SamplingMode> Sine<S> {
    pub fn freq(&self) -> S::T {
        S::freq(self.freq, self.config)
    }
}

impl<S: SamplingMode> Modulation for Sine<S> {
    fn calc(&self) -> ModulationCalcResult {
        let (n, rep) = S::validate(self.freq, self.config)?;
        let intensity = self.intensity;
        let offset = self.offset;
        let phase = self.phase.radian();
        Ok(Arc::new(
            (0..n)
                .map(|i| {
                    ((intensity as f32 / 2.
                        * (2.0 * PI * (rep * i) as f32 / n as f32 + phase).sin())
                        + offset as f32)
                        .round() as u8
                })
                .collect(),
        ))
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
}

// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use autd3_driver::defined::{kHz, Hz};

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(Arc::new(vec![
            127, 157, 185, 210, 230, 245, 253, 254, 248, 236, 217, 194, 166, 137, 107, 78, 52, 30,
            13, 3, 0, 3, 13, 30, 52, 78, 107, 137, 166, 194, 217, 236, 248, 254, 253, 245, 230,
            210, 185, 157, 127, 97, 69, 44, 24, 9, 1, 0, 6, 18, 37, 60, 88, 117, 147, 176, 202,
            224, 241, 251, 255, 251, 241, 224, 202, 176, 147, 117, 88, 60, 37, 18, 6, 0, 1, 9, 24,
            44, 69, 97,
        ])),
        150.*Hz
    )]
    #[case(
        Ok(Arc::new(vec![
            127, 157, 185, 210, 230, 245, 253, 254, 248, 236, 217, 194, 166, 137, 107, 78, 52, 30,
            13, 3, 0, 3, 13, 30, 52, 78, 107, 137, 166, 194, 217, 236, 248, 254, 253, 245, 230,
            210, 185, 157, 127, 97, 69, 44, 24, 9, 1, 0, 6, 18, 37, 60, 88, 117, 147, 176, 202,
            224, 241, 251, 255, 251, 241, 224, 202, 176, 147, 117, 88, 60, 37, 18, 6, 0, 1, 9, 24,
            44, 69, 97,
        ])),
        150*Hz
    )]
    #[case(
        Ok(Arc::new(vec![127, 166, 202, 230, 248, 255, 248, 230, 202, 166, 127, 88, 52, 24, 6, 0, 6, 24, 52, 88])),
        200.*Hz
    )]
    #[case(
        Ok(Arc::new(vec![127, 166, 202, 230, 248, 255, 248, 230, 202, 166, 127, 88, 52, 24, 6, 0, 6, 24, 52, 88])),
        200*Hz
    )]
    #[case(
        Ok(Arc::new(vec![
            127, 247, 208, 61, 2, 108, 239, 221, 78, 0, 90, 229, 233, 96, 0, 72, 217, 242, 115, 3,
            56, 203, 249, 133, 9, 41, 187, 253, 152, 18, 28, 170, 255, 170, 28, 18, 152, 253, 187,
            41, 9, 133, 249, 203, 56, 3, 115, 242, 217, 72, 0, 96, 233, 229, 90, 0, 78, 221, 239,
            108, 2, 61, 208, 247, 127, 7, 46, 193, 252, 146, 15, 33, 176, 254, 164, 25, 21, 158,
            254, 182, 37, 12, 139, 251, 198, 51, 5, 121, 245, 213, 67, 1, 102, 236, 226, 84, 0, 84,
            226, 236, 102, 1, 67, 213, 245, 121, 5, 51, 198, 251, 139, 12, 37, 182, 254, 158, 21,
            25, 164, 254, 176, 33, 15, 146, 252, 193, 46, 7
        ])),
        781.25*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (150.01 Hz) cannot be output with the sampling config (4000 Hz).".to_owned())),
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
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (-0.1 Hz) must be valid positive value".to_owned())),
        -0.1*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency must not be zero. If intentional, Use `Static` instead.".to_owned())),
        0*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency must not be zero. If intentional, Use `Static` instead.".to_owned())),
        0.*Hz
    )]
    fn new(#[case] expect: ModulationCalcResult, #[case] freq: impl SamplingModeInference) {
        let m = Sine::new(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(u8::MAX, m.intensity());
        assert_eq!(u8::MAX / 2, m.offset());
        assert_eq!(Angle::Rad(0.0), m.phase());
        assert_eq!(SamplingConfig::FREQ_4K, m.sampling_config());
        assert_eq!(expect, m.calc());
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(Arc::new(vec![
            127, 156, 184, 209, 229, 244, 253, 254, 249, 237, 220, 197, 171, 142, 112, 83, 57, 34,
            17, 5, 0, 1, 10, 25, 45, 70, 98,
        ])),
        150.*Hz
    )]
    #[case(
        Ok(Arc::new(vec![127, 166, 202, 230, 248, 255, 248, 230, 202, 166, 127, 88, 52, 24, 6, 0, 6, 24, 52, 88])),
        200.*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (NaN Hz) must be valid value".to_owned())),
        f32::NAN * Hz
    )]
    fn new_nearest(#[case] expect: ModulationCalcResult, #[case] freq: Freq<f32>) {
        let m = Sine::new_nearest(freq);
        if !freq.hz().is_nan() {
            assert_eq!(freq, m.freq());
        }
        assert_eq!(u8::MAX, m.intensity());
        assert_eq!(u8::MAX / 2, m.offset());
        assert_eq!(Angle::Rad(0.0), m.phase());
        assert_eq!(SamplingConfig::FREQ_4K, m.sampling_config());
        assert_eq!(expect, m.calc());
    }

    #[test]
    fn test_sine_with_param() {
        let m = Sine::new(100. * Hz)
            .with_intensity(u8::MAX / 2)
            .with_offset(u8::MAX / 4)
            .with_phase(PI / 4.0 * rad)
            .with_sampling_config_nearest(10.1 * kHz);
        assert_eq!(u8::MAX / 2, m.intensity);
        assert_eq!(u8::MAX / 4, m.offset);
        assert_eq!(PI / 4.0 * rad, m.phase);
        assert_eq!((10.1 * kHz).into_sampling_config_nearest(), m.config);
    }
}
