use crate::geometry::Geometry;

use super::TxMessage;

/// A tx buffer pool for single-threaded use
#[derive(Default)]
pub struct TxBufferPoolSync {
    num_devices: usize,
    buffer: Option<Vec<TxMessage>>,
}

impl TxBufferPoolSync {
    /// Creates a new [`TxBufferPoolSync`].
    pub const fn new() -> Self {
        Self {
            num_devices: 0,
            buffer: None,
        }
    }

    /// Sets the number of devices.
    pub fn init(&mut self, geometry: &Geometry) {
        self.num_devices = geometry.len();
    }

    /// Borrows a buffer from the pool.
    pub fn borrow(&mut self) -> Vec<TxMessage> {
        self.buffer
            .take()
            .unwrap_or_else(|| vec![TxMessage::new(); self.num_devices])
    }

    /// Returns a buffer to the pool.
    pub fn return_buffer(&mut self, buffer: Vec<TxMessage>) {
        assert_eq!(buffer.len(), self.num_devices);
        self.buffer = Some(buffer);
    }
}
