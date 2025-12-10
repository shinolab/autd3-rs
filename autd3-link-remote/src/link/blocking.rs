use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use crate::{
    MSG_CLOSE, MSG_CONFIG_GEOMETRY, MSG_ERROR, MSG_HELLO, MSG_OK, MSG_READ_DATA, MSG_SEND_DATA,
    MSG_UPDATE_GEOMETRY, REMOTE_PROTOCOL_MAGIC, REMOTE_PROTOCOL_VERSION,
};

use autd3_core::{
    geometry::Geometry,
    link::{Link, LinkError, RxMessage, TxBufferPoolSync, TxMessage},
};

pub(crate) const REMOTE_HANDSHAKE_LEN: usize =
    size_of::<u8>() + size_of::<u16>() + REMOTE_PROTOCOL_MAGIC.len();
pub(crate) const fn handshake_payload() -> [u8; REMOTE_HANDSHAKE_LEN] {
    let mut payload = [0u8; REMOTE_HANDSHAKE_LEN];
    payload[0] = MSG_HELLO;

    let version = REMOTE_PROTOCOL_VERSION.to_le_bytes();
    let version_end = 1 + version.len();
    let mut i = 1;
    while i < version_end {
        payload[i] = version[i - 1];
        i += 1;
    }
    while i < REMOTE_HANDSHAKE_LEN {
        payload[i] = REMOTE_PROTOCOL_MAGIC[i - version_end];
        i += 1;
    }
    payload
}

struct RemoteInner {
    stream: TcpStream,
    last_geometry_version: usize,
    tx_buffer_pool: TxBufferPoolSync,
    buffer: Vec<u8>,
}

impl RemoteInner {
    fn open(
        addr: &SocketAddr,
        timeout: Option<Duration>,
        geometry: &Geometry,
    ) -> Result<RemoteInner, LinkError> {
        let mut stream = if let Some(timeout) = timeout {
            TcpStream::connect_timeout(addr, timeout)
        } else {
            TcpStream::connect(addr)
        }?;

        stream.set_write_timeout(timeout)?;
        stream.set_read_timeout(timeout)?;

        Self::perform_handshake(&mut stream)?;
        Self::send_geometry(&mut stream, MSG_CONFIG_GEOMETRY, geometry)?;
        Self::wait_response(&mut stream)?;

        let mut tx_buffer_pool = TxBufferPoolSync::default();
        tx_buffer_pool.init(geometry);

        Ok(Self {
            stream,
            last_geometry_version: geometry.version(),
            tx_buffer_pool,
            buffer: Vec::new(),
        })
    }

    fn send_geometry(
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

        stream.write_all(&buffer)?;

        Ok(())
    }

    fn perform_handshake(stream: &mut TcpStream) -> Result<(), LinkError> {
        const PAYLOAD: [u8; REMOTE_HANDSHAKE_LEN] = handshake_payload();
        stream.write_all(&PAYLOAD)?;
        Self::wait_response(stream)
    }

    fn wait_response(stream: &mut TcpStream) -> Result<(), LinkError> {
        let mut status = [0u8; size_of::<u8>()];
        stream.read_exact(&mut status)?;

        match status[0] {
            MSG_OK => Ok(()),
            MSG_ERROR => {
                let mut error_len_buf = [0u8; size_of::<u32>()];
                stream.read_exact(&mut error_len_buf)?;
                let error_len = u32::from_le_bytes(error_len_buf) as usize;

                let mut error_msg = vec![0u8; error_len];
                stream.read_exact(&mut error_msg)?;

                let error_str = String::from_utf8_lossy(&error_msg);
                Err(LinkError::new(format!("Server error: {}", error_str)))
            }
            msg => Err(LinkError::new(format!("Unknown response status: {}", msg))),
        }
    }

    fn close(&mut self) -> Result<(), LinkError> {
        self.stream.write_all(&[MSG_CLOSE])?;
        Self::wait_response(&mut self.stream)?;
        Ok(())
    }

    fn update(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        if self.last_geometry_version == geometry.version() {
            return Ok(());
        }
        self.last_geometry_version = geometry.version();
        Self::send_geometry(&mut self.stream, MSG_UPDATE_GEOMETRY, geometry)?;
        Self::wait_response(&mut self.stream)?;
        Ok(())
    }

    fn alloc_tx_buffer(&mut self) -> Vec<TxMessage> {
        self.tx_buffer_pool.borrow()
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
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

        self.stream.write_all(&self.buffer)?;
        Self::wait_response(&mut self.stream)?;

        Ok(())
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        self.stream.write_all(&[MSG_READ_DATA])?;
        Self::wait_response(&mut self.stream)?;
        rx.iter_mut()
            .map(|msg| unsafe {
                std::slice::from_raw_parts_mut(
                    msg as *mut RxMessage as *mut u8,
                    size_of::<RxMessage>(),
                )
            })
            .try_for_each(|bytes| self.stream.read_exact(bytes))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
/// Options for [`Remote`].
pub struct RemoteOption {
    /// Timeout duration for connecting and read/write operations. The default is `None`, which means no timeout.
    pub timeout: Option<Duration>,
}

/// A [`Link`] for a remote server or [`AUTD3 Simulator`].
///
/// [`AUTD3 Simulator`]: https://github.com/shinolab/autd3-server
pub struct Remote {
    addr: SocketAddr,
    inner: Option<RemoteInner>,
    option: RemoteOption,
}

impl Remote {
    /// Creates a new [`Remote`].
    #[must_use]
    pub const fn new(addr: SocketAddr, option: RemoteOption) -> Remote {
        Remote {
            addr,
            inner: None,
            option,
        }
    }
}

impl Link for Remote {
    fn open(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        self.inner = Some(RemoteInner::open(
            &self.addr,
            self.option.timeout,
            geometry,
        )?);
        Ok(())
    }

    fn close(&mut self) -> Result<(), LinkError> {
        if let Some(mut inner) = self.inner.take() {
            inner.close()?;
        }
        Ok(())
    }

    fn update(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.update(geometry)
        } else {
            Err(LinkError::closed())
        }
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            Ok(inner.alloc_tx_buffer())
        } else {
            Err(LinkError::closed())
        }
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.send(tx)
        } else {
            Err(LinkError::closed())
        }
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.receive(rx)
        } else {
            Err(LinkError::closed())
        }
    }

    fn is_open(&self) -> bool {
        self.inner.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_payload_format() {
        let payload = handshake_payload();
        assert_eq!(payload.len(), REMOTE_HANDSHAKE_LEN);
        assert_eq!(payload[0], MSG_HELLO);
        let version_bytes = REMOTE_PROTOCOL_VERSION.to_le_bytes();
        assert_eq!(payload[1..1 + version_bytes.len()], version_bytes);
        assert_eq!(
            &payload[1 + version_bytes.len()..],
            REMOTE_PROTOCOL_MAGIC.as_slice()
        );
    }
}
