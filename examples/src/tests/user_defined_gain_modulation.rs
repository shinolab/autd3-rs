use autd3::{driver::link::Link, prelude::*};
use autd3_driver::derive::*;

#[derive(Gain, Clone, Copy)]
pub struct MyUniform {}

impl MyUniform {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Gain for MyUniform {
    fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
        Ok(Self::transform(|_| {
            |_| Drive::new(Phase::new(0), EmitIntensity::MAX)
        }))
    }
}

#[derive(Modulation, Clone, Copy)]
pub struct Burst {
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Burst {
    pub fn new() -> Self {
        Self {
            config: SamplingConfig::Freq(4 * kHz),
            loop_behavior: LoopBehavior::infinite(),
        }
    }
}

impl Modulation for Burst {
    fn calc(&self) -> ModulationCalcResult {
        Ok((0..4000)
            .map(|i| if i == 3999 { u8::MAX } else { u8::MIN })
            .collect())
    }
}

pub async fn user_defined(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable()).await?;

    let g = MyUniform::new();
    let m = Burst::new();

    autd.send((m, g)).await?;

    Ok(true)
}
