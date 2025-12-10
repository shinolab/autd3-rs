use std::net::IpAddr;

use autd3_core::{
    geometry::Geometry,
    link::{Link, LinkError, RxMessage, TxBufferPoolSync, TxMessage},
};

use crate::error::AdsError;

pub use ads::{AmsAddr, AmsNetId};
pub use ads::{Source, Timeouts};

const INDEX_GROUP: u32 = 0x0304_0030;
const INDEX_OFFSET_BASE: u32 = 0x8100_0000;
const INDEX_OFFSET_BASE_READ: u32 = 0x8000_0000;
const PORT: u16 = 301;

/// A [`Link`] using TwinCAT3.
///
/// To use this link, you need to install TwinCAT3 and run [`TwinCATAUTDServer`] on server side.
///
/// [`TwinCATAUTDServer`]: https://github.com/shinolab/autd3-server
pub struct RemoteTwinCAT {
    addr: IpAddr,
    ams_addr: AmsAddr,
    option: RemoteTwinCATOption,
    client: Option<ads::Client>,
    buffer_pool: TxBufferPoolSync,
}

/// The option of [`RemoteTwinCAT`].
#[derive(Debug)]
pub struct RemoteTwinCATOption {
    /// The timeouts for ADS communication.
    pub timeouts: Timeouts,
    /// The source AMS address.
    pub source: Source,
}

impl Default for RemoteTwinCATOption {
    fn default() -> Self {
        RemoteTwinCATOption {
            timeouts: Timeouts::none(),
            source: Source::Auto,
        }
    }
}

impl RemoteTwinCAT {
    /// Creates a new [`RemoteTwinCAT`].
    ///
    /// # Arguments
    /// - `addr`: The IP address of the server running TwinCAT3.
    /// - `ams_net_id`: The AMS Net ID of the server running TwinCAT3.
    /// - `option`: The option of [`RemoteTwinCAT`].
    #[must_use]
    pub fn new(addr: IpAddr, ams_net_id: AmsNetId, option: RemoteTwinCATOption) -> RemoteTwinCAT {
        RemoteTwinCAT {
            addr,
            ams_addr: AmsAddr::new(ams_net_id, PORT),
            option,
            client: None,
            buffer_pool: TxBufferPoolSync::default(),
        }
    }
}

impl Link for RemoteTwinCAT {
    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        let RemoteTwinCATOption { timeouts, source } = self.option;

        self.client = Some(
            ads::Client::new((self.addr, ads::PORT), timeouts, source).map_err(AdsError::from)?,
        );
        self.buffer_pool.init(geometry);

        Ok(())
    }

    fn close(&mut self) -> Result<(), LinkError> {
        let _ = self.client.take();
        Ok(())
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        Ok(self.buffer_pool.borrow())
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        if let Some(client) = self.client.as_mut() {
            let device = client.device(self.ams_addr);
            let data = unsafe {
                std::slice::from_raw_parts(
                    tx.as_ptr() as *const u8,
                    tx.len() * std::mem::size_of::<TxMessage>(),
                )
            };
            let res = device.write(INDEX_GROUP, INDEX_OFFSET_BASE, data);
            self.buffer_pool.return_buffer(tx);
            res.map_err(AdsError::from)?;
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        if let Some(client) = self.client.as_mut() {
            let device = client.device(self.ams_addr);
            let data = unsafe {
                std::slice::from_raw_parts_mut(
                    rx.as_mut_ptr() as *mut u8,
                    std::mem::size_of_val(rx),
                )
            };
            device
                .read(INDEX_GROUP, INDEX_OFFSET_BASE_READ, data)
                .map_err(AdsError::from)?;
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    fn is_open(&self) -> bool {
        self.client.is_some()
    }
}

#[cfg(feature = "async")]
impl autd3_core::link::AsyncLink for RemoteTwinCAT {
    async fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        <Self as Link>::open(self, geometry)
    }

    async fn close(&mut self) -> Result<(), LinkError> {
        <Self as Link>::close(self)
    }

    async fn update(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        <Self as Link>::update(self, geometry)
    }

    async fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        <Self as Link>::alloc_tx_buffer(self)
    }

    async fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        <Self as Link>::send(self, tx)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        <Self as Link>::receive(self, rx)
    }

    fn is_open(&self) -> bool {
        <Self as Link>::is_open(self)
    }

    fn ensure_is_open(&self) -> Result<(), LinkError> {
        <Self as Link>::ensure_is_open(self)
    }
}
