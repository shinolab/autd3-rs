use std::{fmt::Debug, hash::Hash, net::SocketAddr};

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

pub struct LightweightDatagram {
    #[allow(clippy::type_complexity)]
    g: Box<dyn FnOnce(&Geometry) -> Result<crate::Datagram, crate::error::AUTDProtoBufError>>,
}

impl std::fmt::Debug for LightweightDatagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LightweightDatagram").finish()
    }
}

pub trait Datagram {
    fn into_lightweight(self) -> LightweightDatagram;
}

pub struct NullOperationGenerator;

impl autd3_driver::firmware::operation::OperationGenerator for NullOperationGenerator {
    type O1 = autd3_core::datagram::NullOp;
    type O2 = autd3_core::datagram::NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((Self::O1 {}, Self::O2 {}))
    }
}

impl autd3_core::datagram::Datagram for LightweightDatagram {
    type G = NullOperationGenerator;
    type Error = std::convert::Infallible;

    fn operation_generator(self, _: &mut Geometry) -> Result<Self::G, Self::Error> {
        unimplemented!("`LightweightDatagram` does not support normal `Controller`");
    }
}

impl Datagram for LightweightDatagram {
    fn into_lightweight(self) -> LightweightDatagram {
        self
    }
}

impl<T> Datagram for T
where
    T: DatagramLightweight + 'static,
{
    fn into_lightweight(self) -> LightweightDatagram {
        LightweightDatagram {
            g: Box::new(move |geometry| {
                Ok(crate::Datagram {
                    datagram: Some(crate::datagram::Datagram::Tuple(crate::DatagramTuple {
                        first: Some(self.into_datagram_lightweight(Some(geometry)).unwrap()),
                        second: None,
                    })),
                })
            }),
        }
    }
}

impl<K, D, F> Datagram for autd3_driver::datagram::Group<K, D, F>
where
    K: Hash + Eq + Debug + 'static,
    D: autd3_driver::datagram::Datagram + Datagram + 'static,
    F: Fn(&Device) -> Option<K> + 'static,
    D::G: autd3_driver::firmware::operation::OperationGenerator,
    autd3::prelude::AUTDDriverError: From<<D as autd3_driver::datagram::Datagram>::Error>,
{
    fn into_lightweight(self) -> LightweightDatagram {
        LightweightDatagram {
            g: Box::new(move |geometry| {
                let (datagram_key, datagrams): (Vec<_>, Vec<_>) =
                    self.datagram_map.into_iter().collect();
                let keys = geometry
                    .iter()
                    .map(|dev| {
                        if !dev.enable {
                            return -1;
                        }
                        if let Some(key) = (self.key_map)(dev) {
                            datagram_key
                                .iter()
                                .position(|k| k == &key)
                                .map(|i| i as i32)
                                .unwrap_or(-1)
                        } else {
                            -1
                        }
                    })
                    .collect();
                let datagrams = datagrams
                    .into_iter()
                    .map(|d| match (d.into_lightweight().g)(geometry)?.datagram {
                        Some(crate::datagram::Datagram::Tuple(tuple)) => Ok(tuple),
                        _ => Err(crate::error::AUTDProtoBufError::SendError(
                            "Nested `Group` is not supported ".to_string(),
                        )),
                    })
                    .collect::<Result<_, _>>()?;
                Ok(crate::Datagram {
                    datagram: Some(crate::datagram::Datagram::Group(crate::Group {
                        keys,
                        datagrams,
                    })),
                })
            }),
        }
    }
}

impl<T1, T2> Datagram for (T1, T2)
where
    T1: DatagramLightweight + 'static,
    T2: DatagramLightweight + 'static,
{
    fn into_lightweight(self) -> LightweightDatagram {
        LightweightDatagram {
            g: Box::new(move |geometry| {
                Ok(crate::Datagram {
                    datagram: Some(crate::datagram::Datagram::Tuple(crate::DatagramTuple {
                        first: Some(self.0.into_datagram_lightweight(Some(geometry)).unwrap()),
                        second: Some(self.1.into_datagram_lightweight(Some(geometry)).unwrap()),
                    })),
                })
            }),
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
                datagram: Some((datagram.into_lightweight().g)(&self.geometry)?),
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
                datagram: Some((datagram.into_lightweight().g)(&self.controller.geometry)?),
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
