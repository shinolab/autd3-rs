use autd3_core::derive::*;

/// [`Modulation`] that applies FIR filter to the original [`Modulation`].
#[derive(Modulation, Debug)]
pub struct Fir<M: Modulation> {
    /// The target [`Modulation`] to apply FIR filter.
    pub target: M,
    /// The coefficients of the FIR filter.
    pub coef: Vec<f32>,
}

impl<M: Modulation> Modulation for Fir<M> {
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        let src = self.target.calc()?;
        let src_len = src.len() as isize;
        let filter_len = self.coef.len() as isize;
        Ok((0..src_len)
            .map(|i| {
                (0..filter_len)
                    .map(|j| {
                        src[(i + j - filter_len / 2).rem_euclid(src_len) as usize] as f32
                            * self.coef[j as usize]
                    })
                    .sum::<f32>() as u8
            })
            .collect())
    }

    fn sampling_config(&self) -> Result<SamplingConfig, ModulationError> {
        self.target.sampling_config()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        modulation::{Custom, Fourier},
        prelude::Sine,
    };
    use autd3_driver::defined::{kHz, Hz};

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::freq_4k(SamplingConfig::new_nearest(4. * kHz))]
    #[case::freq_8k(SamplingConfig::new_nearest(8. * kHz))]
    fn test_sampling_config(#[case] config: SamplingConfig) {
        assert_eq!(
            Ok(config),
            Fir {
                target: Custom {
                    buffer: [u8::MIN; 2].to_vec(),
                    sampling_config: config,
                    option: Default::default(),
                },
                coef: vec![1.0]
            }
            .sampling_config()
        );
    }

    #[test]
    fn test() -> anyhow::Result<()> {
        let coef = vec![
            0.,
            2.336_732_5E-6,
            8.982_681E-6,
            1.888_706_2E-5,
            3.030_097E-5,
            4.075_849E-5,
            4.708_182E-5,
            4.542_212E-5,
            3.134_882_4E-5,
            0.,
            -5.369_572_3E-5,
            -0.000_134_718_74,
            -0.000_247_578_05,
            -0.000_395_855_98,
            -0.000_581_690_7,
            -0.000_805_217_2,
            -0.001_063_996,
            -0.001_352_463_7,
            -0.001_661_447_3,
            -0.001_977_784_6,
            -0.002_284_095_4,
            -0.002_558_745,
            -0.002_776_031,
            -0.002_906_624_2,
            -0.002_918_272_5,
            -0.002_776_767_4,
            -0.002_447_156_7,
            -0.001_895_169_7,
            -0.001_088_802_4,
            0.,
            0.001_393_638_8,
            0.003_107_224_6,
            0.005_147_092_5,
            0.007_509_561,
            0.010_180_013,
            0.013_132_379,
            0.016_329_063,
            0.019_721_36,
            0.023_250_382,
            0.026_848_452,
            0.030_440_966,
            0.033_948_626,
            0.037_290_003,
            0.040_384_263,
            0.043_154_005,
            0.045_528_06,
            0.047_444_11,
            0.048_851_013,
            0.049_710_777,
            0.05,
            0.049_710_777,
            0.048_851_013,
            0.047_444_11,
            0.045_528_06,
            0.043_154_005,
            0.040_384_263,
            0.037_290_003,
            0.033_948_626,
            0.030_440_966,
            0.026_848_452,
            0.023_250_382,
            0.019_721_36,
            0.016_329_063,
            0.013_132_379,
            0.010_180_013,
            0.007_509_561,
            0.005_147_092_5,
            0.003_107_224_6,
            0.001_393_638_8,
            0.,
            -0.001_088_802_4,
            -0.001_895_169_7,
            -0.002_447_156_7,
            -0.002_776_767_4,
            -0.002_918_272_5,
            -0.002_906_624_2,
            -0.002_776_031,
            -0.002_558_745,
            -0.002_284_095_4,
            -0.001_977_784_6,
            -0.001_661_447_3,
            -0.001_352_463_7,
            -0.001_063_996,
            -0.000_805_217_2,
            -0.000_581_690_7,
            -0.000_395_855_98,
            -0.000_247_578_05,
            -0.000_134_718_74,
            -5.369_572_3E-5,
            0.,
            3.134_882_4E-5,
            4.542_212E-5,
            4.708_182E-5,
            4.075_849E-5,
            3.030_097E-5,
            1.888_706_2E-5,
            8.982_681E-6,
            2.336_732_5E-6,
            0.,
        ];

        assert_eq!(
            vec![
                127, 131, 136, 140, 145, 149, 153, 157, 161, 164, 168, 171, 173, 176, 178, 180,
                182, 183, 184, 184, 184, 184, 184, 183, 182, 180, 178, 176, 173, 171, 168, 164,
                161, 157, 153, 149, 145, 140, 136, 131, 127, 122, 118, 113, 109, 105, 100, 96, 93,
                89, 86, 83, 80, 77, 75, 73, 72, 70, 70, 69, 69, 69, 70, 70, 72, 73, 75, 77, 80, 83,
                86, 89, 93, 96, 100, 105, 109, 113, 118, 122
            ],
            *Fir {
                target: Fourier {
                    components: vec![
                        Sine {
                            freq: 50 * Hz,
                            option: Default::default(),
                        },
                        Sine {
                            freq: 1000 * Hz,
                            option: Default::default(),
                        }
                    ],
                    option: Default::default()
                },
                coef
            }
            .calc()?
        );

        Ok(())
    }
}
