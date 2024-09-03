use autd3_driver::{
    firmware::fpga::Drive,
    geometry::{UnitQuaternion, Vector3},
};

use crate::Calc;

impl Calc {
    pub fn gain(&self) -> Vec<(Vector3, UnitQuaternion, Drive)> {
        self.sub_devices
            .iter()
            .flat_map(|sub_device| sub_device.gain())
            .collect()
    }
}
