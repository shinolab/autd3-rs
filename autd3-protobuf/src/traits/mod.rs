use crate::{AUTDProtoBufError, Datagram};

#[cfg(feature = "lightweight")]
mod autd3;
pub(crate) mod driver;
#[cfg(feature = "lightweight")]
mod holo;

pub trait DatagramLightweight {
    fn into_datagram_lightweight(
        self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Datagram, AUTDProtoBufError>;
}

pub trait FromMessage<T>
where
    Self: Sized,
{
    fn from_msg(msg: T) -> Result<Self, AUTDProtoBufError>;
}
