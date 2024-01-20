/*
 * File: wav.rs
 * Project: src
 * Created Date: 15/06/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 20/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3_driver::{common::EmitIntensity, derive::*};
use hound::{SampleFormat, WavSpec};

use std::path::Path;

use crate::error::AudioFileError;

/// Modulation constructed from wav file
///
/// The wav data is resampled to the sampling frequency of Modulation.
#[derive(Modulation, Clone)]
pub struct Wav {
    channels: u16,
    sample_rate: u32,
    raw_buffer: Vec<f32>,
    config: SamplingConfiguration,
}

impl Wav {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the wav file
    ///
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, AudioFileError> {
        let mut reader = hound::WavReader::open(path)?;
        let WavSpec {
            channels,
            sample_format,
            sample_rate,
            bits_per_sample,
        } = reader.spec();
        let raw_buffer = reader.samples::<i32>().collect::<Result<Vec<_>, _>>()?;
        let raw_buffer = match (sample_format, bits_per_sample) {
            (SampleFormat::Int, 8) => raw_buffer
                .iter()
                .map(|i| (i - std::i8::MIN as i32) as f32 / 255.)
                .collect(),
            (SampleFormat::Int, 16) => raw_buffer
                .iter()
                .map(|i| (i - std::i16::MIN as i32) as f32 / 65535.)
                .collect(),
            (SampleFormat::Int, 24) => raw_buffer
                .iter()
                .map(|i| (i - 8388608i32) as f32 / 16777215.)
                .collect(),
            (SampleFormat::Int, 32) => raw_buffer
                .iter()
                .map(|&i| (i as i64 - std::i32::MIN as i64) as f32 / 4294967295.)
                .collect(),
            _ => return Err(AudioFileError::Wav(hound::Error::Unsupported)),
        };

        Ok(Self {
            channels,
            sample_rate,
            raw_buffer,
            config: SamplingConfiguration::FREQ_4K_HZ,
        })
    }
}

impl Modulation for Wav {
    #[allow(clippy::unnecessary_cast)]
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        Ok(wav_io::resample::linear(
            self.raw_buffer.clone(),
            self.channels,
            self.sample_rate,
            self.sampling_config().frequency() as u32,
        )
        .iter()
        .map(|&d| EmitIntensity::new((d * 255.).round() as u8))
        .collect())
    }
}
