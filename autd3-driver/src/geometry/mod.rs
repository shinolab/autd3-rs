/*
 * File: mod.rs
 * Project: geometry
 * Created Date: 04/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

pub(crate) mod device;
mod rotation;
mod transducer;

use crate::defined::float;

pub type Vector3 = nalgebra::Vector3<float>;
pub type UnitVector3 = nalgebra::UnitVector3<float>;
pub type Vector4 = nalgebra::Vector4<float>;
pub type Quaternion = nalgebra::Quaternion<float>;
pub type UnitQuaternion = nalgebra::UnitQuaternion<float>;
pub type Matrix3 = nalgebra::Matrix3<float>;
pub type Matrix4 = nalgebra::Matrix4<float>;
pub type Affine = nalgebra::Affine3<float>;

pub use device::*;
pub use rotation::*;
pub use transducer::*;

use std::ops::{Deref, DerefMut};

pub struct Geometry {
    pub(crate) devices: Vec<Device>,
}

impl Geometry {
    #[doc(hidden)]
    pub fn new(devices: Vec<Device>) -> Geometry {
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
        self.devices.iter().map(|d| d.center()).sum::<Vector3>() / self.devices.len() as float
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
    pub fn set_sound_speed(&mut self, c: float) {
        self.devices_mut().for_each(|dev| dev.sound_speed = c);
    }

    /// Set speed of sound of enabled devices from temperature
    /// This is equivalent to `set_sound_speed_from_temp_with(temp, 1.4, 8.314463, 28.9647e-3)`
    ///
    /// # Arguments
    ///
    /// * `temp` - Temperature in Celsius
    ///
    pub fn set_sound_speed_from_temp(&mut self, temp: float) {
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
    pub fn set_sound_speed_from_temp_with(&mut self, temp: float, k: float, r: float, m: float) {
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

#[cfg(test)]
pub mod tests {
    use crate::defined::MILLIMETER;

    use super::*;

    macro_rules! assert_approx_eq_vec3 {
        ($a:expr, $b:expr) => {
            assert_approx_eq::assert_approx_eq!($a.x, $b.x, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.y, $b.y, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.z, $b.z, 1e-3);
        };
    }

    fn create_device(idx: usize, n: usize) -> Device {
        Device::new(
            idx,
            (0..n)
                .map(|i| Transducer::new(i, Vector3::zeros(), UnitQuaternion::identity()))
                .collect(),
        )
    }

    pub fn create_geometry(n: usize, num_trans_in_unit: usize) -> Geometry {
        Geometry::new(
            (0..n)
                .map(|i| create_device(i, num_trans_in_unit))
                .collect(),
        )
    }

    #[test]
    fn geometry_num_devices() {
        let geometry = Geometry::new(vec![create_device(0, 249)]);
        assert_eq!(geometry.num_devices(), 1);

        let geometry = Geometry::new(vec![create_device(0, 249), create_device(0, 249)]);
        assert_eq!(geometry.num_devices(), 2);
    }

    #[test]
    fn geometry_num_transducers() {
        let geometry = Geometry::new(vec![create_device(0, 249)]);
        assert_eq!(geometry.num_transducers(), 249);

        let geometry = Geometry::new(vec![create_device(0, 249), create_device(0, 249)]);
        assert_eq!(geometry.num_transducers(), 249 * 2);
    }

    #[test]
    fn center() {
        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device0 = Device::new(0, transducers);

        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.) + Vector3::new(10., 20., 30.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device1 = Device::new(1, transducers);

        let geometry = Geometry::new(vec![device0, device1]);

        let expect = geometry.iter().map(|dev| dev.center()).sum::<Vector3>() / 2.0;

        assert_approx_eq_vec3!(geometry.center(), expect);
    }

    #[test]
    fn set_sound_speed() {
        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device0 = Device::new(0, transducers);

        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.) + Vector3::new(10., 20., 30.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device1 = Device::new(1, transducers);

        let mut geometry = Geometry::new(vec![device0, device1]);

        geometry.set_sound_speed_from_temp(15.);
        geometry.iter().for_each(|dev| {
            assert_approx_eq::assert_approx_eq!(
                dev.sound_speed,
                340.29527186788846e3 * MILLIMETER,
                1e-3
            );
        });

        geometry.set_sound_speed(340e3 * MILLIMETER);
        geometry.iter().for_each(|dev| {
            assert_eq!(dev.sound_speed, 340e3 * MILLIMETER);
        });
    }

    #[test]
    fn deref() {
        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device0 = Device::new(0, transducers);

        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.) + Vector3::new(10., 20., 30.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device1 = Device::new(1, transducers);

        let geometry = Geometry::new(vec![device0, device1]);
        let _ = geometry[0].sound_speed;
        let _ = geometry[1].sound_speed;
    }

    #[test]
    fn deref_mut() {
        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device0 = Device::new(0, transducers);

        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.) + Vector3::new(10., 20., 30.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device1 = Device::new(1, transducers);

        let mut geometry = Geometry::new(vec![device0, device1]);
        geometry[0].sound_speed = 0.;
        geometry[1].sound_speed = 0.;
    }

    #[test]
    fn into_iter() {
        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device0 = Device::new(0, transducers);

        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.) + Vector3::new(10., 20., 30.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device1 = Device::new(1, transducers);

        let geometry = Geometry::new(vec![device0, device1]);
        for dev in &geometry {
            let _ = dev.sound_speed;
        }
    }

    #[test]
    fn into_iter_mut() {
        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device0 = Device::new(0, transducers);

        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as float, y as float, 0.) + Vector3::new(10., 20., 30.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let device1 = Device::new(1, transducers);

        let mut geometry = Geometry::new(vec![device0, device1]);

        for dev in &mut geometry {
            dev.sound_speed = 0.;
        }
    }
}
