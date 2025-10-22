#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides a link for remote server or [`AUTD3 Simulator`].
//!
//! [`AUTD3 Simulator`]: https://github.com/shinolab/autd3-server

mod link;

pub use link::{Remote, RemoteOption};

// # Protocol Specification
//
// ## Message Types
//
// - `0x01`: Configure Geometry
// - `0x02`: Update Geometry
// - `0x03`: Send Data
// - `0x04`: Read Data
// - `0x05`: Close
//
// ## Response Status Codes
//
// - `0x00`: OK
// - `0xFF`: Error
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
// Response (Success):
// - 1 byte: status (0x00 = OK)
//
// Response (Error):
// - 1 byte: status (0xFF = Error)
// - 4 bytes: error message length (u32, little-endian)
// - N bytes: error message (UTF-8 string)
//
// ### Send Data
// Request:
// - 1 byte: message type (0x03)
// - 4 bytes: number of devices (u32, little-endian)
// - Raw TxMessage data for each device
//
// Response (Success):
// - 1 byte: status (0x00 = OK)
//
// Response (Error):
// - 1 byte: status (0xFF = Error)
// - 4 bytes: error message length (u32, little-endian)
// - N bytes: error message (UTF-8 string)
//
// ### Read Data
// Request:
// - 1 byte: message type (0x04)
//
// Response (Success):
// - 1 byte: status (0x00 = OK)
// - 4 bytes: number of devices (u32, little-endian)
// - Raw RxMessage data for each device
//
// Response (Error):
// - 1 byte: status (0xFF = Error)
// - 4 bytes: error message length (u32, little-endian)
// - N bytes: error message (UTF-8 string)
//
// ### Close
// Request:
// - 1 byte: message type (0x05)
//
// Response (Success):
// - 1 byte: status (0x00 = OK)
//
// Response (Error):
// - 1 byte: status (0xFF = Error)
// - 4 bytes: error message length (u32, little-endian)
// - N bytes: error message (UTF-8 string)

pub(crate) const MSG_CONFIG_GEOMETRY: u8 = 0x01;
pub(crate) const MSG_UPDATE_GEOMETRY: u8 = 0x02;
pub(crate) const MSG_SEND_DATA: u8 = 0x03;
pub(crate) const MSG_READ_DATA: u8 = 0x04;
pub(crate) const MSG_CLOSE: u8 = 0x05;

pub(crate) const MSG_OK: u8 = 0x00;
pub(crate) const MSG_ERROR: u8 = 0xFF;
