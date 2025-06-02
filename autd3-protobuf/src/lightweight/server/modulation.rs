use crate::{AUTDProtoBufError, FromMessage, pb::modulation::Modulation};
use autd3::modulation::*;
use autd3_core::common::Freq;
use autd3_driver::datagram::BoxedModulation;

pub(crate) fn modulation_into_boxed(
    msg: crate::pb::Modulation,
) -> Result<BoxedModulation, AUTDProtoBufError> {
    let modulation = msg.modulation.ok_or(AUTDProtoBufError::DataParseError)?;
    match modulation {
        Modulation::Static(msg) => Static::from_msg(msg).map(BoxedModulation::new),
        Modulation::SineNearest(msg) => {
            Sine::<sampling_mode::Nearest>::from_msg(msg).map(BoxedModulation::new)
        }
        Modulation::SineExact(msg) => Sine::<Freq<u32>>::from_msg(msg).map(BoxedModulation::new),
        Modulation::SineExactFloat(msg) => {
            Sine::<Freq<f32>>::from_msg(msg).map(BoxedModulation::new)
        }
        Modulation::SquareNearest(msg) => {
            Square::<sampling_mode::Nearest>::from_msg(msg).map(BoxedModulation::new)
        }
        Modulation::SquareExact(msg) => {
            Square::<Freq<u32>>::from_msg(msg).map(BoxedModulation::new)
        }
        Modulation::SquareExactFloat(msg) => {
            Square::<Freq<f32>>::from_msg(msg).map(BoxedModulation::new)
        }
    }
}
