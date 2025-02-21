use std::fmt::Debug;

use crate::modulation::sine::SineOption;

use super::{sampling_mode::SamplingMode, sine::Sine};

use autd3_core::derive::*;

use derive_more::Deref;
use num::integer::lcm;

/// The option of [`Fourier`].
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct FourierOption {
    /// The scaling factor of the modulation. If `None`, the scaling factor is set to reciprocal of the number of components. The default value is `None`.
    pub scale_factor: Option<f32>,
    /// If `true`, the modulation value is clamped to the range of `u8`. If `false`, returns an error if the value is out of range. The default value is `false`.
    pub clamp: bool,
    /// The offset of the modulation value. The default value is `0`.
    pub offset: u8,
}

/// `Moudlation` that is a sum of multiple [`Sine`].
///
/// The modulation value is calculated as `⌊offset + scale_factor * (sum of components)⌋`, where `offset` and `scale_factor` can be set by the [`FourierOption`].
#[derive(Modulation, Clone, PartialEq, Debug, Deref)]
pub struct Fourier<S: Into<SamplingMode> + Clone + Debug> {
    #[deref]
    /// The [`Sine`] components of the Fourier modulation.
    pub components: Vec<Sine<S>>,
    /// The option of the modulation.
    pub option: FourierOption,
}

impl<S: Into<SamplingMode> + Clone + Debug> Fourier<S> {
    /// Create a new [`Fourier`].
    #[must_use]
    pub const fn new(components: Vec<Sine<S>>, option: FourierOption) -> Self {
        Self { components, option }
    }
}

impl<S: Into<SamplingMode> + Clone + Debug> Modulation for Fourier<S> {
    fn sampling_config(&self) -> SamplingConfig {
        self.components
            .first()
            .map(|m| m.sampling_config())
            .unwrap_or(SamplingConfig::FREQ_40K)
    }

    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        if self.components.is_empty() {
            return Err(ModulationError::new("Components must not be empty"));
        }
        let sampling_config = self.sampling_config();
        let components = self
            .components
            .into_iter()
            .map(|s| Sine {
                freq: s.freq,
                option: SineOption {
                    clamp: false,
                    ..s.option
                },
            })
            .collect::<Vec<_>>();
        tracing::trace!("Fourier components: {:?}", components);
        if components
            .iter()
            .skip(1)
            .any(|c| c.sampling_config() != sampling_config)
        {
            return Err(ModulationError::new(
                "All components must have the same sampling configuration",
            ));
        }

        let buffers = components
            .iter()
            .map(|c| Ok(c.calc_raw()?.collect::<Vec<_>>()))
            .collect::<Result<Vec<_>, ModulationError>>()?;
        let scale = self
            .option
            .scale_factor
            .unwrap_or(1. / buffers.len() as f32);
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
            .map(|x| (x * scale + self.option.offset as f32).floor() as isize)
            .map(|v| {
                if (u8::MIN as _..=u8::MAX as _).contains(&v) {
                    Ok(v as _)
                } else if self.option.clamp {
                    Ok(v.clamp(u8::MIN as _, u8::MAX as _) as _)
                } else {
                    Err(ModulationError::new(format!(
                        "Fourier modulation value ({}) is out of range [{}, {}]",
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
    use super::*;

    use autd3_core::defined::Freq;
    use autd3_driver::defined::{Hz, PI, rad};

    #[test]
    fn test_fourier() -> anyhow::Result<()> {
        let f0 = Sine {
            freq: 50. * Hz,
            option: SineOption {
                phase: PI / 2.0 * rad,
                ..SineOption::default()
            },
        };
        let f1 = Sine {
            freq: 100. * Hz,
            option: SineOption {
                phase: PI / 3.0 * rad,
                ..SineOption::default()
            },
        };
        let f2 = Sine {
            freq: 150. * Hz,
            option: SineOption {
                phase: PI / 4.0 * rad,
                ..SineOption::default()
            },
        };
        let f3 = Sine {
            freq: 200. * Hz,
            option: SineOption::default(),
        };
        let f4 = Sine {
            freq: 250. * Hz,
            option: SineOption::default(),
        };

        let f0_buf = f0.calc_raw()?.collect::<Vec<_>>();
        let f1_buf = f1.calc_raw()?.collect::<Vec<_>>();
        let f2_buf = f2.calc_raw()?.collect::<Vec<_>>();
        let f3_buf = f3.calc_raw()?.collect::<Vec<_>>();
        let f4_buf = f4.calc_raw()?.collect::<Vec<_>>();

        let f = Fourier {
            components: vec![f0, f1, f2, f3, f4],
            option: FourierOption::default(),
        };

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
            Err(ModulationError::new(
                "All components must have the same sampling configuration"
            )),
            Fourier {
                components: vec![
                    Sine {
                        freq: 50. * Hz,
                        option: SineOption {
                            sampling_config: SamplingConfig::FREQ_4K,
                            ..Default::default()
                        }
                    },
                    Sine {
                        freq: 50. * Hz,
                        option: SineOption {
                            sampling_config: SamplingConfig::FREQ_40K,
                            ..Default::default()
                        }
                    },
                ],
                option: FourierOption::default(),
            }
            .calc()
        );
        Ok(())
    }

    #[test]
    fn empty_components() {
        assert_eq!(
            Err(ModulationError::new("Components must not be empty")),
            Fourier::<Freq<u32>> {
                components: vec![],
                option: FourierOption::default(),
            }
            .calc()
        );
    }

    #[rstest::rstest]
    #[case(
        Err(ModulationError::new("Fourier modulation value (-1) is out of range [0, 255]")),
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
        Err(ModulationError::new("Fourier modulation value (510) is out of range [0, 255]")),
        0xFF,
        false,
        Some(2.)
    )]
    #[test]
    fn out_of_range(
        #[case] expect: Result<Vec<u8>, ModulationError>,
        #[case] offset: u8,
        #[case] clamp: bool,
        #[case] scale_factor: Option<f32>,
    ) {
        assert_eq!(
            expect,
            Fourier {
                components: vec![Sine {
                    freq: 200 * Hz,
                    option: SineOption {
                        offset,
                        ..Default::default()
                    },
                }],
                option: FourierOption {
                    clamp,
                    scale_factor,
                    ..Default::default()
                },
            }
            .calc()
        );
    }
}
