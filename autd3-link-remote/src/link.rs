use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use crate::{
    MSG_CLOSE, MSG_CONFIG_GEOMETRY, MSG_ERROR, MSG_OK, MSG_READ_DATA, MSG_SEND_DATA,
    MSG_UPDATE_GEOMETRY,
};

use autd3_core::link::{Link, LinkError, RxMessage, TxBufferPoolSync, TxMessage};

struct RemoteInner {
    stream: TcpStream,
    last_geometry_version: usize,
    buffer_pool: TxBufferPoolSync,
}

impl RemoteInner {
    fn open(
        addr: &SocketAddr,
        timeout: Option<Duration>,
        geometry: &autd3_core::geometry::Geometry,
    ) -> Result<RemoteInner, LinkError> {
        let mut stream = if let Some(timeout) = timeout {
            TcpStream::connect_timeout(addr, timeout)
        } else {
            TcpStream::connect(addr)
        }?;

        stream.set_write_timeout(timeout)?;
        stream.set_read_timeout(timeout)?;

        Self::send_geometry(&mut stream, MSG_CONFIG_GEOMETRY, geometry)?;
        Self::wait_response(&mut stream)?;

        let mut buffer_pool = TxBufferPoolSync::default();
        buffer_pool.init(geometry);

        Ok(Self {
            stream,
            last_geometry_version: geometry.version(),
            buffer_pool,
        })
    }

    fn send_geometry(
        stream: &mut TcpStream,
        msg_type: u8,
        geometry: &autd3_core::geometry::Geometry,
    ) -> Result<(), LinkError> {
        let num_devices = geometry.len() as u32;
        let mut buffer = Vec::with_capacity(1 + 4 + (3 + 16) * geometry.len());
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

    fn wait_response(stream: &mut TcpStream) -> Result<(), LinkError> {
        let mut status = [0u8; 1];
        stream.read_exact(&mut status)?;

        match status[0] {
            MSG_OK => Ok(()),
            MSG_ERROR => {
                let mut error_len_buf = [0u8; 4];
                stream.read_exact(&mut error_len_buf)?;
                let error_len = u32::from_le_bytes(error_len_buf) as usize;

                let mut error_msg = vec![0u8; error_len];
                stream.read_exact(&mut error_msg)?;

                let error_str = String::from_utf8_lossy(&error_msg);
                Err(LinkError::new(format!("Server error: {}", error_str)))
            }
            _ => Err(LinkError::new(format!(
                "Unknown response status: {}",
                status[0]
            ))),
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
        self.buffer_pool.borrow()
    }

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        let num_devices = tx.len() as u32;
        let data_size = std::mem::size_of::<TxMessage>();

        let mut buffer = Vec::with_capacity(1 + 4 + data_size * tx.len());
        buffer.push(MSG_SEND_DATA);
        buffer.extend_from_slice(&num_devices.to_le_bytes());

        for msg in &tx {
            let bytes = unsafe {
                std::slice::from_raw_parts(msg as *const TxMessage as *const u8, data_size)
            };
            buffer.extend_from_slice(bytes);
        }

        self.stream.write_all(&buffer)?;

        self.buffer_pool.return_buffer(tx);

        Self::wait_response(&mut self.stream)?;

        Ok(())
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, LinkError> {
        self.stream.write_all(&[MSG_READ_DATA])?;

        let mut status = [0u8; 1];
        self.stream.read_exact(&mut status)?;

        match status[0] {
            MSG_OK => {
                let mut num_devices = [0u8; 4];
                self.stream.read_exact(&mut num_devices)?;

                let num_devices = u32::from_le_bytes(num_devices) as usize;

                if num_devices != rx.len() {
                    return Ok(false);
                }

                let data_size = std::mem::size_of::<RxMessage>();
                for rx_msg in rx.iter_mut() {
                    let bytes = unsafe {
                        std::slice::from_raw_parts_mut(
                            rx_msg as *mut RxMessage as *mut u8,
                            data_size,
                        )
                    };
                    self.stream.read_exact(bytes)?;
                }

                Ok(true)
            }
            MSG_ERROR => {
                let mut error_len_buf = [0u8; 4];
                self.stream.read_exact(&mut error_len_buf)?;
                let error_len = u32::from_le_bytes(error_len_buf) as usize;

                let mut error_msg = vec![0u8; error_len];
                self.stream.read_exact(&mut error_msg)?;

                let error_str = String::from_utf8_lossy(&error_msg);
                Err(LinkError::new(format!("Server error: {}", error_str)))
            }
            _ => Err(LinkError::new(format!(
                "Unknown response status: {}",
                status[0]
            ))),
        }
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
            inner.update(geometry)?;
        }
        Ok(())
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
            inner.send(tx)?;
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            inner.receive(rx)?;
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    fn is_open(&self) -> bool {
        self.inner.is_some()
    }
}
