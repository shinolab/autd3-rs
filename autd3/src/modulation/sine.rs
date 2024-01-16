/*
 * File: sine.rs
 * Project: modulation
 * Created Date: 28/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3_driver::{common::EmitIntensity, defined::PI, derive::*};

use num::integer::gcd;

use super::sampling_mode::SamplingMode;

/// Sine wave modulation
#[derive(Modulation, Clone, Copy)]
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
    /// * `freq` - Frequency of the sine wave [Hz]
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
    pub fn with_intensity<A: Into<EmitIntensity>>(self, intensity: A) -> Self {
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
    pub fn with_offset<A: Into<EmitIntensity>>(self, offset: A) -> Self {
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
    /// * `mode` - Sampling mode
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
}

impl Modulation for Sine {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let (n, rep) = match self.mode {
            SamplingMode::ExactFrequency => {
                let sf = self.sampling_config().frequency() as usize;
                if self.freq.fract() != 0.0 {
                    return Err(AUTDInternalError::ModulationError(
                        "Frequency must be integer".to_string(),
                    ));
                }
                let freq = (self.freq as usize).clamp(1, sf / 2);
                let d = gcd(sf, freq);
                let n = sf / d;
                let rep = freq / d;
                (n, rep)
            }
            SamplingMode::SizeOptimized => {
                let sf = self.sampling_config().frequency();
                let freq = self.freq.clamp(0., sf / 2.);
                let n = (sf / freq).round() as usize;
                (n, 1)
            }
        };
        Ok((0..n)
            .map(|i| {
                EmitIntensity::new(
                    (((self.intensity / 2).value() as float
                        * (2.0 * PI * (rep * i) as float / n as float + self.phase.radian()).sin())
                    .round()
                        + self.offset.value() as float) as u8,
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine() {
        let expect = [
            127, 157, 185, 209, 230, 244, 252, 254, 248, 235, 217, 193, 166, 137, 107, 78, 52, 30,
            14, 4, 0, 4, 14, 30, 52, 78, 107, 137, 166, 193, 217, 235, 248, 254, 252, 244, 230,
            209, 185, 157, 127, 97, 69, 45, 24, 10, 2, 0, 6, 19, 37, 61, 88, 117, 147, 176, 202,
            224, 240, 250, 254, 250, 240, 224, 202, 176, 147, 117, 88, 61, 37, 19, 6, 0, 2, 10, 24,
            45, 69, 97,
        ];
        let m = Sine::new(150.);
        assert_eq!(m.sampling_config(), SamplingConfiguration::FREQ_4K_HZ);
        assert_eq!(expect.len(), m.calc().unwrap().len());
        expect
            .into_iter()
            .zip(m.calc().unwrap().iter())
            .for_each(|(e, a)| {
                assert_eq!(e, a.value());
            });
    }

    #[test]
    fn test_sine_clone() {
        let m = Sine::new(150.);
        let m2 = m.clone();
        assert_eq!(m.sampling_config(), m2.sampling_config());
        assert_eq!(m.freq(), m2.freq());
        assert_eq!(m.intensity(), m2.intensity());
        assert_eq!(m.offset(), m2.offset());
        assert_eq!(m.phase(), m2.phase());
    }

    #[test]
    fn test_sine_with_size_opt() {
        let expect = [
            127, 156, 184, 209, 229, 244, 252, 254, 249, 237, 219, 197, 170, 142, 112, 84, 57, 35,
            17, 5, 0, 2, 10, 25, 45, 70, 98,
        ];
        let m = Sine::new(150.).with_mode(SamplingMode::SizeOptimized);
        assert_eq!(m.sampling_config(), SamplingConfiguration::FREQ_4K_HZ);
        assert_eq!(expect.len(), m.calc().unwrap().len());
        expect
            .into_iter()
            .zip(m.calc().unwrap().iter())
            .for_each(|(e, a)| {
                assert_eq!(e, a.value());
            });
    }

    #[test]
    fn test_sine_new() {
        let m = Sine::new(100.);
        assert_eq!(m.freq(), 100.);
        assert_eq!(m.intensity(), EmitIntensity::MAX);
        assert_eq!(m.offset(), EmitIntensity::MAX / 2);
        assert_eq!(m.phase(), Phase::new(0));
        let vec = m.calc().unwrap();
        assert!(!vec.is_empty());
        assert!(vec
            .iter()
            .all(|&x| x >= m.offset - m.intensity / 2 && x <= m.offset + m.intensity / 2));

        let m = Sine::new(100.1);
        assert_eq!(
            m.calc(),
            Err(AUTDInternalError::ModulationError(
                "Frequency must be integer".to_string()
            ))
        );
        let m = Sine::new(100.1).with_mode(SamplingMode::SizeOptimized);
        assert!(m.calc().is_ok());
    }

    #[test]
    fn test_sine_with_intensity() {
        let m = Sine::new(100.).with_intensity(EmitIntensity::MAX / 2);
        assert_eq!(m.intensity, EmitIntensity::MAX / 2);

        let vec = m.calc().unwrap();
        assert!(!vec.is_empty());
        assert!(vec
            .iter()
            .all(|&x| x >= m.offset - m.intensity / 2 && x <= m.offset + m.intensity / 2));
    }

    #[test]
    fn test_sine_with_offset() {
        let m = Sine::new(100.)
            .with_offset(EmitIntensity::MAX / 4)
            .with_intensity(EmitIntensity::MAX / 2);
        assert_eq!(m.offset, EmitIntensity::MAX / 4);

        let vec = m.calc().unwrap();
        assert!(!vec.is_empty());
        assert!(vec
            .iter()
            .all(|&x| x >= m.offset - m.intensity / 2 && x <= m.offset + m.intensity / 2));
    }

    #[test]
    fn test_sine_with_phase() {
        let m = Sine::new(100.).with_phase(PI / 4.0 * Rad);
        assert_eq!(m.phase, PI / 4.0 * Rad);

        let vec = m.calc().unwrap();
        assert!(!vec.is_empty());
        assert!(vec
            .iter()
            .all(|&x| x >= m.offset - m.intensity / 2 && x <= m.offset + m.intensity / 2));
    }
}
