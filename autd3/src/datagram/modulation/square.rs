use autd3_core::{common::Freq, derive::*, firmware::SamplingConfig};

use super::sampling_mode::{Nearest, SamplingMode};

use derive_more::Debug;

/// The option of [`Square`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SquareOption {
    /// The low value of the modulation. The default value is [`u8::MIN`].
    pub low: u8,
    /// The high value of the modulation. The default value is [`u8::MAX`].
    pub high: u8,
    /// The duty ratio of the modulation, that is the ratio of high value to the period. The default value is `0.5`.
    pub duty: f32,
    /// The sampling configuration of the modulation. The default value is [`SamplingConfig::FREQ_4K`].
    pub sampling_config: SamplingConfig,
}

impl Default for SquareOption {
    fn default() -> Self {
        Self {
            low: u8::MIN,
            high: u8::MAX,
            duty: 0.5,
            sampling_config: SamplingConfig::FREQ_4K,
        }
    }
}

/// Square wave modulation
#[derive(Modulation, Clone, Copy, PartialEq, Debug)]
pub struct Square<S: Into<SamplingMode> + Clone + Copy + std::fmt::Debug> {
    /// The frequency of the square wave.
    pub freq: S,
    /// The option of the modulation.
    pub option: SquareOption,
}

impl<S: Into<SamplingMode> + Clone + Copy + std::fmt::Debug> Square<S> {
    /// Create a new [`Square`].
    #[must_use]
    pub const fn new(freq: S, option: SquareOption) -> Self {
        Self { freq, option }
    }
}

impl Square<Freq<f32>> {
    /// Converts to the nearest frequency that can be output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use autd3::prelude::*;
    /// Square {
    ///     freq: 150.0 * Hz,
    ///     option: Default::default(),
    /// }.into_nearest();
    /// ```
    #[must_use]
    pub const fn into_nearest(self) -> Square<Nearest> {
        Square {
            freq: Nearest(self.freq),
            option: self.option,
        }
    }
}

impl<S: Into<SamplingMode> + Clone + Copy + std::fmt::Debug> Modulation for Square<S> {
    fn calc(self, limits: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
        if !(0.0..=1.0).contains(&self.option.duty) {
            return Err(ModulationError::new("duty must be in range from 0 to 1"));
        }

        let sampling_mode: SamplingMode = self.freq.into();
        let (n, rep) = sampling_mode.validate(self.option.sampling_config, limits)?;
        let high = self.option.high;
        let low = self.option.low;
        let duty = self.option.duty;
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

    fn sampling_config(&self) -> SamplingConfig {
        self.option.sampling_config
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::{
        common::Hz,
        firmware::{driver::Driver, v12_1::V12_1},
    };

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
        Err(ModulationError::new("Frequency must not be zero. If intentional, Use `Static` instead.")),
        0*Hz
    )]
    #[case(
        Err(ModulationError::new("Frequency must not be zero. If intentional, Use `Static` instead.")),
        0.*Hz
    )]
    fn with_freq_float_exact(
        #[case] expect: Result<Vec<u8>, ModulationError>,
        #[case] freq: impl Into<SamplingMode> + Copy + std::fmt::Debug,
    ) {
        let m = Square::new(freq, SquareOption::default());
        assert_eq!(u8::MIN, m.option.low);
        assert_eq!(u8::MAX, m.option.high);
        assert_eq!(0.5, m.option.duty);
        assert_eq!(SamplingConfig::FREQ_4K, m.sampling_config());
        assert_eq!(expect, m.calc(&V12_1.firmware_limits()));
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
    fn into_nearest(#[case] expect: Result<Vec<u8>, ModulationError>, #[case] freq: Freq<f32>) {
        let m = Square {
            freq,
            option: SquareOption::default(),
        }
        .into_nearest();
        assert_eq!(u8::MIN, m.option.low);
        assert_eq!(u8::MAX, m.option.high);
        assert_eq!(0.5, m.option.duty);
        assert_eq!(SamplingConfig::FREQ_4K, m.sampling_config());
        assert_eq!(expect, m.calc(&V12_1.firmware_limits()));
    }

    #[test]
    fn with_low() -> anyhow::Result<()> {
        let m = Square {
            freq: 150. * Hz,
            option: SquareOption {
                low: u8::MAX,
                ..Default::default()
            },
        };
        assert!(
            m.calc(&V12_1.firmware_limits())?
                .iter()
                .all(|&x| x == u8::MAX)
        );

        Ok(())
    }

    #[test]
    fn with_high() -> anyhow::Result<()> {
        let m = Square {
            freq: 150. * Hz,
            option: SquareOption {
                high: u8::MIN,
                ..Default::default()
            },
        };
        assert!(
            m.calc(&V12_1.firmware_limits())?
                .iter()
                .all(|&x| x == u8::MIN)
        );

        Ok(())
    }

    #[rstest::rstest]
    #[case(u8::MIN, 0.0)]
    #[case(u8::MAX, 1.0)]
    #[test]
    fn with_duty(#[case] expect: u8, #[case] duty: f32) -> anyhow::Result<()> {
        let m = Square {
            freq: 150. * Hz,
            option: SquareOption {
                duty,
                ..Default::default()
            },
        };
        assert!(
            m.calc(&V12_1.firmware_limits())?
                .iter()
                .all(|&x| x == expect)
        );

        Ok(())
    }

    #[rstest::rstest]
    #[case("duty must be in range from 0 to 1", -0.1)]
    #[case("duty must be in range from 0 to 1", 1.1)]
    #[test]
    fn duty_out_of_range(#[case] expect: &str, #[case] duty: f32) {
        assert_eq!(
            Some(ModulationError::new(expect)),
            Square {
                freq: 150. * Hz,
                option: SquareOption {
                    duty,
                    ..Default::default()
                },
            }
            .calc(&V12_1.firmware_limits())
            .err()
        );
    }
}
