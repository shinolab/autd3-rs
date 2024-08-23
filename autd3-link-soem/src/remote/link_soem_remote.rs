use std::{net::SocketAddr, time::Duration};

use autd3_driver::{
    derive::*,
    firmware::cpu::{RxMessage, TxDatagram},
    link::{Link, LinkBuilder},
};

use autd3_protobuf::*;

pub struct RemoteSOEM {
    client: ecat_client::EcatClient<tonic::transport::Channel>,
    timeout: Duration,
    is_open: bool,
}

#[derive(Builder)]
pub struct RemoteSOEMBuilder {
    addr: SocketAddr,
    #[get]
    #[set]
    timeout: Duration,
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for RemoteSOEMBuilder {
    type L = RemoteSOEM;

    async fn open(
        self,
        _: &autd3_driver::geometry::Geometry,
    ) -> Result<Self::L, AUTDInternalError> {
        Ok(Self::L {
            client: ecat_client::EcatClient::connect(format!("http://{}", self.addr))
                .await
                .map_err(|e| AUTDInternalError::from(AUTDProtoBufError::from(e)))?,
            timeout: self.timeout,
            is_open: true,
        })
    }
}

impl RemoteSOEM {
    pub const fn builder(addr: SocketAddr) -> RemoteSOEMBuilder {
        RemoteSOEMBuilder {
            addr,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for RemoteSOEM {
    async fn close(&mut self) -> Result<(), AUTDInternalError> {
        self.is_open = false;
        self.client
            .close(CloseRequest {})
            .await
            .map_err(AUTDProtoBufError::from)?;
        Ok(())
    }

    async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
        Ok(self
            .client
            .send_data(tx.to_msg(None))
            .await
            .map_err(AUTDProtoBufError::from)?
            .into_inner()
            .success)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
        let rx_ = Vec::<RxMessage>::from_msg(
            &self
                .client
                .read_data(ReadRequest {})
                .await
                .map_err(AUTDProtoBufError::from)?
                .into_inner(),
        )?;
        rx.copy_from_slice(&rx_);

        Ok(true)
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn timeout(&self) -> Duration {
        self.timeout
    }
}
