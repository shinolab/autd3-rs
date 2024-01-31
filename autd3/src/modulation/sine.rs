use autd3_driver::{common::EmitIntensity, defined::PI, derive::*};

use num::integer::gcd;

use super::sampling_mode::SamplingMode;

/// Sine wave modulation
#[derive(Modulation, Clone, Copy, PartialEq, Debug)]
pub struct Sine {
    freq: float,
    intensity: EmitIntensity,
    phase: Phase,
    offset: EmitIntensity,
    config: SamplingConfiguration,
    mode: SamplingMode,
}

impl Sine {
    /// constructor
    ///
    /// The sine wave is defined as `intensity / 2 * sin(2π * freq * t + phase) + offset`, where `t` is time, and `intensity = EmitIntensity::MAX`, `phase = 0`, `offset = EmitIntensity::MAX/2` by default.
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of the sine wave \[Hz\]
    ///
    pub const fn new(freq: float) -> Self {
        Self {
            freq,
            intensity: EmitIntensity::MAX,
            phase: Phase::new(0),
            offset: EmitIntensity::new(127),
            config: SamplingConfiguration::FREQ_4K_HZ,
            mode: SamplingMode::ExactFrequency,
        }
    }

    /// set intensity
    ///
    /// # Arguments
    ///
    /// * `intensity` - peek to peek intensity
    ///
    pub fn with_intensity(self, intensity: impl Into<EmitIntensity>) -> Self {
        Self {
            intensity: intensity.into(),
            ..self
        }
    }

    /// set offset
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset of the wave
    ///
    pub fn with_offset(self, offset: impl Into<EmitIntensity>) -> Self {
        Self {
            offset: offset.into(),
            ..self
        }
    }

    /// set phase
    ///
    /// # Arguments
    ///
    /// * `phase` - Phase of the wave
    ///
    pub const fn with_phase(self, phase: Phase) -> Self {
        Self { phase, ..self }
    }

    /// set sampling mode
    ///
    /// # Arguments
    ///
    /// * `mode` - [SamplingMode]
    ///
    pub const fn with_mode(self, mode: SamplingMode) -> Self {
        Self { mode, ..self }
    }

    pub const fn freq(&self) -> float {
        self.freq
    }

    pub const fn intensity(&self) -> EmitIntensity {
        self.intensity
    }

    pub const fn offset(&self) -> EmitIntensity {
        self.offset
    }

    pub const fn phase(&self) -> Phase {
        self.phase
    }

    pub const fn mode(&self) -> SamplingMode {
        self.mode
    }
}

impl Modulation for Sine {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let (n, rep) = match self.mode {
            SamplingMode::ExactFrequency => {
                if self.sampling_config().frequency().fract() != 0.0 {
                    return Err(AUTDInternalError::ModulationError(
                        "Sampling frequency must be integer".to_string(),
                    ));
                }
                if self.freq.fract() != 0.0 {
                    return Err(AUTDInternalError::ModulationError(
                        "Frequency must be integer".to_string(),
                    ));
                }
                let sf = self.sampling_config().frequency() as usize;
                let freq = (self.freq as usize).clamp(1, sf / 2);
                let k = gcd(sf, freq);
                (sf / k, freq / k)
            }
            SamplingMode::SizeOptimized => {
                let sf = self.sampling_config().frequency();
                let freq = self.freq.clamp(0., sf / 2.);
                ((sf / freq).round() as usize, 1)
            }
        };
        let intensity = self.intensity.value() as float;
        let phase = self.phase.radian();
        let offset = self.offset.value() as float;
        Ok((0..n)
            .map(|i| {
                (((intensity / 2. * (2.0 * PI * (rep * i) as float / n as float + phase).sin())
                    + offset)
                    .round() as u8)
                    .into()
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine() -> anyhow::Result<()> {
        let expect = [
            127, 157, 185, 210, 230, 245, 253, 254, 248, 236, 217, 194, 166, 137, 107, 78, 52, 30,
            13, 3, 0, 3, 13, 30, 52, 78, 107, 137, 166, 194, 217, 236, 248, 254, 253, 245, 230,
            210, 185, 157, 127, 97, 69, 44, 24, 9, 1, 0, 6, 18, 37, 60, 88, 117, 147, 176, 202,
            224, 241, 251, 255, 251, 241, 224, 202, 176, 147, 117, 88, 60, 37, 18, 6, 0, 1, 9, 24,
            44, 69, 97,
        ];
        let m = Sine::new(150.);
        assert_eq!(SamplingConfiguration::FREQ_4K_HZ, m.sampling_config());
        assert_eq!(
            expect
                .into_iter()
                .map(EmitIntensity::new)
                .collect::<Vec<_>>(),
            m.calc()?
        );

        Ok(())
    }

    #[test]
    fn test_sine_with_size_opt() -> anyhow::Result<()> {
        let expect = [
            127, 156, 184, 209, 229, 244, 253, 254, 249, 237, 220, 197, 171, 142, 112, 83, 57, 34,
            17, 5, 0, 1, 10, 25, 45, 70, 98,
        ];

        let m = Sine::new(150.).with_mode(SamplingMode::SizeOptimized);
        assert_eq!(SamplingConfiguration::FREQ_4K_HZ, m.sampling_config());
        assert_eq!(
            expect
                .into_iter()
                .map(EmitIntensity::new)
                .collect::<Vec<_>>(),
            m.calc()?
        );

        Ok(())
    }

    #[test]
    fn test_sine_new() {
        let m = Sine::new(100.);
        assert_eq!(100., m.freq());
        assert_eq!(EmitIntensity::MAX, m.intensity());
        assert_eq!(EmitIntensity::MAX / 2, m.offset());
        assert_eq!(Phase::new(0), m.phase());
        assert_eq!(SamplingMode::ExactFrequency, m.mode());

        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "Frequency must be integer".to_string()
            )),
            Sine::new(100.1).calc()
        );

        assert!(Sine::new(100.1)
            .with_mode(SamplingMode::SizeOptimized)
            .calc()
            .is_ok());
    }

    #[test]
    fn test_sine_with_param() {
        let m = Sine::new(100.)
            .with_intensity(EmitIntensity::MAX / 2)
            .with_offset(EmitIntensity::MAX / 4)
            .with_phase(PI / 4.0 * Rad);
        assert_eq!(EmitIntensity::MAX / 2, m.intensity);
        assert_eq!(EmitIntensity::MAX / 4, m.offset);
        assert_eq!(PI / 4.0 * Rad, m.phase);
    }

    #[test]
    fn test_sine_derive() {
        let m = Sine::new(150.);
        assert_eq!(m, m.clone());
    }
}
