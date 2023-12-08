/*
 * File: device.rs
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

use std::ops::Deref;

use crate::defined::{float, METER};

use super::{Matrix3, Transducer, UnitQuaternion, Vector3};

pub struct Device {
    idx: usize,
    transducers: Vec<Transducer>,
    pub enable: bool,
    pub sound_speed: float,
    pub attenuation: float,
    inv: Matrix3,
}

impl Device {
    #[doc(hidden)]
    pub fn new(idx: usize, transducers: Vec<Transducer>) -> Self {
        let inv = Matrix3::from_columns(&[
            transducers[0].x_direction(),
            transducers[0].y_direction(),
            transducers[0].z_direction(),
        ])
        .transpose();
        Self {
            idx,
            transducers,
            enable: true,
            sound_speed: 340.0 * METER,
            attenuation: 0.0,
            inv,
        }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    /// Get the number of transducers
    pub fn num_transducers(&self) -> usize {
        self.transducers.len()
    }

    /// Get center position
    pub fn center(&self) -> Vector3 {
        self.transducers
            .iter()
            .map(|tr| tr.position())
            .sum::<Vector3>()
            / self.transducers.len() as float
    }

    pub fn to_local(&self, p: &Vector3) -> Vector3 {
        self.inv * (p - self.transducers[0].position())
    }

    /// Set positions of all transducers in the device
    pub fn translate_to(&mut self, t: Vector3) {
        let cur_pos = self.transducers[0].position();
        self.translate(t - cur_pos);
    }

    /// Set rotation of all transducers in the device
    pub fn rotate_to(&mut self, r: UnitQuaternion) {
        let cur_rot = self.transducers[0].rotation();
        self.rotate(r * cur_rot.conjugate());
    }

    /// Translate all transducers in the device
    pub fn translate(&mut self, t: Vector3) {
        self.affine(t, UnitQuaternion::identity());
    }

    /// Rorate all transducers in the device
    pub fn rotate(&mut self, r: UnitQuaternion) {
        self.affine(Vector3::zeros(), r);
    }

    /// Affine transform
    pub fn affine(&mut self, t: Vector3, r: UnitQuaternion) {
        self.transducers.iter_mut().for_each(|tr| tr.affine(t, r));
    }

    /// Set speed of sound from temperature
    /// This is equivalent to `set_sound_speed_from_temp_with(temp, 1.4, 8.314463, 28.9647e-3)`
    ///
    /// # Arguments
    ///
    /// * `temp` - Temperature in Celsius
    ///
    pub fn set_sound_speed_from_temp(&mut self, temp: float) {
        self.set_sound_speed_from_temp_with(temp, 1.4, 8.314_463, 28.9647e-3);
    }

    /// Set speed of sound from temperature with air parameter
    ///
    /// # Arguments
    ///
    /// * `temp` - Temperature in Celsius
    /// * `k` - Ratio of specific heat
    /// * `r` - Gas constant
    /// * `m` - Molar mass
    ///
    pub fn set_sound_speed_from_temp_with(&mut self, temp: float, k: float, r: float, m: float) {
        self.sound_speed = (k * r * (273.15 + temp) / m).sqrt() * METER;
    }
}

impl Deref for Device {
    type Target = [Transducer];

    fn deref(&self) -> &Self::Target {
        &self.transducers
    }
}

impl<'a> IntoIterator for &'a Device {
    type Item = &'a Transducer;
    type IntoIter = std::slice::Iter<'a, Transducer>;

    fn into_iter(self) -> Self::IntoIter {
        self.transducers.iter()
    }
}

pub trait IntoDevice {
    fn into_device(self, dev_idx: usize) -> Device;
}

#[cfg(test)]
pub mod tests {
    use crate::{
        defined::{MILLIMETER, PI},
        geometry::Transducer,
    };

    use super::*;

    macro_rules! assert_approx_eq_vec3 {
        ($a:expr, $b:expr) => {
            assert_approx_eq::assert_approx_eq!($a.x, $b.x, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.y, $b.y, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.z, $b.z, 1e-3);
        };
    }

    macro_rules! assert_approx_eq_quat {
        ($a:expr, $b:expr) => {
            assert_approx_eq::assert_approx_eq!($a.w, $b.w, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.i, $b.i, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.j, $b.j, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.k, $b.k, 1e-3);
        };
    }

    pub fn create_device(idx: usize, n: usize) -> Device {
        Device::new(
            idx,
            (0..n)
                .map(|i| Transducer::new(i, Vector3::zeros(), UnitQuaternion::identity()))
                .collect(),
        )
    }

    #[test]
    fn device_idx() {
        let device = create_device(0, 249);
        assert_eq!(device.idx(), 0);

        let device = create_device(1, 249);
        assert_eq!(device.idx(), 1);
    }

    #[test]
    fn device_num_transducers() {
        let device = create_device(0, 249);
        assert_eq!(device.num_transducers(), 249);
    }

    #[test]
    fn device_center() {
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

        let expected =
            transducers.iter().map(|t| t.position()).sum::<Vector3>() / transducers.len() as float;

        let device = Device::new(0, transducers);

        assert_approx_eq_vec3!(device.center(), expected);
    }

    #[test]
    fn set_sound_speed_from_temp() {
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

        let mut device = Device::new(0, transducers);

        device.set_sound_speed_from_temp(15.);
        assert_approx_eq::assert_approx_eq!(
            device.sound_speed,
            340.29527186788846e3 * MILLIMETER,
            1e-3
        );
    }

    #[test]
    fn device_to_local() {
        {
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

            let device = Device::new(0, transducers);

            let p = Vector3::new(10., 20., 30.);

            assert_approx_eq_vec3!(device.to_local(&p), p);
        }

        {
            let p = Vector3::new(10., 20., 30.);

            let transducers = itertools::iproduct!((0..18), (0..14))
                .enumerate()
                .map(|(i, (y, x))| {
                    Transducer::new(
                        i,
                        10.16 * Vector3::new(x as float, y as float, 0.) + p,
                        UnitQuaternion::identity(),
                    )
                })
                .collect::<Vec<_>>();

            let device = Device::new(0, transducers);

            assert_approx_eq_vec3!(device.to_local(&p), Vector3::zeros());
        }

        {
            let q = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.);

            let transducers = itertools::iproduct!((0..18), (0..14))
                .enumerate()
                .map(|(i, (y, x))| {
                    Transducer::new(i, 10.16 * Vector3::new(x as float, y as float, 0.), q)
                })
                .collect::<Vec<_>>();

            let device = Device::new(0, transducers);

            let p = Vector3::new(10., 20., 30.);

            assert_approx_eq_vec3!(device.to_local(&p), Vector3::new(p.y, -p.x, p.z));
        }

        {
            let p = Vector3::new(10., 20., 30.);
            let q = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.);

            let transducers = itertools::iproduct!((0..18), (0..14))
                .enumerate()
                .map(|(i, (y, x))| {
                    Transducer::new(i, 10.16 * Vector3::new(x as float, y as float, 0.) + p, q)
                })
                .collect::<Vec<_>>();

            let device = Device::new(0, transducers);

            assert_approx_eq_vec3!(device.to_local(&p), Vector3::new(0., 0., 0.));

            let d = Vector3::new(40., 50., 60.);

            assert_approx_eq_vec3!(device.to_local(&(p + d)), Vector3::new(d.x, d.z, -d.y));
        }
    }

    #[test]
    fn device_translate_to() {
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

        let mut device = Device::new(0, transducers);
        device.translate(Vector3::new(10., 20., 30.));

        let t = Vector3::new(40., 50., 60.);
        device.translate_to(t);

        itertools::iproduct!((0..18), (0..14))
            .map(|(y, x)| 10.16 * Vector3::new(x as float, y as float, 0.) + t)
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[test]
    fn device_rotate_to() {
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

        let mut device = Device::new(0, transducers);
        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), 0.);
        device.rotate(rot);

        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.);
        device.rotate_to(rot);

        let expect_x = Vector3::new(0., 1., 0.);
        let expect_y = Vector3::new(-1., 0., 0.);
        let expect_z = Vector3::new(0., 0., 1.);
        device.iter().for_each(|tr| {
            assert_approx_eq_quat!(rot, tr.rotation());
            assert_approx_eq_vec3!(expect_x, tr.x_direction());
            assert_approx_eq_vec3!(expect_y, tr.y_direction());
            assert_approx_eq_vec3!(expect_z, tr.z_direction());
        });
        itertools::iproduct!((0..18), (0..14))
            .map(|(y, x)| 10.16 * Vector3::new(-y as float, x as float, 0.))
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[test]
    fn device_translate() {
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

        let mut device = Device::new(0, transducers);

        let t = Vector3::new(40., 50., 60.);
        device.translate(t);

        itertools::iproduct!((0..18), (0..14))
            .map(|(y, x)| 10.16 * Vector3::new(x as float, y as float, 0.) + t)
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[test]
    fn device_rotate() {
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

        let mut device = Device::new(0, transducers);

        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.);
        device.rotate(rot);
        let expect_x = Vector3::new(0., 1., 0.);
        let expect_y = Vector3::new(-1., 0., 0.);
        let expect_z = Vector3::new(0., 0., 1.);
        device.iter().for_each(|tr| {
            assert_approx_eq_quat!(rot, tr.rotation());
            assert_approx_eq_vec3!(expect_x, tr.x_direction());
            assert_approx_eq_vec3!(expect_y, tr.y_direction());
            assert_approx_eq_vec3!(expect_z, tr.z_direction());
        });
        itertools::iproduct!((0..18), (0..14))
            .map(|(y, x)| 10.16 * Vector3::new(-y as float, x as float, 0.))
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });

        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), 0.);
        device.rotate(rot);
        let expect_x = Vector3::new(0., 0., 1.);
        let expect_y = Vector3::new(-1., 0., 0.);
        let expect_z = Vector3::new(0., -1., 0.);
        device.iter().for_each(|tr| {
            assert_approx_eq_vec3!(expect_x, tr.x_direction());
            assert_approx_eq_vec3!(expect_y, tr.y_direction());
            assert_approx_eq_vec3!(expect_z, tr.z_direction());
        });
        itertools::iproduct!((0..18), (0..14))
            .map(|(y, x)| 10.16 * Vector3::new(-y as float, 0., x as float))
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[test]
    fn affine() {
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

        let mut device = Device::new(0, transducers);

        let t = Vector3::new(40., 50., 60.);
        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.);
        device.affine(t, rot);

        let expect_x = Vector3::new(0., 1., 0.);
        let expect_y = Vector3::new(-1., 0., 0.);
        let expect_z = Vector3::new(0., 0., 1.);
        device.iter().for_each(|tr| {
            assert_approx_eq_quat!(rot, tr.rotation());
            assert_approx_eq_vec3!(expect_x, tr.x_direction());
            assert_approx_eq_vec3!(expect_y, tr.y_direction());
            assert_approx_eq_vec3!(expect_z, tr.z_direction());
        });

        itertools::iproduct!((0..18), (0..14))
            .map(|(y, x)| 10.16 * Vector3::new(-y as float, x as float, 0.) + t)
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
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

        let device = Device::new(0, transducers);

        for tr in &device {
            let _ = tr.idx();
        }
    }
}
