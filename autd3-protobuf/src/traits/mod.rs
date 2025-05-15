use crate::AUTDProtoBufError;

#[cfg(feature = "lightweight")]
mod autd3;
pub(crate) mod driver;
#[cfg(feature = "lightweight")]
mod holo;

#[cfg(feature = "lightweight")]
#[allow(clippy::result_large_err)]
pub trait DatagramLightweight {
    fn into_datagram_lightweight(
        self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<crate::Datagram, AUTDProtoBufError>;
}

#[allow(clippy::result_large_err)]
pub trait FromMessage<T>
where
    Self: Sized,
{
    fn from_msg(msg: T) -> Result<Self, AUTDProtoBufError>;
}
