use autd3_driver::{
    defined::{Angle, Freq, PI},
    derive::*,
};

use super::sampling_mode::{ExactFreq, NearestFreq, SamplingMode, SamplingModeInference};

#[derive(Modulation, Clone, PartialEq, Builder)]
pub struct Sine<S: SamplingMode> {
    #[get]
    freq: S::T,
    #[getset]
    intensity: u8,
    #[getset]
    phase: Angle,
    #[getset]
    offset: u8,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    __phantom: std::marker::PhantomData<S>,
}

impl Sine<ExactFreq> {
    pub fn new<S: SamplingModeInference>(freq: S) -> Sine<S::T> {
        Sine {
            freq,
            intensity: u8::MAX,
            phase: Angle::Rad(0.0),
            offset: u8::MAX / 2,
            config: SamplingConfig::Division(5120),
            loop_behavior: LoopBehavior::infinite(),
            __phantom: std::marker::PhantomData,
        }
    }

    pub fn with_freq_nearest(freq: Freq<f32>) -> Sine<NearestFreq> {
        Sine {
            freq,
            intensity: u8::MAX,
            phase: Angle::Rad(0.0),
            offset: u8::MAX / 2,
            config: SamplingConfig::Division(5120),
            loop_behavior: LoopBehavior::infinite(),
            __phantom: std::marker::PhantomData,
        }
    }
}

impl<S: SamplingMode> Modulation for Sine<S> {
    fn calc(&self, geometry: &Geometry) -> ModulationCalcResult {
        let (n, rep) = S::validate(self.freq, self.config, geometry.ultrasound_freq())?;
        let intensity = self.intensity;
        let offset = self.offset;
        let phase = self.phase.radian();
        Ok((0..n)
            .map(|i| {
                ((intensity as f32 / 2. * (2.0 * PI * (rep * i) as f32 / n as f32 + phase).sin())
                    + offset as f32)
                    .round() as u8
            })
            .collect())
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

// TODO: add Debug to SamplingMode and use derive(Debug)
// GRCOV_EXCL_START
impl<S: SamplingMode> std::fmt::Debug for Sine<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sine")
            .field("freq", &self.freq)
            .field("intensity", &self.intensity)
            .field("phase", &self.phase)
            .field("offset", &self.offset)
            .field("config", &self.config)
            .field("loop_behavior", &self.loop_behavior)
            .finish()
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use autd3_driver::defined::{kHz, Hz};

    use crate::tests::create_geometry;

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
        150.*Hz
    )]
    #[case(
        Ok(vec![
            127, 157, 185, 210, 230, 245, 253, 254, 248, 236, 217, 194, 166, 137, 107, 78, 52, 30,
            13, 3, 0, 3, 13, 30, 52, 78, 107, 137, 166, 194, 217, 236, 248, 254, 253, 245, 230,
            210, 185, 157, 127, 97, 69, 44, 24, 9, 1, 0, 6, 18, 37, 60, 88, 117, 147, 176, 202,
            224, 241, 251, 255, 251, 241, 224, 202, 176, 147, 117, 88, 60, 37, 18, 6, 0, 1, 9, 24,
            44, 69, 97,
        ]),
        150*Hz
    )]
    #[case(
        Ok(vec![127, 166, 202, 230, 248, 255, 248, 230, 202, 166, 127, 88, 52, 24, 6, 0, 6, 24, 52, 88]),
        200.*Hz
    )]
    #[case(
        Ok(vec![127, 166, 202, 230, 248, 255, 248, 230, 202, 166, 127, 88, 52, 24, 6, 0, 6, 24, 52, 88]),
        200*Hz
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
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (-0.1 Hz) must be positive".to_owned())),
        -0.1*Hz
    )]
    fn new(
        #[case] expect: Result<Vec<u8>, AUTDInternalError>,
        #[case] freq: impl SamplingModeInference,
    ) {
        let geometry = create_geometry(1);
        let m = Sine::new(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(u8::MAX, m.intensity());
        assert_eq!(u8::MAX / 2, m.offset());
        assert_eq!(Angle::Rad(0.0), m.phase());
        assert_eq!(SamplingConfig::Division(5120), m.sampling_config());
        assert_eq!(expect, m.calc(&geometry));
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(vec![
            127, 156, 184, 209, 229, 244, 253, 254, 249, 237, 220, 197, 171, 142, 112, 83, 57, 34,
            17, 5, 0, 1, 10, 25, 45, 70, 98,
        ]),
        150.*Hz
    )]
    #[case(
        Ok(vec![127, 166, 202, 230, 248, 255, 248, 230, 202, 166, 127, 88, 52, 24, 6, 0, 6, 24, 52, 88]),
        200.*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (2000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        2e3*Hz
    )]
    #[case(
        Err(AUTDInternalError::ModulationError("Frequency (4000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        4e3*Hz
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
        let m = Sine::with_freq_nearest(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(u8::MAX, m.intensity());
        assert_eq!(u8::MAX / 2, m.offset());
        assert_eq!(Angle::Rad(0.0), m.phase());
        assert_eq!(SamplingConfig::Division(5120), m.sampling_config());
        assert_eq!(expect, m.calc(&geometry));
    }

    #[test]
    fn test_sine_with_param() {
        let m = Sine::new(100. * Hz)
            .with_intensity(u8::MAX / 2)
            .with_offset(u8::MAX / 4)
            .with_phase(PI / 4.0 * rad)
            .with_sampling_config(SamplingConfig::FreqNearest(10.1 * kHz));
        assert_eq!(u8::MAX / 2, m.intensity);
        assert_eq!(u8::MAX / 4, m.offset);
        assert_eq!(PI / 4.0 * rad, m.phase);
        assert_eq!(SamplingConfig::FreqNearest(10.1 * kHz), m.config);
    }
}
