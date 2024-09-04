use std::time::Duration;

use autd3_driver::{
    defined::ULTRASOUND_PERIOD,
    firmware::fpga::Drive,
    geometry::{Device, UnitQuaternion, Vector3},
};
use autd3_firmware_emulator::CPUEmulator;

pub struct SubDevice {
    pub(crate) device: Device,
    pub(crate) cpu: CPUEmulator,
}

impl SubDevice {
    pub fn gain(&self) -> Vec<(Vector3, UnitQuaternion, Drive)> {
        self.device
            .iter()
            .zip(self.cpu.fpga().drives())
            .map(|(t, d)| (*t.position(), *self.device.rotation(), d))
            .collect()
    }

    pub fn modulation(&self) -> impl Iterator<Item = (Duration, u8)> + '_ {
        let segment = self.cpu.fpga().current_mod_segment();
        let cycle = self.cpu.fpga().modulation_cycle(segment);
        (0..cycle)
            .flat_map(move |i| {
                std::iter::repeat(self.cpu.fpga().modulation_at(segment, i))
                    .take(self.cpu.fpga().modulation_freq_division(segment) as _)
            })
            .enumerate()
            .map(|(i, v)| (ULTRASOUND_PERIOD * i as _, v))
    }
}
