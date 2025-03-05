use std::net::SocketAddr;

use autd3_core::geometry::{Device, Geometry};

use derive_more::Deref;

use crate::{OpenRequestLightweight, SenderOption, traits::*};

type Client = crate::pb::ecat_light_client::EcatLightClient<tonic::transport::Channel>;

#[derive(Deref)]
pub struct Controller {
    client: Client,
    #[deref]
    geometry: Geometry,
}

pub struct Sender<'a, S: autd3::r#async::controller::AsyncSleep>
where
    autd3::controller::SenderOption<S>: ToMessage<Message = SenderOption>,
{
    pub(crate) controller: &'a mut Controller,
    pub(crate) option: autd3::controller::SenderOption<S>,
}

pub trait Datagram {
    fn send(
        self,
        client: &mut Client,
        sender_option: Option<crate::SenderOption>,
    ) -> impl std::future::Future<Output = Result<(), crate::error::AUTDProtoBufError>>;
}

impl<T> Datagram for T
where
    T: ToMessage<Message = crate::pb::Datagram>,
{
    async fn send(
        self,
        client: &mut Client,
        sender_option: Option<crate::SenderOption>,
    ) -> Result<(), crate::error::AUTDProtoBufError> {
        let res = client
            .send(tonic::Request::new(crate::SendRequestLightweight {
                datagram: Some(crate::DatagramTuple {
                    first: Some(self.to_msg(None)?),
                    second: None,
                }),
                sender_option,
            }))
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(())
    }
}

impl<T1, T2> Datagram for (T1, T2)
where
    T1: ToMessage<Message = crate::pb::Datagram>,
    T2: ToMessage<Message = crate::pb::Datagram>,
{
    async fn send(
        self,
        client: &mut Client,
        sender_option: Option<crate::SenderOption>,
    ) -> Result<(), crate::error::AUTDProtoBufError> {
        let (d1, d2) = self;
        let res = client
            .send(tonic::Request::new(crate::SendRequestLightweight {
                datagram: Some(crate::DatagramTuple {
                    first: Some(d1.to_msg(None)?),
                    second: Some(d2.to_msg(None)?),
                }),
                sender_option,
            }))
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(())
    }
}

impl Controller {
    pub async fn open<D: Into<Device>, F: IntoIterator<Item = D>>(
        devices: F,
        addr: SocketAddr,
    ) -> Result<Self, crate::error::AUTDProtoBufError> {
        Controller::open_impl(devices.into_iter().map(|d| d.into()).collect(), addr, None).await
    }

    pub async fn open_with_option<
        D: Into<Device>,
        F: IntoIterator<Item = D>,
        S: autd3::r#async::controller::AsyncSleep,
    >(
        devices: F,
        addr: SocketAddr,
        sender_option: autd3::controller::SenderOption<S>,
    ) -> Result<Self, crate::error::AUTDProtoBufError>
    where
        autd3::controller::SenderOption<S>: ToMessage<Message = SenderOption>,
    {
        Controller::open_impl(
            devices.into_iter().map(|d| d.into()).collect(),
            addr,
            Some(sender_option.to_msg(None)?),
        )
        .await
    }

    async fn open_impl(
        devices: Vec<Device>,
        addr: SocketAddr,
        sender_option: Option<crate::SenderOption>,
    ) -> Result<Self, crate::error::AUTDProtoBufError> {
        let conn = tonic::transport::Endpoint::new(format!("http://{}", addr))?
            .connect()
            .await?;
        let mut client = crate::pb::ecat_light_client::EcatLightClient::new(conn);
        let geometry = Geometry::new(devices);
        let res = client
            .open(OpenRequestLightweight {
                geometry: Some(geometry.to_msg(None)?),
                sender_option,
            })
            .await?
            .into_inner();
        if res.err {
            return Err(crate::error::AUTDProtoBufError::SendError(res.msg));
        }
        Ok(Self { client, geometry })
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
        datagram.send(&mut self.client, None).await
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

    pub fn sender<S: autd3::r#async::controller::AsyncSleep>(
        &mut self,
        option: autd3::controller::SenderOption<S>,
    ) -> Sender<S>
    where
        autd3::controller::SenderOption<S>: ToMessage<Message = SenderOption>,
    {
        Sender {
            controller: self,
            option,
        }
    }
}

impl<S: autd3::r#async::controller::AsyncSleep> Sender<'_, S>
where
    autd3::controller::SenderOption<S>: ToMessage<Message = SenderOption>,
{
    pub async fn send(
        &mut self,
        datagram: impl Datagram,
    ) -> Result<(), crate::error::AUTDProtoBufError> {
        datagram
            .send(&mut self.controller.client, Some(self.option.to_msg(None)?))
            .await
    }
}
