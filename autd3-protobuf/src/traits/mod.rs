use crate::AUTDProtoBufError;

#[cfg(feature = "lightweight")]
mod autd3;
pub(crate) mod driver;
#[cfg(feature = "lightweight")]
mod holo;

pub trait ToMessage {
    type Message: prost::Message;

    fn to_msg(
        &self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError>;
}

pub trait FromMessage<T>
where
    Self: Sized,
{
    fn from_msg(msg: T) -> Result<Self, AUTDProtoBufError>;
}
