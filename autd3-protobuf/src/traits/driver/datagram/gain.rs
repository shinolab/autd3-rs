use autd3_core::gain::{GainCalculator, GainCalculatorGenerator};

use crate::{Datagram, Gain, ToMessage, datagram};

#[allow(clippy::wrong_self_convention)]
pub trait IntoLightweightGain {
    fn into_lightweight(&self) -> Gain;
}

impl<T: IntoLightweightGain> ToMessage for T {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, crate::AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Gain(self.into_lightweight())),
        })
    }
}

// Followings are required to GainSTM in lightweight mode
pub struct Nop {}

impl GainCalculator for Nop {
    fn calc(&self, _: &autd3_core::derive::Transducer) -> autd3::prelude::Drive {
        unreachable!()
    }
}

impl GainCalculatorGenerator for Nop {
    type Calculator = Self;

    fn generate(&mut self, _: &autd3_core::derive::Device) -> Self::Calculator {
        unreachable!()
    }
}

impl autd3_core::gain::Gain for Gain {
    type G = Nop;

    fn init(self) -> Result<Self::G, autd3_core::gain::GainError> {
        unreachable!()
    }
}
