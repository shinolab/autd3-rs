use std::collections::HashMap;

use autd3::prelude::*;
use autd3_driver::derive::*;

#[derive(Gain, Clone, Copy)]
pub struct MyUniform {}

impl MyUniform {
    pub fn new() -> Self {
        Self {}
    }
}

impl Gain for MyUniform {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(Self::transform(geometry, filter, |_dev, _tr| {
            Drive::new(Phase::new(0), EmitIntensity::MAX)
        }))
    }
}

#[derive(Modulation, Clone, Copy)]
pub struct Burst {
    config: SamplingConfiguration,
    loop_behavior: LoopBehavior,
}

impl Burst {
    pub fn new() -> Self {
        Self {
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::Infinite,
        }
    }
}

impl Modulation for Burst {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        Ok((0..4000)
            .map(|i| {
                if i == 3999 {
                    EmitIntensity::MAX
                } else {
                    EmitIntensity::MIN
                }
            })
            .collect())
    }
}

pub async fn custom(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(ConfigureSilencer::disable()).await?;

    let g = MyUniform::new();
    let m = Burst::new();

    autd.send((m, g)).await?;

    Ok(true)
}
