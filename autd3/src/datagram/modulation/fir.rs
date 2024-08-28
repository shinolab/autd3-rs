use std::sync::Arc;

use crate::derive::*;

#[derive(Modulation, Debug)]
pub struct Fir<M: Modulation> {
    m: M,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    filter: Vec<f32>,
}

impl<M: Modulation> Fir<M> {
    fn new(m: M, filter: impl IntoIterator<Item = f32>) -> Self {
        Self {
            config: m.sampling_config(),
            loop_behavior: m.loop_behavior(),
            m,
            filter: filter.into_iter().collect(),
        }
    }
}

pub trait IntoFir<M: Modulation> {
    fn with_fir(self, filter: impl IntoIterator<Item = f32>) -> Fir<M>;
}

impl<M: Modulation> IntoFir<M> for M {
    fn with_fir(self, filter: impl IntoIterator<Item = f32>) -> Fir<M> {
        Fir::new(self, filter)
    }
}

impl<M: Modulation> Modulation for Fir<M> {
    fn calc(&self) -> Result<Arc<Vec<u8>>, AUTDInternalError> {
        let src = self.m.calc()?;
        let src_len = src.len() as isize;
        let filter_len = self.filter.len() as isize;
        Ok(Arc::new(
            (0..src_len)
                .map(|i| {
                    (0..filter_len)
                        .map(|j| {
                            src[(i + j - filter_len / 2).rem_euclid(src_len) as usize] as f32
                                * self.filter[j as usize]
                        })
                        .sum::<f32>() as u8
                })
                .collect(),
        ))
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
            config,
            Custom::new(&[u8::MIN; 2], config)
                .unwrap()
                .with_fir([1.0])
                .sampling_config()
        );
    }

    #[test]
    fn test() -> anyhow::Result<()> {
        let lpf = [
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
                126, 131, 135, 140, 144, 148, 152, 156, 160, 164, 167, 170, 173, 175, 178, 180,
                181, 182, 183, 184, 184, 184, 183, 182, 181, 180, 178, 175, 173, 170, 167, 164,
                160, 156, 152, 148, 144, 140, 135, 131, 126, 122, 117, 113, 108, 104, 100, 96, 92,
                88, 85, 82, 79, 77, 74, 73, 71, 70, 69, 68, 68, 68, 69, 70, 71, 73, 74, 77, 79, 82,
                85, 88, 92, 96, 100, 104, 108, 113, 117, 122
            ],
            *Fourier::new([Sine::new(50 * Hz), Sine::new(1000 * Hz)])?
                .with_fir(lpf)
                .calc()?
        );

        Ok(())
    }
}
