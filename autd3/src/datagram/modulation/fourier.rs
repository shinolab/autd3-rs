use super::{sampling_mode::SamplingMode, sine::Sine};

use autd3_driver::derive::*;

use derive_more::Deref;
use num::integer::lcm;

/// `Moudlation` that is a sum of multiple [`Sine`].
///
/// The modulation value is calculated as `⌊offset + scale_factor * (sum of components)⌋`.
///
/// [`Sine`]: crate::modulation::Sine
#[derive(Modulation, Clone, PartialEq, Debug, Deref, Builder)]
pub struct Fourier<S: SamplingMode> {
    #[deref]
    components: Vec<Sine<S>>,
    #[no_change]
    config: SamplingConfig,
    #[get]
    #[set]
    /// The scaling factor of the modulation. If `None`, the scaling factor is set to reciprocal of the number of components. The default value is `None`.
    scale_factor: Option<f32>,
    #[get]
    #[set]
    /// If `true`, the modulation value is clamped to the range of `u8`. If `false`, returns an error if the value is out of range. The default value is `false`.
    clamp: bool,
    #[get]
    #[set]
    /// The offset of the modulation value. The default value is `0`.
    offset: u8,
    loop_behavior: LoopBehavior,
}

impl<S: SamplingMode> Fourier<S> {
    /// Create a new [`Fourier`] modulation.
    pub fn new(componens: impl IntoIterator<Item = Sine<S>>) -> Result<Self, AUTDDriverError> {
        let components = componens
            .into_iter()
            .map(|s| s.with_clamp(false))
            .collect::<Vec<_>>();
        tracing::trace!("Fourier components: {:?}", components);
        let config = components
            .first()
            .ok_or(AUTDDriverError::ModulationError(
                "Components must not be empty".to_string(),
            ))?
            .sampling_config();
        if components
            .iter()
            .skip(1)
            .any(|c| c.sampling_config() != config)
        {
            return Err(AUTDDriverError::ModulationError(
                "All components must have the same sampling configuration".to_string(),
            ));
        }
        Ok(Self {
            config,
            components,
            scale_factor: None,
            clamp: false,
            offset: 0,
            loop_behavior: LoopBehavior::infinite(),
        })
    }
}

impl<S: SamplingMode> Modulation for Fourier<S> {
    fn calc(self) -> Result<Vec<u8>, AUTDDriverError> {
        let buffers = self
            .components
            .iter()
            .map(|c| Ok(c.calc_raw()?.collect::<Vec<_>>()))
            .collect::<Result<Vec<_>, AUTDDriverError>>()?;
        let scale = self.scale_factor.unwrap_or(1. / buffers.len() as f32);
        let res = vec![0f32; buffers.iter().fold(1, |acc, x| lcm(acc, x.len()))];
        buffers
            .into_iter()
            .fold(res, |mut acc, x| {
                acc.iter_mut()
                    .zip(x.iter().cycle())
                    .for_each(|(a, &b)| *a += b);
                acc
            })
            .into_iter()
            .map(|x| (x * scale + self.offset as f32).floor() as isize)
            .map(|v| {
                if (u8::MIN as _..=u8::MAX as _).contains(&v) {
                    Ok(v as _)
                } else if self.clamp {
                    Ok(v.clamp(u8::MIN as _, u8::MAX as _) as _)
                } else {
                    Err(AUTDDriverError::ModulationError(format!(
                        "Fourier modulation value ({}) is out of range [{}, {}]",
                        v,
                        u8::MIN,
                        u8::MAX,
                    )))?
                }
            })
            .collect::<Result<Vec<_>, AUTDDriverError>>()
    }
}

#[cfg(test)]
mod tests {
    use crate::modulation::sampling_mode::ExactFreq;

    use super::*;

    use autd3_driver::defined::{rad, Hz, PI};

    #[test]
    fn test_fourier() -> anyhow::Result<()> {
        let f0 = Sine::new(50. * Hz).with_phase(PI / 2.0 * rad);
        let f1 = Sine::new(100. * Hz).with_phase(PI / 3.0 * rad);
        let f2 = Sine::new(150. * Hz).with_phase(PI / 4.0 * rad);
        let f3 = Sine::new(200. * Hz);
        let f4 = Sine::new(250. * Hz);

        let f0_buf = f0.calc_raw()?.collect::<Vec<_>>();
        let f1_buf = f1.calc_raw()?.collect::<Vec<_>>();
        let f2_buf = f2.calc_raw()?.collect::<Vec<_>>();
        let f3_buf = f3.calc_raw()?.collect::<Vec<_>>();
        let f4_buf = f4.calc_raw()?.collect::<Vec<_>>();

        let f = Fourier::new([f0, f1, f2, f3, f4])?;

        assert_eq!(f.sampling_config(), SamplingConfig::FREQ_4K);
        assert_eq!(f[0].freq(), 50. * Hz);
        assert_eq!(f[0].phase(), PI / 2.0 * rad);
        assert_eq!(f[1].freq(), 100. * Hz);
        assert_eq!(f[1].phase(), PI / 3.0 * rad);
        assert_eq!(f[2].freq(), 150. * Hz);
        assert_eq!(f[2].phase(), PI / 4.0 * rad);
        assert_eq!(f[3].freq(), 200. * Hz);
        assert_eq!(f[3].phase(), 0.0 * rad);
        assert_eq!(f[4].freq(), 250. * Hz);
        assert_eq!(f[4].phase(), 0.0 * rad);

        let buf = &f.calc()?;

        (0..buf.len()).for_each(|i| {
            assert_eq!(
                buf[i],
                ((f0_buf[i % f0_buf.len()]
                    + f1_buf[i % f1_buf.len()]
                    + f2_buf[i % f2_buf.len()]
                    + f3_buf[i % f3_buf.len()]
                    + f4_buf[i % f4_buf.len()])
                    / 5.)
                    .floor() as u8
            );
        });

        Ok(())
    }

    #[test]
    fn mismatch_sampling_config() -> anyhow::Result<()> {
        assert_eq!(
            Err(AUTDDriverError::ModulationError(
                "All components must have the same sampling configuration".to_string()
            )),
            Fourier::new([
                Sine::new(50. * Hz),
                Sine::new(50. * Hz).with_sampling_config(SamplingConfig::FREQ_40K)?,
            ])
        );
        Ok(())
    }

    #[test]
    fn empty_components() {
        assert_eq!(
            Err(AUTDDriverError::ModulationError(
                "Components must not be empty".to_string()
            )),
            Fourier::<ExactFreq>::new(vec![])
        );
    }

    #[rstest::rstest]
    #[case(
        Err(AUTDDriverError::ModulationError("Fourier modulation value (-1) is out of range [0, 255]".to_owned())),
        0x00,
        false,
        None
    )]
    #[case(
        Ok(vec![0, 39, 74, 103, 121, 127, 121, 103, 74, 39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        0x00,
        true,
        None
    )]
    #[case(
        Err(AUTDDriverError::ModulationError("Fourier modulation value (510) is out of range [0, 255]".to_owned())),
        0xFF,
        false,
        Some(2.)
    )]
    #[test]
    fn out_of_range(
        #[case] expect: Result<Vec<u8>, AUTDDriverError>,
        #[case] offset: u8,
        #[case] clamp: bool,
        #[case] scale: Option<f32>,
    ) {
        assert_eq!(
            expect,
            Fourier::new([Sine::new(200 * Hz).with_offset(offset)])
                .unwrap()
                .with_clamp(clamp)
                .with_scale_factor(scale)
                .calc()
        );
    }
}
