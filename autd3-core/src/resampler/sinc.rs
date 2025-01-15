use std::{f64::consts::PI, num::NonZeroUsize};

use crate::utils::float::is_integer;

use super::{window::InterpolationWindow, Blackman, Resampler};

/// Sinc interpolation resampler.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SincInterpolation<T: InterpolationWindow> {
    /// Window function.
    pub window: T,
}

impl Default for SincInterpolation<Blackman> {
    fn default() -> Self {
        Self {
            window: Blackman {
                size: NonZeroUsize::new(32).unwrap(),
            },
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
        let target_size = source_len as f64 * ratio;
        // GRCOV_EXCL_START
        if !is_integer(target_size) {
            tracing::warn!(
                "The target size ({}) is not an integer, ceiling to {}.",
                target_size,
                target_size.ceil()
            );
        }
        // GRCOV_EXCL_STOP
        (0..target_size.ceil() as usize)
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
        let target_size = source_len as f64 * ratio;
        // GRCOV_EXCL_START
        if !is_integer(target_size) {
            tracing::warn!(
                "The target size ({}) is not an integer, ceiling to {}.",
                target_size,
                target_size.ceil()
            );
        }
        // GRCOV_EXCL_STOP
        (0..target_size.ceil() as usize)
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
    use crate::resampler::{Blackman, Rectangular};

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(vec![127, 217, 255, 223, 127, 42, 0, 37], vec![127, 255, 127, 0], 2.0, Rectangular { size: NonZeroUsize::new(32).unwrap() })]
    #[case(vec![127, 217, 255, 217, 127, 37, 0, 37], vec![127, 255, 127, 0], 2.0, Rectangular { size: NonZeroUsize::new(4096).unwrap() })]
    #[case(vec![127, 130, 127, 130], vec![127, 127], 2.0, Rectangular { size: NonZeroUsize::new(32).unwrap() })]
    #[case(vec![127, 127, 127, 127], vec![127, 127], 2.0, Rectangular { size: NonZeroUsize::new(4096).unwrap() })]
    #[case(vec![127, 217, 255, 217, 127, 37, 0, 37], vec![127, 255, 127, 0], 2.0, Blackman { size: NonZeroUsize::new(32).unwrap() })]
    #[case(vec![127, 217, 255, 217, 127, 37, 0, 37], vec![127, 255, 127, 0], 2.0, Blackman { size: NonZeroUsize::new(4096).unwrap() })]
    #[case(vec![127, 126, 127, 126], vec![127, 127], 2.0, Blackman { size: NonZeroUsize::new(32).unwrap() })]
    #[case(vec![127, 127, 127, 127], vec![127, 127], 2.0, Blackman { size: NonZeroUsize::new(4096).unwrap() })]
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
    #[case(vec![124, 249, 124, 1], vec![127, 217, 255, 217, 127, 37, 0, 37], 0.5, Rectangular { size: NonZeroUsize::new(32).unwrap() })]
    #[case(vec![127, 255, 127, 0], vec![127, 217, 255, 217, 127, 37, 0, 37], 0.5, Rectangular { size: NonZeroUsize::new(4096).unwrap() })]
    #[case(vec![124, 124], vec![127, 127, 127, 127], 0.5, Rectangular { size: NonZeroUsize::new(32).unwrap() })]
    #[case(vec![127, 127], vec![127, 127, 127, 127], 0.5, Rectangular { size: NonZeroUsize::new(4096).unwrap() })]
    #[case(vec![127, 255, 127, 0], vec![127, 217, 255, 217, 127, 37, 0, 37], 0.5, Blackman { size: NonZeroUsize::new(32).unwrap() })]
    #[case(vec![127, 255, 127, 0], vec![127, 217, 255, 217, 127, 37, 0, 37], 0.5, Blackman { size: NonZeroUsize::new(4096).unwrap() })]
    #[case(vec![127, 127], vec![127, 127, 127, 127], 0.5, Blackman { size: NonZeroUsize::new(32).unwrap() })]
    #[case(vec![127, 127], vec![127, 127, 127, 127], 0.5, Blackman { size: NonZeroUsize::new(4096).unwrap() })]
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
