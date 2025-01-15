use std::f32::consts::PI;

use autd3_core::{
    defined::{Angle, Freq},
    derive::*,
};
use autd3_derive::Builder;

use super::sampling_mode::{ExactFreq, NearestFreq, SamplingMode, SamplingModeInference};

use derive_more::Debug;

/// Sine wave modulation
///
/// The modulation value is calculated as `⌊intensity / 2 * sin(2 * PI * freq * t + phase) + offset⌋`.
#[derive(Modulation, Clone, PartialEq, Builder, Debug)]
pub struct Sine<S: SamplingMode> {
    #[debug("{}({:?})", tynm::type_name::<S>(), self.freq)]
    freq: S::T,
    #[get]
    #[set]
    /// The intensity of the modulation. The default value is [`u8::MAX`].
    intensity: u8,
    #[get]
    #[set]
    /// The offset of the modulation. The default value is `0x80`.
    offset: u8,
    #[get]
    #[set]
    /// The phase of the modulation. The default value is `0 rad`.
    phase: Angle,
    #[get]
    #[set]
    /// If `true`, the modulation value is clamped to the range of `u8`. If `false`, returns an error if the value is out of range. The default value is `false`.
    clamp: bool,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Sine<ExactFreq> {
    /// Creates a new [`Sine`] modulation with exact frequency.
    ///
    /// # Examples
    ///
    /// ```
    /// use autd3::prelude::*;
    ///
    /// Sine::new(100 * Hz);
    /// // or
    /// Sine::new(100. * Hz);
    /// ```
    pub const fn new<S: SamplingModeInference>(freq: S) -> Sine<S::T> {
        Sine {
            freq,
            intensity: u8::MAX,
            offset: 0x80,
            phase: Angle::Rad(0.0),
            clamp: false,
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    /// Creates a new [`Sine`] with the nearest frequency to the specified value of the possible values.
    pub const fn new_nearest(freq: Freq<f32>) -> Sine<NearestFreq> {
        Sine {
            freq,
            intensity: u8::MAX,
            phase: Angle::Rad(0.0),
            offset: 0x80,
            clamp: false,
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
    }
}

impl<S: SamplingMode> Sine<S> {
    /// The frequency of the modulation.
    pub fn freq(&self) -> S::T {
        S::freq(self.freq, self.config)
    }
}

impl<S: SamplingMode> Sine<S> {
    pub(super) fn calc_raw(&self) -> Result<impl Iterator<Item = f32>, ModulationError> {
        let (n, rep) = S::validate(self.freq, self.config)?;
        let intensity = self.intensity;
        let offset = self.offset;
        let phase = self.phase.radian();
        Ok((0..n).map(move |i| {
            (intensity as f32 / 2. * (2.0 * PI * (rep * i) as f32 / n as f32 + phase).sin())
                + offset as f32
        }))
    }
}

impl<S: SamplingMode> Modulation for Sine<S> {
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        self.calc_raw()?
            .map(|v| v.floor() as i16)
            .map(|v| {
                if (u8::MIN as _..=u8::MAX as _).contains(&v) {
                    Ok(v as _)
                } else if self.clamp {
                    Ok(v.clamp(u8::MIN as _, u8::MAX as _) as _)
                } else {
                    Err(ModulationError::new(format!(
                        "Sine modulation value ({}) is out of range [{}, {}]",
                        v,
                        u8::MIN,
                        u8::MAX,
                    )))?
                }
            })
            .collect::<Result<Vec<_>, ModulationError>>()
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::defined::{kHz, rad, Hz};

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(vec![
            128, 157, 185, 210, 231, 245, 253, 255, 249, 236, 218, 194, 167, 138, 108, 79, 53, 31, 14, 4, 0, 4, 14, 31, 53, 79, 108, 138, 167, 194, 218, 236, 249, 255, 253, 245, 231, 210, 185, 157, 128, 98, 70, 45, 24, 10, 2, 0, 6, 19, 37, 61, 88, 117, 147, 176, 202, 224, 241, 251, 255, 251, 241, 224, 202, 176, 147, 117, 88, 61, 37, 19, 6, 0, 2, 10, 24, 45, 70, 98,
        ]),
        150.*Hz
    )]
    #[case(
        Ok(vec![
            128, 157, 185, 210, 231, 245, 253, 255, 249, 236, 218, 194, 167, 138, 108, 79, 53, 31, 14, 4, 0, 4, 14, 31, 53, 79, 108, 138, 167, 194, 218, 236, 249, 255, 253, 245, 231, 210, 185, 157, 128, 98, 70, 45, 24, 10, 2, 0, 6, 19, 37, 61, 88, 117, 147, 176, 202, 224, 241, 251, 255, 251, 241, 224, 202, 176, 147, 117, 88, 61, 37, 19, 6, 0, 2, 10, 24, 45, 70, 98,
        ]),
        150*Hz
    )]
    #[case(
        Ok(vec![128, 167, 202, 231, 249, 255, 249, 231, 202, 167, 127, 88, 53, 24, 6, 0, 6, 24, 53, 88]),
        200.*Hz
    )]
    #[case(
        Ok(vec![128, 167, 202, 231, 249, 255, 249, 231, 202, 167, 127, 88, 53, 24, 6, 0, 6, 24, 53, 88]),
        200*Hz
    )]
    #[case(
        Ok(vec![
            128, 248, 208, 62, 2, 109, 240, 222, 79, 0, 90, 230, 234, 97, 1, 73, 218, 243, 115, 4, 57, 203, 250, 134, 10, 42, 188, 254, 152, 18, 29, 170, 255, 170, 29, 18, 152, 254, 188, 42, 10, 134, 250, 203, 57, 4, 115, 243, 218, 73, 1, 97, 234, 230, 90, 0, 79, 222, 240, 109, 2, 62, 208, 248, 127, 7, 47, 193, 253, 146, 15, 33, 176, 255, 165, 25, 21, 158, 254, 182, 37, 12, 140, 251, 198, 52, 5, 121, 245, 213, 67, 1, 103, 237, 226, 85, 0, 85, 226, 237, 103, 1, 67, 213, 245, 121, 5, 52, 198, 251, 140, 12, 37, 182, 254, 158, 21, 25, 165, 255, 176, 33, 15, 146, 253, 193, 47, 7
        ]),
        781.25*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (150.01 Hz) cannot be output with the sampling config (SamplingConfig { division: 10 }).".to_owned())),
        150.01*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (2000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        2000.*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (2000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        2000*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (4000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        4000.*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (4000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)".to_owned())),
        4000*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (-0.1 Hz) must be valid positive value".to_owned())),
        -0.1*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency must not be zero. If intentional, Use `Static` instead.".to_owned())),
        0*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency must not be zero. If intentional, Use `Static` instead.".to_owned())),
        0.*Hz
    )]
    fn new(
        #[case] expect: Result<Vec<u8>, ModulationError>,
        #[case] freq: impl SamplingModeInference,
    ) {
        let m = Sine::new(freq);
        assert_eq!(freq, m.freq());
        assert_eq!(u8::MAX, m.intensity());
        assert_eq!(0x80, m.offset());
        assert_eq!(Angle::Rad(0.0), m.phase());
        assert_eq!(SamplingConfig::FREQ_4K, m.sampling_config());
        assert_eq!(expect, m.calc());
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(vec![
            128, 157, 185, 209, 230, 245, 253, 255, 250, 238, 220, 198, 171, 142, 113, 84, 57, 35, 17, 5, 0, 2, 10, 25, 46, 70, 98,
        ]),
        150.*Hz
    )]
    #[case(
        Ok(vec![128, 167, 202, 231, 249, 255, 249, 231, 202, 167, 127, 88, 53, 24, 6, 0, 6, 24, 53, 88]),
        200.*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (NaN Hz) must be valid value".to_owned())),
        f32::NAN * Hz
    )]
    fn new_nearest(#[case] expect: Result<Vec<u8>, ModulationError>, #[case] freq: Freq<f32>) {
        let m = Sine::new_nearest(freq);
        if !freq.hz().is_nan() {
            assert_eq!(freq, m.freq());
        }
        assert_eq!(u8::MAX, m.intensity());
        assert_eq!(0x80, m.offset());
        assert_eq!(Angle::Rad(0.0), m.phase());
        assert_eq!(SamplingConfig::FREQ_4K, m.sampling_config());
        assert_eq!(expect, m.calc());
    }

    #[test]
    fn test_sine_with_param() -> anyhow::Result<()> {
        let m = Sine::new(100. * Hz)
            .with_intensity(0x80)
            .with_offset(0x40)
            .with_phase(PI / 4.0 * rad)
            .with_sampling_config(SamplingConfig::new_nearest(10.1 * kHz))?;
        assert_eq!(0x80, m.intensity);
        assert_eq!(0x40, m.offset);
        assert_eq!(PI / 4.0 * rad, m.phase);
        assert_eq!(SamplingConfig::new_nearest(10.1 * kHz), m.config);

        Ok(())
    }

    #[rstest::rstest]
    #[case(
        Err(ModulationError::new("Sine modulation value (-1) is out of range [0, 255]".to_owned())),
        0x00,
        false
    )]
    #[case(
        Ok(vec![0, 39, 74, 103, 121, 127, 121, 103, 74, 39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        0x00,
        true
    )]
    #[test]
    fn out_of_range(
        #[case] expect: Result<Vec<u8>, ModulationError>,
        #[case] offset: u8,
        #[case] clamp: bool,
    ) {
        assert_eq!(
            expect,
            Sine::new(200 * Hz)
                .with_offset(offset)
                .with_clamp(clamp)
                .calc()
        );
    }
}
