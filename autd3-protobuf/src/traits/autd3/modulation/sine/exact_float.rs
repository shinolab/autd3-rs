use autd3_core::modulation::ModulationProperty;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::modulation::Sine<autd3::modulation::sampling_mode::ExactFreqFloat> {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_core::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineExactFloat(SineExactFloat {
                    config: Some(self.sampling_config().to_msg(None)),
                    freq: self.freq().hz() as _,
                    intensity: Some(self.intensity() as _),
                    offset: Some(self.offset() as _),
                    phase: Some(self.phase().to_msg(None)),
                })),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<SineExactFloat>
    for autd3::modulation::Sine<autd3::modulation::sampling_mode::ExactFreqFloat>
{
    fn from_msg(msg: &SineExactFloat) -> Result<Self, AUTDProtoBufError> {
        let mut sine = autd3::modulation::Sine::new(msg.freq * autd3_core::defined::Hz);
        if let Some(intensity) = msg.intensity {
            sine = sine.with_intensity(intensity as _);
        }
        if let Some(offset) = msg.offset {
            sine = sine.with_offset(offset as _);
        }
        if msg.phase.is_some() {
            sine = sine.with_phase(autd3_core::defined::Angle::from_msg(&msg.phase)?);
        }
        if let Some(config) = msg.config.as_ref() {
            sine = sine.with_sampling_config(
                autd3_driver::firmware::fpga::SamplingConfig::from_msg(config)?,
            )?;
        }
        Ok(sine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3::modulation::sampling_mode::ExactFreqFloat;
    use autd3_core::defined::{rad, Hz};
    use rand::Rng;

    #[test]
    fn test_sine() {
        let mut rng = rand::thread_rng();

        let m = autd3::modulation::Sine::new(rng.gen::<f32>() * Hz)
            .with_intensity(rng.gen())
            .with_offset(rng.gen())
            .with_phase(rng.gen::<f32>() * rad);
        let msg = m.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineExactFloat(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Sine::<ExactFreqFloat>::from_msg(&modulation).unwrap();
                assert_eq!(m.freq(), m2.freq());
                assert_eq!(m.intensity(), m2.intensity());
                assert_eq!(m.offset(), m2.offset());
                assert_eq!(m.phase(), m2.phase());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
