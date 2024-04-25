use autd3_driver::{defined::PI, derive::*};

use num::integer::gcd;

use super::sampling_mode::SamplingMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactFrequency;
impl SamplingMode for ExactFrequency {
    type F = usize;
    type D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration);
    fn calc(freq: Self::F, data: Self::D) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let (intensity, phase, offset, sampling_config) = data;
        if sampling_config.frequency().fract() != 0.0 {
            return Err(AUTDInternalError::ModulationError(
                "Sampling frequency must be integer".to_string(),
            ));
        }
        let sf = sampling_config.frequency() as usize;
        let freq = freq.clamp(1, sf / 2);
        let k = gcd(sf, freq);
        let n = sf / k;
        let rep = freq / k;
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
pub struct SizeOptimized;
impl SamplingMode for SizeOptimized {
    type F = f64;
    type D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration);
    fn calc(freq: Self::F, data: Self::D) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let (intensity, phase, offset, sampling_config) = data;
        let sf = sampling_config.frequency();
        let freq = freq.clamp(0., sf / 2.);
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

pub trait FrequencyType: Copy {
    type S: SamplingMode<F = Self, D = (EmitIntensity, Phase, EmitIntensity, SamplingConfiguration)>;
}
impl FrequencyType for usize {
    type S = ExactFrequency;
}
impl FrequencyType for f64 {
    type S = SizeOptimized;
}

/// Sine wave modulation
#[derive(Modulation, Clone, PartialEq, Debug, Builder)]
pub struct Sine<F: FrequencyType> {
    #[get]
    freq: F,
    #[getset]
    intensity: EmitIntensity,
    #[getset]
    phase: Phase,
    #[getset]
    offset: EmitIntensity,
    config: SamplingConfiguration,
    loop_behavior: LoopBehavior,
}

impl<F: FrequencyType> Sine<F> {
    pub fn new(freq: F) -> Sine<F> {
        Sine {
            freq,
            intensity: EmitIntensity::MAX,
            phase: Phase::new(0),
            offset: EmitIntensity::new(127),
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::Infinite,
        }
    }
}

impl<F: FrequencyType> Modulation for Sine<F> {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        F::S::calc(
            self.freq,
            (self.intensity, self.phase, self.offset, self.config),
        )
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
        let m = Sine::new(150);
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
    fn test_sine_new() {
        let m = Sine::new(100);
        assert_eq!(100, m.freq());
        assert_eq!(EmitIntensity::MAX, m.intensity());
        assert_eq!(EmitIntensity::MAX / 2, m.offset());
        assert_eq!(Phase::new(0), m.phase());

        assert_eq!(
            Err(AUTDInternalError::ModulationError(
                "Sampling frequency must be integer".to_string()
            )),
            Sine::new(100)
                .with_sampling_config(SamplingConfiguration::from_frequency(10.1).unwrap())
                .calc()
        );
    }

    #[test]
    fn test_sine_with_param() {
        let m = Sine::new(100)
            .with_intensity(EmitIntensity::MAX / 2)
            .with_offset(EmitIntensity::MAX / 4)
            .with_phase(PI / 4.0 * Rad);
        assert_eq!(EmitIntensity::MAX / 2, m.intensity);
        assert_eq!(EmitIntensity::MAX / 4, m.offset);
        assert_eq!(PI / 4.0 * Rad, m.phase);
    }

    #[test]
    fn test_sine_derive() {
        let m = Sine::new(150);
        assert_eq!(m, m.clone());
    }
}
