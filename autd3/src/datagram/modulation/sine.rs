use std::f32::consts::PI;

use autd3_core::{
    common::{Angle, Freq, rad},
    derive::*,
    firmware::SamplingConfig,
};

use super::sampling_mode::{Nearest, SamplingMode};

use derive_more::Debug;

/// The option of [`Sine`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SineOption {
    /// The intensity of the modulation. The default value is [`u8::MAX`].
    pub intensity: u8,
    /// The offset of the modulation. The default value is `0x80`.
    pub offset: u8,
    /// The phase of the modulation. The default value is `0 rad`.
    pub phase: Angle,
    /// If `true`, the modulation value is clamped to the range of `u8`. If `false`, returns an error if the value is out of range. The default value is `false`.
    pub clamp: bool,
    /// The sampling configuration of the modulation. The default value is [`SamplingConfig::FREQ_4K`].
    pub sampling_config: SamplingConfig,
}

impl Default for SineOption {
    fn default() -> Self {
        Self {
            intensity: u8::MAX,
            offset: 0x80,
            phase: 0. * rad,
            clamp: false,
            sampling_config: SamplingConfig::FREQ_4K,
        }
    }
}

/// Sine wave modulation
///
/// The modulation value is calculated as `⌊intensity / 2 * sin(2 * PI * freq * t + phase) + offset⌋`, where `t` is time, and `intensity`, `offset`, and `phase` can be set by the [`SineOption`].
#[derive(Modulation, Clone, Copy, PartialEq, Debug)]
pub struct Sine<S: Into<SamplingMode> + Clone + Copy + std::fmt::Debug> {
    /// The frequency of the sine wave.
    pub freq: S,
    /// The option of the modulation.
    pub option: SineOption,
}

impl<S: Into<SamplingMode> + Clone + Copy + std::fmt::Debug> Sine<S> {
    /// Create a new [`Sine`].
    #[must_use]
    pub const fn new(freq: S, option: SineOption) -> Self {
        Self { freq, option }
    }
}

impl Sine<Freq<f32>> {
    /// Converts to the nearest frequency that can be output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use autd3::prelude::*;
    /// Sine {
    ///     freq: 150.0 * Hz,
    ///     option: Default::default(),
    /// }.into_nearest();
    /// ```
    #[must_use]
    pub const fn into_nearest(self) -> Sine<Nearest> {
        Sine {
            freq: Nearest(self.freq),
            option: self.option,
        }
    }
}

impl<S: Into<SamplingMode> + Clone + Copy + std::fmt::Debug> Sine<S> {
    pub(super) fn calc_raw(
        &self,
        limits: &FirmwareLimits,
    ) -> Result<impl Iterator<Item = f32>, ModulationError> {
        let sampling_mode: SamplingMode = self.freq.into();
        let (n, rep) = sampling_mode.validate(self.option.sampling_config, limits)?;
        let intensity = self.option.intensity;
        let offset = self.option.offset;
        let phase = self.option.phase.radian();
        Ok((0..n).map(move |i| {
            (intensity as f32 / 2. * (2.0 * PI * (rep * i) as f32 / n as f32 + phase).sin())
                + offset as f32
        }))
    }
}

impl<S: Into<SamplingMode> + Clone + Copy + std::fmt::Debug> Modulation for Sine<S> {
    fn calc(self, limits: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
        self.calc_raw(limits)?
            .map(|v| v.floor() as i16)
            .map(|v| {
                if (u8::MIN as i16..=u8::MAX as _).contains(&v) {
                    Ok(v as _)
                } else if self.option.clamp {
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

    fn sampling_config(&self) -> SamplingConfig {
        self.option.sampling_config
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::{
        common::{Hz, rad},
        firmware::{driver::Driver, v12_1::V12_1},
    };

    use super::*;

    #[rstest::rstest]
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
        Err(ModulationError::new("Frequency (150.01 Hz) cannot be output with the sampling config (SamplingConfig::Freq(4000 Hz)).")),
        150.01*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (2000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)")),
        2000.*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (2000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)")),
        2000*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (4000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)")),
        4000.*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (4000 Hz) is equal to or greater than the Nyquist frequency (2000 Hz)")),
        4000*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency (-0.1 Hz) must be valid positive value")),
        -0.1*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency must not be zero. If intentional, Use `Static` instead.")),
        0*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency must not be zero. If intentional, Use `Static` instead.")),
        0.*Hz
    )]
    fn new(
        #[case] expect: Result<Vec<u8>, ModulationError>,
        #[case] freq: impl Into<SamplingMode> + Copy + std::fmt::Debug,
    ) {
        let m = Sine::new(freq, SineOption::default());
        assert_eq!(u8::MAX, m.option.intensity);
        assert_eq!(0x80, m.option.offset);
        assert_eq!(0. * rad, m.option.phase);
        assert_eq!(SamplingConfig::FREQ_4K, m.sampling_config());
        assert_eq!(expect, m.calc(&V12_1.firmware_limits()));
    }

    #[rstest::rstest]
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
        Err(ModulationError::new("Frequency (NaN Hz) must be valid value")),
        f32::NAN * Hz
    )]
    fn new_nearest(#[case] expect: Result<Vec<u8>, ModulationError>, #[case] freq: Freq<f32>) {
        let m = Sine {
            freq,
            option: SineOption::default(),
        }
        .into_nearest();
        assert_eq!(u8::MAX, m.option.intensity);
        assert_eq!(0x80, m.option.offset);
        assert_eq!(0. * rad, m.option.phase);
        assert_eq!(SamplingConfig::FREQ_4K, m.sampling_config());
        assert_eq!(expect, m.calc(&V12_1.firmware_limits()));
    }

    #[rstest::rstest]
    #[case(
        Err(ModulationError::new("Sine modulation value (-1) is out of range [0, 255]")),
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
            Sine {
                freq: 200 * Hz,
                option: SineOption {
                    offset,
                    clamp,
                    ..Default::default()
                }
            }
            .calc(&V12_1.firmware_limits())
        );
    }
}
