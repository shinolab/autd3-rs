use std::net::SocketAddr;

use autd3_core::geometry::{Device, Geometry};

use derive_more::Deref;

use crate::{OpenRequestLightweight, Sleeper, traits::*};

type Client = crate::pb::ecat_light_client::EcatLightClient<tonic::transport::Channel>;

#[derive(Deref)]
pub struct Controller {
    client: Client,
    #[deref]
    geometry: Geometry,
    /// The default sender option used for [`send`](Controller::send).
    pub default_sender_option: autd3::controller::SenderOption,
}

pub struct Sender<'a, S>
where
    S: autd3::r#async::controller::AsyncSleep,
    Sleeper: for<'b> From<&'b S>,
{
    pub(crate) controller: &'a mut Controller,
    pub(crate) option: autd3::controller::SenderOption,
    pub(crate) sleeper: S,
}

pub trait Datagram {
    fn into_lightweight(self) -> crate::DatagramTuple;
}

impl<T> Datagram for T
where
    T: DatagramLightweight,
{
    fn into_lightweight(self) -> crate::DatagramTuple {
        crate::DatagramTuple {
            first: Some(self.into_datagram_lightweight(None).unwrap()),
            second: None,
        }
    }
}

impl<T1, T2> Datagram for (T1, T2)
where
    T1: DatagramLightweight,
    T2: DatagramLightweight,
{
    fn into_lightweight(self) -> crate::DatagramTuple {
        crate::DatagramTuple {
            first: Some(self.0.into_datagram_lightweight(None).unwrap()),
            second: Some(self.1.into_datagram_lightweight(None).unwrap()),
        }
    }
}

impl Controller {
    pub async fn open<D: Into<Device>, F: IntoIterator<Item = D>>(
        devices: F,
        addr: SocketAddr,
    ) -> Result<Self, crate::error::AUTDProtoBufError> {
        Controller::open_impl(
            devices.into_iter().map(|d| d.into()).collect(),
            addr,
            Default::default(),
            autd3::r#async::AsyncSleeper::default(),
        )
        .await
    }

    pub async fn open_with_option<D: Into<Device>, F: IntoIterator<Item = D>, S>(
        devices: F,
        addr: SocketAddr,
        sender_option: autd3::controller::SenderOption,
        sleeper: S,
    ) -> Result<Self, crate::error::AUTDProtoBufError>
    where
        S: autd3::r#async::controller::AsyncSleep,
        Sleeper: for<'a> From<&'a S>,
    {
        Controller::open_impl(
            devices.into_iter().map(|d| d.into()).collect(),
            addr,
            sender_option,
            sleeper,
        )
        .await
    }

    async fn open_impl<S>(
        devices: Vec<Device>,
        addr: SocketAddr,
        option: autd3::controller::SenderOption,
        sleeper: S,
    ) -> Result<Self, crate::error::AUTDProtoBufError>
    where
        S: autd3::r#async::controller::AsyncSleep,
        Sleeper: for<'a> From<&'a S>,
    {
        let conn = tonic::transport::Endpoint::new(format!("http://{}", addr))?
            .connect()
            .await?;
        let mut client = crate::pb::ecat_light_client::EcatLightClient::new(conn);
        let geometry = Geometry::new(devices);
        let res = client
            .open(OpenRequestLightweight {
                geometry: Some((&geometry).into()),
                sender_option: Some(option.into()),
                sleeper: Some((&sleeper).into()),
            })
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(Self {
            client,
            geometry,
            default_sender_option: option,
        })
    }

    pub async fn firmware_version(
        &mut self,
    ) -> Result<
        Vec<autd3_driver::firmware::version::FirmwareVersion>,
        crate::error::AUTDProtoBufError,
    > {
        let res = self
            .client
            .firmware_version(tonic::Request::new(
                crate::pb::FirmwareVersionRequestLightweight {},
            ))
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Vec::from_msg(res)
    }

    pub async fn fpga_state(
        &mut self,
    ) -> Result<Vec<Option<autd3_driver::firmware::fpga::FPGAState>>, crate::error::AUTDProtoBufError>
    {
        let res = self
            .client
            .fpga_state(tonic::Request::new(
                crate::pb::FpgaStateRequestLightweight {},
            ))
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Vec::from_msg(res)
    }

    pub async fn send(
        &mut self,
        datagram: impl Datagram,
    ) -> Result<(), crate::error::AUTDProtoBufError> {
        let res = self
            .client
            .send(tonic::Request::new(crate::SendRequestLightweight {
                datagram: Some(datagram.into_lightweight()),
                sender_option: Some(self.default_sender_option.into()),
                sleeper: None,
            }))
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(())
    }

    pub async fn close(mut self) -> Result<(), crate::error::AUTDProtoBufError> {
        let res = self
            .client
            .close(crate::pb::CloseRequestLightweight {})
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(())
    }

    pub fn sender<S>(&mut self, option: autd3::controller::SenderOption, sleeper: S) -> Sender<S>
    where
        S: autd3::r#async::controller::AsyncSleep,
        Sleeper: for<'a> From<&'a S>,
    {
        Sender {
            controller: self,
            option,
            sleeper,
        }
    }
}

impl<S> Sender<'_, S>
where
    S: autd3::r#async::controller::AsyncSleep,
    Sleeper: for<'a> From<&'a S>,
{
    pub async fn send(
        &mut self,
        datagram: impl Datagram,
    ) -> Result<(), crate::error::AUTDProtoBufError> {
        let res = self
            .controller
            .client
            .send(tonic::Request::new(crate::SendRequestLightweight {
                datagram: Some(datagram.into_lightweight()),
                sender_option: Some(self.option.into()),
                sleeper: Some((&self.sleeper).into()),
            }))
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(())
    }
}
