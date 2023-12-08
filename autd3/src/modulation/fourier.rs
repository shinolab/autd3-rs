/*
 * File: fourier.rs
 * Project: modulation
 * Created Date: 28/07/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 02/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use std::ops::{Deref, DerefMut};

use super::sine::Sine;

use autd3_derive::Modulation;
use autd3_driver::{common::EmitIntensity, derive::prelude::*};

use num::integer::lcm;

/// Multi-frequency sine wave modulation
#[derive(Modulation, Clone)]
pub struct Fourier {
    #[no_change]
    config: SamplingConfiguration,
    components: Vec<Sine>,
}

impl Fourier {
    pub fn new(sine: Sine) -> Self {
        Self {
            config: sine.sampling_config(),
            components: vec![sine],
        }
    }

    /// Add a sine wave component
    ///
    /// # Arguments
    /// - `sine` - `Sine` modulation
    ///
    pub fn add_component(self, sine: Sine) -> Self {
        let Self {
            mut components,
            config,
        } = self;
        let config = SamplingConfiguration::from_frequency_division(
            config
                .frequency_division()
                .min(sine.sampling_config().frequency_division()),
        )
        .unwrap();
        components.push(sine.with_sampling_config(config));
        Self { components, config }
    }

    /// Add sine wave components from iterator
    ///
    /// # Arguments
    /// - `iter` - Iterator of `Sine` modulation
    ///
    pub fn add_components_from_iter<M: Into<Sine>, T: IntoIterator<Item = M>>(
        self,
        iter: T,
    ) -> Self {
        let Self {
            mut components,
            config,
        } = self;
        let append = iter.into_iter().map(|m| m.into()).collect::<Vec<_>>();
        let freq_div = append.iter().fold(config.frequency_division(), |acc, m| {
            acc.min(m.sampling_config().frequency_division())
        });
        let config = SamplingConfiguration::from_frequency_division(freq_div).unwrap();
        components.extend(append.iter().map(|m| m.with_sampling_config(config)));
        Self { components, config }
    }
}

impl From<Sine> for Fourier {
    fn from(sine: Sine) -> Self {
        Self::new(sine)
    }
}

impl Deref for Fourier {
    type Target = [Sine];

    fn deref(&self) -> &Self::Target {
        &self.components
    }
}

impl DerefMut for Fourier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.components
    }
}

impl std::ops::Add<Sine> for Fourier {
    type Output = Self;

    fn add(self, rhs: Sine) -> Self::Output {
        self.add_component(rhs)
    }
}

impl std::ops::Add<Sine> for Sine {
    type Output = Fourier;

    fn add(self, rhs: Sine) -> Self::Output {
        Fourier::from(self).add_component(rhs)
    }
}

impl Modulation for Fourier {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let n = self.components.len();
        let buffers = self
            .components
            .iter()
            .map(|c| c.calc())
            .collect::<Result<Vec<_>, _>>()?;
        let len = buffers.iter().fold(1, |acc, x| lcm(acc, x.len()));
        Ok(buffers
            .iter()
            .map(|b| b.iter().cycle().take(len).collect::<Vec<_>>())
            .fold(vec![0usize; len], |acc, x| {
                acc.iter()
                    .zip(x.iter())
                    .map(|(a, &b)| a + b.value() as usize)
                    .collect::<Vec<_>>()
            })
            .iter()
            .map(|x| EmitIntensity::new((x / n) as u8))
            .collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use autd3_driver::defined::PI;

    #[test]
    fn test_fourier() {
        let f0 = Sine::new(50.).with_phase(PI / 2.0);
        let f1 = Sine::new(100.).with_phase(PI / 3.0);
        let f2 = Sine::new(150.).with_phase(PI / 4.0);
        let f3 = Sine::new(200.);
        let f4 = Sine::new(250.);

        let f0_buf = f0.calc().unwrap();
        let f1_buf = f1.calc().unwrap();
        let f2_buf = f2.calc().unwrap();
        let f3_buf = f3.calc().unwrap();
        let f4_buf = f4.calc().unwrap();

        let f = (f0 + f1).add_component(f2).add_components_from_iter([f3]) + f4;

        let buf = f.calc().unwrap();

        for i in 0..buf.len() {
            assert_eq!(
                buf[i].value(),
                ((f0_buf[i % f0_buf.len()].value() as usize
                    + f1_buf[i % f1_buf.len()].value() as usize
                    + f2_buf[i % f2_buf.len()].value() as usize
                    + f3_buf[i % f3_buf.len()].value() as usize
                    + f4_buf[i % f4_buf.len()].value() as usize)
                    / 5) as u8
            );
        }
    }
}
