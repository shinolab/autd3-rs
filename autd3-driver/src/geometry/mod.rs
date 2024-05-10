pub(crate) mod device;
mod rotation;
mod transducer;

pub type Vector3 = nalgebra::Vector3<f64>;
pub type UnitVector3 = nalgebra::UnitVector3<f64>;
pub type Vector4 = nalgebra::Vector4<f64>;
pub type Quaternion = nalgebra::Quaternion<f64>;
pub type UnitQuaternion = nalgebra::UnitQuaternion<f64>;
pub type Matrix3 = nalgebra::Matrix3<f64>;
pub type Matrix4 = nalgebra::Matrix4<f64>;
pub type Affine = nalgebra::Affine3<f64>;

pub use device::*;
pub use rotation::*;
pub use transducer::*;

use std::ops::{Deref, DerefMut};

pub struct Geometry {
    pub(crate) devices: Vec<Device>,
}

impl Geometry {
    #[doc(hidden)]
    pub const fn new(devices: Vec<Device>) -> Geometry {
        Self { devices }
    }

    /// Get the number of devices
    pub fn num_devices(&self) -> usize {
        self.devices.len()
    }

    /// Get the number of total transducers
    pub fn num_transducers(&self) -> usize {
        self.devices.iter().map(|dev| dev.num_transducers()).sum()
    }

    /// Get center position of all devices
    pub fn center(&self) -> Vector3 {
        self.devices.iter().map(|d| d.center()).sum::<Vector3>() / self.devices.len() as f64
    }

    /// Enumerate enabled devices
    pub fn devices(&self) -> impl Iterator<Item = &Device> {
        self.devices.iter().filter(|dev| dev.enable)
    }

    /// Enumerate enabled devices mutably
    pub fn devices_mut(&mut self) -> impl Iterator<Item = &mut Device> {
        self.devices.iter_mut().filter(|dev| dev.enable)
    }

    /// Set speed of sound of all enabled devices
    ///
    /// # Arguments
    ///
    /// * `c` - Speed of sound
    ///
    pub fn set_sound_speed(&mut self, c: f64) {
        self.devices_mut().for_each(|dev| dev.sound_speed = c);
    }

    /// Set speed of sound of enabled devices from temperature
    /// This is equivalent to `set_sound_speed_from_temp_with(temp, 1.4, 8.314463, 28.9647e-3)`
    ///
    /// # Arguments
    ///
    /// * `temp` - Temperature in Celsius
    ///
    pub fn set_sound_speed_from_temp(&mut self, temp: f64) {
        self.set_sound_speed_from_temp_with(temp, 1.4, 8.314_463, 28.9647e-3);
    }

    /// Set speed of sound of enabled devices from temperature with air parameter
    ///
    /// # Arguments
    ///
    /// * `temp` - Temperature in Celsius
    /// * `k` - Ratio of specific heat
    /// * `r` - Gas constant
    /// * `m` - Molar mass
    ///
    pub fn set_sound_speed_from_temp_with(&mut self, temp: f64, k: f64, r: f64, m: f64) {
        self.devices_mut()
            .for_each(|dev| dev.set_sound_speed_from_temp_with(temp, k, r, m));
    }
}

impl Deref for Geometry {
    type Target = [Device];

    fn deref(&self) -> &Self::Target {
        &self.devices
    }
}

impl DerefMut for Geometry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.devices
    }
}

// GRCOV_EXCL_START
impl<'a> IntoIterator for &'a Geometry {
    type Item = &'a Device;
    type IntoIter = std::slice::Iter<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.devices.iter()
    }
}

impl<'a> IntoIterator for &'a mut Geometry {
    type Item = &'a mut Device;
    type IntoIter = std::slice::IterMut<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.devices.iter_mut()
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
pub mod tests {
    use crate::{
        defined::{FREQ_40K, MILLIMETER},
        freq::Freq,
    };

    use super::*;

    macro_rules! assert_approx_eq_vec3 {
        ($a:expr, $b:expr) => {
            assert_approx_eq::assert_approx_eq!($a.x, $b.x, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.y, $b.y, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.z, $b.z, 1e-3);
        };
    }

    pub fn create_device(idx: usize, n: usize, freq: Freq<u32>) -> Device {
        Device::new(
            idx,
            (0..n)
                .map(|i| Transducer::new(i, Vector3::zeros(), UnitQuaternion::identity()))
                .collect(),
            freq,
        )
    }

