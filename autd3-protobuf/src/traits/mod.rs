use crate::AUTDProtoBufError;

#[cfg(feature = "lightweight")]
mod autd3;
mod driver;
#[cfg(feature = "lightweight")]
mod holo;

pub trait ToMessage {
    type Message: prost::Message;

    fn to_msg(&self, geometry: Option<&autd3_driver::geometry::Geometry>) -> Self::Message;
}

pub trait FromMessage<T>
where
    Self: Sized,
{
    fn from_msg(msg: &T) -> Result<Self, AUTDProtoBufError>;
}
