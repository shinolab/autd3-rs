#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides a link for [`AUTD3 Simulator`].
//!
//! [`AUTD3 Simulator`]: https://github.com/shinolab/autd3-server

use autd3_core::link::{Link, LinkError, RxMessage, TxBufferPoolSync, TxMessage};

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

// # Protocol Specification
//
// This link uses a simple TCP-based binary protocol for communication with the simulator.
//
// ## Message Types
//
// - `0x01`: Configure Geometry
// - `0x02`: Update Geometry
// - `0x03`: Send Data
// - `0x04`: Read Data
// - `0x05`: Close
//
// ## Message Formats
//
// ### Configure/Update Geometry
// Request:
// - 1 byte: message type (0x01 or 0x02)
// - 4 bytes: number of devices (u32, little-endian)
// - For each device:
//   - 12 bytes: position (3x f32, little-endian)
//   - 16 bytes: rotation quaternion (w, i, j, k as f32, little-endian)
//
// Response:
// - 1 byte: status (0x00 = OK)
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

/// Message type: Configure Geometry
pub const MSG_CONFIG_GEOMETRY: u8 = 0x01;
/// Message type: Update Geometry
pub const MSG_UPDATE_GEOMETRY: u8 = 0x02;
/// Message type: Send Data
pub const MSG_SEND_DATA: u8 = 0x03;
/// Message type: Read Data
pub const MSG_READ_DATA: u8 = 0x04;
/// Message type: Close
pub const MSG_CLOSE: u8 = 0x05;

/// Status: OK
pub const MSG_OK: u8 = 0x00;

struct SimulatorInner {
    stream: TcpStream,
    last_geometry_version: usize,
    buffer_pool: TxBufferPoolSync,
}

impl SimulatorInner {
    fn open(
        addr: &SocketAddr,
        geometry: &autd3_core::geometry::Geometry,
    ) -> Result<SimulatorInner, LinkError> {
        let mut stream = TcpStream::connect(addr)
            .map_err(|e| LinkError::new(format!("Failed to connect to simulator: {e}")))?;

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
            let pos = dev.center();
            buffer.extend_from_slice(&pos.x.to_le_bytes());
            buffer.extend_from_slice(&pos.y.to_le_bytes());
            buffer.extend_from_slice(&pos.z.to_le_bytes());

            let rot = dev.rotation();
            buffer.extend_from_slice(&rot.w.to_le_bytes());
            buffer.extend_from_slice(&rot.i.to_le_bytes());
            buffer.extend_from_slice(&rot.j.to_le_bytes());
            buffer.extend_from_slice(&rot.k.to_le_bytes());
        });

        stream
            .write_all(&buffer)
            .map_err(|e| LinkError::new(format!("Failed to send geometry: {e}")))?;

        Ok(())
    }

    fn wait_response(stream: &mut TcpStream) -> Result<(), LinkError> {
        let mut status = [0u8; 1];
        stream
            .read_exact(&mut status)
            .map_err(|e| LinkError::new(format!("Failed to read response: {e}")))?;

        if status[0] == MSG_OK {
            Ok(())
        } else {
            Err(LinkError::new("Simulator returned error"))
        }
    }

    fn close(&mut self) -> Result<(), LinkError> {
        self.stream
            .write_all(&[MSG_CLOSE])
            .map_err(|e| LinkError::new(format!("Failed to send close message: {e}")))?;

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

        self.stream
            .write_all(&buffer)
            .map_err(|e| LinkError::new(format!("Failed to send data: {e}")))?;

        self.buffer_pool.return_buffer(tx);

        Self::wait_response(&mut self.stream)?;

        Ok(())
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, LinkError> {
        self.stream
            .write_all(&[MSG_READ_DATA])
            .map_err(|e| LinkError::new(format!("Failed to send read request: {e}")))?;

        let mut status = [0u8; 1];
        self.stream
            .read_exact(&mut status)
            .map_err(|e| LinkError::new(format!("Failed to read response: {e}")))?;

        if status[0] != MSG_OK {
            return Err(LinkError::new("Simulator returned error"));
        }

        let mut num_devices = [0u8; 4];
        self.stream
            .read_exact(&mut num_devices)
            .map_err(|e| LinkError::new(format!("Failed to read num_devices: {e}")))?;

        let num_devices = u32::from_le_bytes(num_devices) as usize;

        if num_devices != rx.len() {
            return Ok(false);
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

        Ok(true)
    }
}

/// A [`Link`] for [`AUTD3 Simulator`].
///
/// [`AUTD3 Simulator`]: https://github.com/shinolab/autd3-server
pub struct Simulator {
    num_devices: usize,
    addr: SocketAddr,
    inner: Option<SimulatorInner>,
}

impl Simulator {
    /// Creates a new [`Simulator`].
    #[must_use]
    pub const fn new(addr: SocketAddr) -> Simulator {
        Simulator {
            num_devices: 0,
            addr,
            inner: None,
        }
    }
}

impl Link for Simulator {
    fn open(&mut self, geometry: &autd3_core::geometry::Geometry) -> Result<(), LinkError> {
        self.inner = Some(SimulatorInner::open(&self.addr, geometry)?);
        self.num_devices = geometry.len();
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
