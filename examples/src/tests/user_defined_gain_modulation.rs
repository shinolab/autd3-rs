use std::sync::Arc;

use autd3::{driver::link::Link, prelude::*};
use autd3_driver::derive::*;

#[derive(Gain, Clone, Copy, Debug)]
pub struct MyUniform {}

impl MyUniform {
    pub const fn new() -> Self {
        Self {}
    }
}

pub struct Context {}

impl GainContext for Context {
    fn calc(&self, _: &Transducer) -> Drive {
        EmitIntensity::MAX.into()
    }
}

impl GainContextGenerator for MyUniform {
    type Context = Context;

    fn generate(&mut self, _device: &Device) -> Self::Context {
        Context {}
    }
}

impl Gain for MyUniform {
    type G = MyUniform;

    fn init_with_filter(
        self,
        _geometry: &Geometry,
        _filter: Option<HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::G, AUTDInternalError> {
        Ok(self)
    }
}

#[derive(Modulation, Clone, Copy, Debug)]
pub struct Burst {
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Burst {
    pub fn new() -> Self {
        Self {
            config: SamplingConfig::FREQ_4K,
            loop_behavior: LoopBehavior::infinite(),
        }
    }
}

impl Modulation for Burst {
    fn calc(&self) -> Result<Arc<Vec<u8>>, AUTDInternalError> {
        Ok(Arc::new(
            (0..4000)
                .map(|i| if i == 3999 { u8::MAX } else { u8::MIN })
                .collect(),
        ))
    }
}

pub async fn user_defined(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable()).await?;

    let g = MyUniform::new();
    let m = Burst::new();

    autd.send((m, g)).await?;

    Ok(true)
}
