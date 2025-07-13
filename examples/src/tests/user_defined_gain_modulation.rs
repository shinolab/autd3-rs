use autd3::{
    core::{derive::*, link::Link},
    prelude::*,
};

#[derive(Gain, Clone, Copy, Debug)]
pub struct MyUniform {}

impl MyUniform {
    pub const fn new() -> Self {
        Self {}
    }
}

impl GainCalculator<'_> for MyUniform {
    fn calc(&self, _: &Transducer) -> Drive {
        Drive {
            intensity: Intensity::MAX,
            phase: Phase::ZERO,
        }
    }
}

impl GainCalculatorGenerator<'_, '_> for MyUniform {
    type Calculator = MyUniform;

    fn generate(&mut self, _device: &Device) -> Self::Calculator {
        MyUniform {}
    }
}

impl Gain<'_, '_, '_> for MyUniform {
    type G = MyUniform;

    fn init(
        self,
        _geometry: &Geometry,
        _env: &Environment,
        _filter: &TransducerFilter,
    ) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

#[derive(Modulation, Clone, Copy, Debug)]
pub struct Burst {
    config: SamplingConfig,
}

impl Burst {
    pub fn new() -> Self {
        Self {
            config: SamplingConfig::FREQ_4K,
        }
    }
}

impl Modulation for Burst {
    fn calc(self, _: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
        Ok((0..4000)
            .map(|i| if i == 3999 { u8::MAX } else { u8::MIN })
            .collect())
    }

    fn sampling_config(&self) -> SamplingConfig {
        self.config
    }
}

pub fn user_defined(autd: &mut Controller<impl Link, firmware::Auto>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable())?;

    let g = MyUniform::new();
    let m = Burst::new();

    autd.send((m, g))?;

    Ok(true)
}
