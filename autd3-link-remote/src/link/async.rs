use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    MSG_CLOSE, MSG_CONFIG_GEOMETRY, MSG_ERROR, MSG_OK, MSG_READ_DATA, MSG_SEND_DATA,
    MSG_UPDATE_GEOMETRY,
    link::blocking::{REMOTE_HANDSHAKE_LEN, handshake_payload},
};

use autd3_core::{
    geometry::Geometry,
    link::{AsyncLink, LinkError, RxMessage, TxBufferPoolSync, TxMessage},
};

struct RemoteInner {
    stream: TcpStream,
    last_geometry_version: usize,
    tx_buffer_pool: TxBufferPoolSync,
    buffer: Vec<u8>,
}

impl RemoteInner {
    async fn open(addr: &SocketAddr, geometry: &Geometry) -> Result<RemoteInner, LinkError> {
        let mut stream = TcpStream::connect(addr).await?;

        Self::perform_handshake(&mut stream).await?;
        Self::send_geometry(&mut stream, MSG_CONFIG_GEOMETRY, geometry).await?;
        Self::wait_response(&mut stream).await?;

        let mut tx_buffer_pool = TxBufferPoolSync::default();
        tx_buffer_pool.init(geometry);

        Ok(Self {
            stream,
            last_geometry_version: geometry.version(),
            tx_buffer_pool,
            buffer: Vec::new(),
        })
    }

    async fn send_geometry(
        stream: &mut TcpStream,
        msg_type: u8,
        geometry: &autd3_core::geometry::Geometry,
    ) -> Result<(), LinkError> {
        let num_devices = geometry.len() as u32;

        let mut buffer = Vec::with_capacity(
            size_of::<u8>()
                + size_of::<u32>()
                + (size_of::<f32>() * 3 + size_of::<f32>() * 4) * geometry.len(),
        );
        buffer.push(msg_type);
        buffer.extend_from_slice(&num_devices.to_le_bytes());
        geometry.iter().for_each(|dev| {
            let pos = dev[0].position();
            buffer.extend_from_slice(&pos.x.to_le_bytes());
            buffer.extend_from_slice(&pos.y.to_le_bytes());
            buffer.extend_from_slice(&pos.z.to_le_bytes());

            let rot = dev.rotation();
            buffer.extend_from_slice(&rot.w.to_le_bytes());
            buffer.extend_from_slice(&rot.i.to_le_bytes());
            buffer.extend_from_slice(&rot.j.to_le_bytes());
            buffer.extend_from_slice(&rot.k.to_le_bytes());
        });

        stream.write_all(&buffer).await?;

        Ok(())
    }

    async fn perform_handshake(stream: &mut TcpStream) -> Result<(), LinkError> {
        const PAYLOAD: [u8; REMOTE_HANDSHAKE_LEN] = handshake_payload();
        stream.write_all(&PAYLOAD).await?;
        Self::wait_response(stream).await
    }

    async fn wait_response(stream: &mut TcpStream) -> Result<(), LinkError> {
        let mut status = [0u8; size_of::<u8>()];
        stream.read_exact(&mut status).await?;

        match status[0] {
            MSG_OK => Ok(()),
            MSG_ERROR => {
                let mut error_len_buf = [0u8; size_of::<u32>()];
                stream.read_exact(&mut error_len_buf).await?;
                let error_len = u32::from_le_bytes(error_len_buf) as usize;

                let mut error_msg = vec![0u8; error_len];
                stream.read_exact(&mut error_msg).await?;

                let error_str = String::from_utf8_lossy(&error_msg);
                Err(LinkError::new(format!("Server error: {}", error_str)))
            }
            msg => Err(LinkError::new(format!("Unknown response status: {}", msg))),
        }
    }

    async fn close(&mut self) -> Result<(), LinkError> {
        self.stream.write_all(&[MSG_CLOSE]).await?;
        Self::wait_response(&mut self.stream).await?;
        Ok(())
    }

    async fn update(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        if self.last_geometry_version == geometry.version() {
            return Ok(());
        }
        self.last_geometry_version = geometry.version();
        Self::send_geometry(&mut self.stream, MSG_UPDATE_GEOMETRY, geometry).await?;
        Self::wait_response(&mut self.stream).await?;
        Ok(())
    }

    fn alloc_tx_buffer(&mut self) -> Vec<TxMessage> {
        self.tx_buffer_pool.borrow()
    }

    async fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        let buffer_size = size_of::<u8>() + size_of::<TxMessage>() * tx.len();
        if self.buffer.len() < buffer_size {
            self.buffer.resize(buffer_size, 0);
        }

        self.buffer[0] = MSG_SEND_DATA;
        unsafe {
            std::ptr::copy_nonoverlapping(
                tx.as_ptr() as *const u8,
                self.buffer.as_mut_ptr().add(1),
                size_of::<TxMessage>() * tx.len(),
            );
        }
        self.tx_buffer_pool.return_buffer(tx);

        self.stream.write_all(&self.buffer).await?;
        Self::wait_response(&mut self.stream).await?;

        Ok(())
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        self.stream.write_all(&[MSG_READ_DATA]).await?;
        Self::wait_response(&mut self.stream).await?;
        for bytes in rx.iter_mut().map(|msg| unsafe {
            std::slice::from_raw_parts_mut(msg as *mut RxMessage as *mut u8, size_of::<RxMessage>())
        }) {
            self.stream.read_exact(bytes).await?;
        }
        Ok(())
    }
}

/// A [`AsyncLink`] for a remote server or [`AUTD3 Simulator`].
///
/// [`AUTD3 Simulator`]: https://github.com/shinolab/autd3-server
pub struct AsyncRemote {
    addr: SocketAddr,
    inner: Option<RemoteInner>,
}

impl AsyncRemote {
    /// Creates a new [`AsyncRemote`].
    #[must_use]
    pub const fn new(addr: SocketAddr) -> AsyncRemote {
        AsyncRemote { addr, inner: None }
    }
}

impl AsyncLink for AsyncRemote {
    async fn open(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        self.inner = Some(RemoteInner::open(&self.addr, geometry).await?);
        Ok(())
    }

    async fn close(&mut self) -> Result<(), LinkError> {
        if let Some(mut inner) = self.inner.take() {
            inner.close().await?;
        }
        Ok(())
    }

    async fn update(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.update(geometry).await
        } else {
            Err(LinkError::closed())
        }
    }

    async fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            Ok(inner.alloc_tx_buffer())
        } else {
            Err(LinkError::closed())
        }
    }

    async fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.send(tx).await
        } else {
            Err(LinkError::closed())
        }
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.receive(rx).await
        } else {
            Err(LinkError::closed())
        }
    }

    fn is_open(&self) -> bool {
        self.inner.is_some()
    }

    fn ensure_is_open(&self) -> Result<(), LinkError> {
        if self.is_open() {
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }
}
