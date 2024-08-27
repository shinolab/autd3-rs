use autd3_driver::{
    ethercat::{EC_INPUT_FRAME_SIZE, EC_OUTPUT_FRAME_SIZE},
    firmware::cpu::{RxMessage, TxDatagram},
};

pub struct IOMap {
    buf: Vec<u8>,
    num_devices: usize,
}

impl IOMap {
    pub fn new(num_devices: usize) -> Self {
        let size = (EC_OUTPUT_FRAME_SIZE + EC_INPUT_FRAME_SIZE) * num_devices;
        Self {
            buf: vec![0x00; size],
            num_devices,
        }
    }

    pub fn data(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    pub fn input(&self) -> *const RxMessage {
        unsafe {
            self.buf
                .as_ptr()
                .add(self.num_devices * EC_OUTPUT_FRAME_SIZE) as *const _
        }
    }

    pub fn copy_from(&mut self, tx: &TxDatagram) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                tx.as_ptr() as *const _,
                self.buf.as_mut_ptr(),
                tx.total_len(),
            );
        }
    }

    pub fn clear(&mut self) {
        self.buf.fill(0x00);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iomap() {
        let mut iomap = IOMap::new(1);
        let mut tx = TxDatagram::new(1);
        let payload_size = tx[0].payload.len();
        tx[0].header.msg_id = 0x01;
        tx[0].header.slot_2_offset = 0x0302;
        tx[0].payload[0] = 0x04;
        tx[0].payload[payload_size - 1] = 5;

        iomap.copy_from(&tx);

        unsafe {
            assert_eq!(iomap.data().read(), 0x01);
            assert_eq!(iomap.data().add(1).read(), 0x00);
            assert_eq!(iomap.data().add(2).read(), 0x02);
            assert_eq!(iomap.data().add(3).read(), 0x03);
            assert_eq!(iomap.data().add(3 + 1).read(), 0x04);
            assert_eq!(iomap.data().add(3 + payload_size).read(), 0x05);
        }
    }
}
