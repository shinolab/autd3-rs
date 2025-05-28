use std::collections::HashMap;

use autd3_core::{
    gain::{BitVec, GainCalculator, GainCalculatorGenerator},
    geometry::Geometry,
};

use crate::{DatagramLightweight, Gain, RawDatagram, raw_datagram};

#[allow(clippy::wrong_self_convention)]
pub trait IntoLightweightGain {
    fn into_lightweight(self) -> Gain;
}

impl<T: IntoLightweightGain> DatagramLightweight for T {
    fn into_datagram_lightweight(
        self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<RawDatagram, crate::AUTDProtoBufError> {
        Ok(RawDatagram {
            datagram: Some(raw_datagram::Datagram::Gain(self.into_lightweight())),
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

    fn init(
        self,
        _: &Geometry,
        _: Option<&HashMap<usize, BitVec>>,
    ) -> Result<Self::G, autd3_core::gain::GainError> {
        unreachable!()
    }
}
