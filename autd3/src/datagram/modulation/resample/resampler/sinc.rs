use std::f64::consts::PI;

use super::{window::InterpolationWindow, Blackman, Resampler};

pub struct SincInterpolation<T: InterpolationWindow> {
    pub window: T,
}

impl Default for SincInterpolation<Blackman> {
    fn default() -> Self {
        Self {
            window: Blackman { size: 32 },
        }
    }
}

#[inline]
fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        1.0
    } else {
        (x * PI).sin() / (x * PI)
    }
}

#[inline]
fn modf(lhs: f64) -> (isize, f64) {
    let int = lhs.floor() as isize;
    let frac = lhs - int as f64;
    (int, frac)
}

impl<T: InterpolationWindow> Resampler for SincInterpolation<T> {
    fn upsample(&self, buffer: &[u8], ratio: f64) -> Vec<u8> {
        let source_len = buffer.len();
        let window_size = self.window.window_size();
        (0..(source_len as f64 * ratio).ceil() as usize)
            .map(|m| {
                let (n, frac) = modf(m as f64 / ratio);
                (0..window_size)
                    .map(|k| {
                        let kk = k as isize - window_size as isize / 2;
                        let idx = ((n + kk).rem_euclid(source_len as isize)) as usize;
                        buffer[idx] as f64 * sinc(kk as f64 - frac) * self.window.value(k)
                    })
                    .sum::<f64>()
                    .round() as u8
            })
            .collect()
    }

    fn downsample(&self, buffer: &[u8], ratio: f64) -> Vec<u8> {
        let source_len = buffer.len();
        let window_size = self.window.window_size();
        (0..(source_len as f64 * ratio).ceil() as usize)
            .map(|m| {
                let (n, frac) = modf(m as f64 / ratio);
                (0..window_size)
                    .map(|k| {
                        let kk = k as isize - window_size as isize / 2;
                        let idx = ((n + kk).rem_euclid(source_len as isize)) as usize;
                        ratio
                            * (buffer[idx] as f64
                                * sinc((kk as f64 - frac) * ratio)
                                * self.window.value(k))
                    })
                    .sum::<f64>()
                    .round() as u8
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::modulation::resample::{Blackman, Rectangular};

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(vec![127, 217, 255, 223, 127, 42, 0, 37], vec![127, 255, 127, 0], 2.0, Rectangular { size: 32 })]
    #[case(vec![127, 217, 255, 217, 127, 37, 0, 37], vec![127, 255, 127, 0], 2.0, Rectangular { size: 4096 })]
    #[case(vec![127, 130, 127, 130], vec![127, 127], 2.0, Rectangular { size: 32 })]
    #[case(vec![127, 127, 127, 127], vec![127, 127], 2.0, Rectangular { size: 4096 })]
    #[case(vec![127, 217, 255, 217, 127, 37, 0, 37], vec![127, 255, 127, 0], 2.0, Blackman { size: 32 })]
    #[case(vec![127, 217, 255, 217, 127, 37, 0, 37], vec![127, 255, 127, 0], 2.0, Blackman { size: 4096 })]
    #[case(vec![127, 126, 127, 126], vec![127, 127], 2.0, Blackman { size: 32 })]
    #[case(vec![127, 127, 127, 127], vec![127, 127], 2.0, Blackman { size: 4096 })]
    fn upsample(
        #[case] expect: Vec<u8>,
        #[case] buffer: Vec<u8>,
        #[case] ratio: f64,
        #[case] window: impl InterpolationWindow,
    ) {
        let resampler = SincInterpolation { window };
        assert_eq!(expect, resampler.upsample(&buffer, ratio));
    }

    #[rstest::rstest]
    #[test]
    #[case(vec![124, 249, 124, 1], vec![127, 217, 255, 217, 127, 37, 0, 37], 0.5, Rectangular { size: 32 })]
    #[case(vec![127, 255, 127, 0], vec![127, 217, 255, 217, 127, 37, 0, 37], 0.5, Rectangular { size: 4096 })]
    #[case(vec![124, 124], vec![127, 127, 127, 127], 0.5, Rectangular { size: 32 })]
    #[case(vec![127, 127], vec![127, 127, 127, 127], 0.5, Rectangular { size: 4096 })]
    #[case(vec![127, 255, 127, 0], vec![127, 217, 255, 217, 127, 37, 0, 37], 0.5, Blackman { size: 32 })]
    #[case(vec![127, 255, 127, 0], vec![127, 217, 255, 217, 127, 37, 0, 37], 0.5, Blackman { size: 4096 })]
    #[case(vec![127, 127], vec![127, 127, 127, 127], 0.5, Blackman { size: 32 })]
    #[case(vec![127, 127], vec![127, 127, 127, 127], 0.5, Blackman { size: 4096 })]
    fn downsample(
        #[case] expect: Vec<u8>,
        #[case] buffer: Vec<u8>,
        #[case] ratio: f64,
        #[case] window: impl InterpolationWindow,
    ) {
        let resampler = SincInterpolation { window };
        assert_eq!(expect, resampler.downsample(&buffer, ratio));
    }
}
