use crate::AUTDProtoBufError;

#[cfg(feature = "lightweight")]
mod autd3;
pub(crate) mod driver;
#[cfg(feature = "lightweight")]
mod holo;

#[cfg(feature = "lightweight")]
pub trait DatagramLightweight {
    fn into_datagram_lightweight(
        self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<crate::RawDatagram, AUTDProtoBufError>;
}

pub trait FromMessage<T>
where
    Self: Sized,
{
    fn from_msg(msg: T) -> Result<Self, AUTDProtoBufError>;
}
