use crate::AUTDProtoBufError;

pub(crate) mod driver;

pub trait FromMessage<T>
where
    Self: Sized,
{
    fn from_msg(msg: T) -> Result<Self, AUTDProtoBufError>;
}
