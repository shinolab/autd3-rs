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

    pub fn modulation(&self) -> Vec<(Duration, u8)> {
        let segment = self.cpu.fpga().current_mod_segment();
        let cycle = self.cpu.fpga().modulation_cycle(segment);
        let sampling_period =
            ULTRASOUND_PERIOD * self.cpu.fpga().modulation_freq_division(segment) as _;
        (0..cycle)
            .map(|i| {
                (
                    sampling_period * i as _,
                    self.cpu.fpga().modulation_at(segment, i),
                )
            })
            .collect()
    }
}
