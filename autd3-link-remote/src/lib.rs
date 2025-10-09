#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides a link for remote server.

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

use autd3_core::{
    geometry::Geometry,
    link::{Link, LinkError, RxMessage, TxBufferPoolSync, TxMessage},
};

// # Protocol Specification
//
// This link uses a simple TCP-based binary protocol for communication with the remote server.
//
// ## Message Types
//
// - `0x03`: Send Data
// - `0x04`: Read Data
// - `0x05`: Close
//
// ## Message Formats
//
// ### Send Data
// Request:
// - 1 byte: message type (0x03)
// - 4 bytes: number of devices (u32, little-endian)
// - Raw TxMessage data for each device
//
// Response:
// - 1 byte: status (0x00 = OK)
//
// ### Read Data
// Request:
// - 1 byte: message type (0x04)
//
// Response:
// - 1 byte: status (0x00 = OK)
// - 4 bytes: number of devices (u32, little-endian)
// - Raw RxMessage data for each device
//
// ### Close
// Request:
// - 1 byte: message type (0x05)
//
// Response:
// - 1 byte: status (0x00 = OK)

/// Message type: Send Data
pub const MSG_SEND_DATA: u8 = 0x03;
/// Message type: Read Data
pub const MSG_READ_DATA: u8 = 0x04;
/// Message type: Close
pub const MSG_CLOSE: u8 = 0x05;

/// Status: OK
pub const MSG_OK: u8 = 0x00;

struct RemoteInner {
    stream: TcpStream,
    buffer_pool: TxBufferPoolSync,
}

impl RemoteInner {
    fn open(addr: &SocketAddr, geometry: &Geometry) -> Result<Self, LinkError> {
        let stream = TcpStream::connect(addr)
            .map_err(|e| LinkError::new(format!("Failed to connect to remote server: {e}")))?;

        let mut buffer_pool = TxBufferPoolSync::new();
        buffer_pool.init(geometry);

        Ok(Self {
            stream,
            buffer_pool,
        })
    }

    fn wait_response(stream: &mut TcpStream) -> Result<(), LinkError> {
        let mut status = [0u8; 1];
        stream
            .read_exact(&mut status)
            .map_err(|e| LinkError::new(format!("Failed to read response: {e}")))?;

        if status[0] == MSG_OK {
            Ok(())
        } else {
            Err(LinkError::new("Remote server returned error"))
        }
    }

    fn close(&mut self) -> Result<(), LinkError> {
        self.stream
            .write_all(&[MSG_CLOSE])
            .map_err(|e| LinkError::new(format!("Failed to send close message: {e}")))?;

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

        self.stream
            .write_all(&buffer)
            .map_err(|e| LinkError::new(format!("Failed to send data: {e}")))?;

        self.buffer_pool.return_buffer(tx);

        Self::wait_response(&mut self.stream)?;

        Ok(())
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        self.stream
            .write_all(&[MSG_READ_DATA])
            .map_err(|e| LinkError::new(format!("Failed to send read request: {e}")))?;

        let mut status = [0u8; 1];
        self.stream
            .read_exact(&mut status)
            .map_err(|e| LinkError::new(format!("Failed to read response: {e}")))?;

        if status[0] != MSG_OK {
            return Err(LinkError::new("Remote server returned error"));
        }

        let mut num_devices = [0u8; 4];
        self.stream
            .read_exact(&mut num_devices)
            .map_err(|e| LinkError::new(format!("Failed to read num_devices: {e}")))?;

        let num_devices = u32::from_le_bytes(num_devices) as usize;

        if num_devices != rx.len() {
            return Err(LinkError::new(format!(
                "Received unexpected number of devices: expected {}, got {}",
                rx.len(),
                num_devices
            )));
        }

        let data_size = std::mem::size_of::<RxMessage>();
        for rx_msg in rx.iter_mut() {
            let bytes = unsafe {
                std::slice::from_raw_parts_mut(rx_msg as *mut RxMessage as *mut u8, data_size)
            };
            self.stream
                .read_exact(bytes)
                .map_err(|e| LinkError::new(format!("Failed to read rx data: {e}")))?;
        }

        Ok(())
    }
}

/// An [`Link`] for a remote server.
pub struct Remote {
    addr: SocketAddr,
    inner: Option<RemoteInner>,
}

impl Remote {
    /// Create a new [`Remote`].
    pub const fn new(addr: SocketAddr) -> Remote {
        Remote { addr, inner: None }
    }
}

impl Link for Remote {
    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        self.inner = Some(RemoteInner::open(&self.addr, geometry)?);
        Ok(())
    }

    fn close(&mut self) -> Result<(), LinkError> {
        if let Some(mut inner) = self.inner.take() {
            inner.close()?;
        }
        Ok(())
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

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        if let Some(inner) = self.inner.as_mut() {
            Ok(inner.alloc_tx_buffer())
        } else {
            Err(LinkError::closed())
        }
    }
}