    pub fn create_geometry(n: usize, num_trans_in_unit: usize, freq: Freq<u32>) -> Geometry {
        Geometry::new(
            (0..n)
                .map(|i| create_device(i, num_trans_in_unit, freq))
                .collect(),
        )
    }

    #[rstest::rstest]
    #[test]
    #[case(1, vec![create_device(0, 249, FREQ_40K)])]
    #[case(2, vec![create_device(0, 249, FREQ_40K), create_device(0, 249, FREQ_40K)])]
    fn test_num_devices(#[case] expected: usize, #[case] devices: Vec<Device>) {
        assert_eq!(expected, Geometry::new(devices).num_devices());
    }

    #[rstest::rstest]
    #[test]
    #[case(249, vec![create_device(0, 249, FREQ_40K)])]
    #[case(498, vec![create_device(0, 249, FREQ_40K), create_device(0, 249, FREQ_40K)])]
    fn test_num_transducers(#[case] expected: usize, #[case] devices: Vec<Device>) {
        assert_eq!(expected, Geometry::new(devices).num_transducers());
    }

    #[test]
    fn test_center() {
        let geometry = Geometry::new(vec![
            Device::new(
                0,
                itertools::iproduct!((0..18), (0..14))
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(
                            i,
                            10.16 * Vector3::new(x as f64, y as f64, 0.),
                            UnitQuaternion::identity(),
                        )
                    })
                    .collect::<Vec<_>>(),
                FREQ_40K,
            ),
            Device::new(
                1,
                itertools::iproduct!((0..18), (0..14))
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(
                            i,
                            10.16 * Vector3::new(x as f64, y as f64, 0.)
                                + Vector3::new(10., 20., 30.),
                            UnitQuaternion::identity(),
                        )
                    })
                    .collect::<Vec<_>>(),
                FREQ_40K,
            ),
        ]);
        let expect = geometry.iter().map(|dev| dev.center()).sum::<Vector3>()
            / geometry.num_devices() as f64;
        assert_approx_eq_vec3!(expect, geometry.center());
    }

    #[rstest::rstest]
    #[test]
    #[case(340.29527186788846e3, 15.)]
    #[case(343.23498846612807e3, 20.)]
    #[case(349.0401521469255e3, 30.)]
    fn test_set_sound_speed_from_temp(#[case] expected: f64, #[case] temp: f64) {
        let mut geometry = Geometry::new(vec![
            Device::new(
                0,
                itertools::iproduct!((0..18), (0..14))
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(
                            i,
                            10.16 * Vector3::new(x as f64, y as f64, 0.),
                            UnitQuaternion::identity(),
                        )
                    })
                    .collect::<Vec<_>>(),
                FREQ_40K,
            ),
            Device::new(
                1,
                itertools::iproduct!((0..18), (0..14))
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(
                            i,
                            10.16 * Vector3::new(x as f64, y as f64, 0.)
                                + Vector3::new(10., 20., 30.),
                            UnitQuaternion::identity(),
                        )
                    })
                    .collect::<Vec<_>>(),
                FREQ_40K,
            ),
        ]);
        geometry.set_sound_speed_from_temp(temp);
        geometry.iter().for_each(|dev| {
            assert_approx_eq::assert_approx_eq!(expected * MILLIMETER, dev.sound_speed, 1e-3);
        });
    }

    #[rstest::rstest]
    #[test]
    #[case(340.29527186788846e3)]
    #[case(343.23498846612807e3)]
    #[case(349.0401521469255e3)]
    fn test_set_sound_speed(#[case] temp: f64) {
        let mut geometry = Geometry::new(vec![
            Device::new(
                0,
                itertools::iproduct!((0..18), (0..14))
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(
                            i,
                            10.16 * Vector3::new(x as f64, y as f64, 0.),
                            UnitQuaternion::identity(),
                        )
                    })
                    .collect::<Vec<_>>(),
                FREQ_40K,
            ),
            Device::new(
                1,
                itertools::iproduct!((0..18), (0..14))
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(
                            i,
                            10.16 * Vector3::new(x as f64, y as f64, 0.)
                                + Vector3::new(10., 20., 30.),
                            UnitQuaternion::identity(),
                        )
                    })
                    .collect::<Vec<_>>(),
                FREQ_40K,
            ),
        ]);
        geometry.set_sound_speed(temp * MILLIMETER);
        geometry.iter().for_each(|dev| {
            assert_eq!(dev.sound_speed, temp * MILLIMETER);
        });
    }
}
