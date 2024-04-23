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
                tx.all_data().as_ptr(),
                self.buf.as_mut_ptr(),
                tx.all_data().len(),
            );
        }
    }

    pub fn clear(&mut self) {
        self.buf.fill(0x00);
    }
}
