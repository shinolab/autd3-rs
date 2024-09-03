use std::f32::consts::PI;

use autd3_derive::Builder;
use derive_more::{Deref, IntoIterator};

use crate::defined::{METER, ULTRASOUND_FREQ};

use super::{Quaternion, Transducer, UnitQuaternion, Vector3};

#[derive(Builder, Deref, IntoIterator)]
pub struct Device {
    #[get]
    idx: usize,
    #[deref]
    #[into_iterator(ref)]
    transducers: Vec<Transducer>,
    pub enable: bool,
    pub sound_speed: f32,
    #[get(ref)]
    rotation: UnitQuaternion,
    #[get(ref)]
    x_direction: Vector3,
    #[get(ref)]
    y_direction: Vector3,
    #[get(ref)]
    axial_direction: Vector3,
    inv: nalgebra::Rotation<f32, 3>,
}

impl Device {
    #[doc(hidden)]
    pub fn new(idx: usize, rot: UnitQuaternion, transducers: Vec<Transducer>) -> Self {
        Self {
            idx,
            transducers,
            enable: true,
            sound_speed: 340.0 * METER,
            rotation: rot,
            x_direction: Self::get_direction(Vector3::x(), &rot),
            y_direction: Self::get_direction(Vector3::y(), &rot),
            #[cfg(feature = "left_handed")]
            axial_direction: Self::get_direction(-Vector3::z(), &rot),
            #[cfg(not(feature = "left_handed"))]
            axial_direction: Self::get_direction(Vector3::z(), &rot),
            inv: rot.inverse().to_rotation_matrix(),
        }
    }

    pub fn num_transducers(&self) -> usize {
        self.transducers.len()
    }

    pub fn center(&self) -> Vector3 {
        self.transducers
            .iter()
            .map(|tr| tr.position())
            .sum::<Vector3>()
            / self.transducers.len() as f32
    }

    pub fn to_local(&self, p: &Vector3) -> Vector3 {
        self.inv * (p - self.transducers[0].position())
    }

    pub fn translate_to(&mut self, t: Vector3) {
        let cur_pos = self.transducers[0].position();
        self.translate(t - cur_pos);
    }

    pub fn rotate_to(&mut self, r: UnitQuaternion) {
        let cur_rot = self.rotation;
        self.rotate(r * cur_rot.conjugate());
    }

    pub fn translate(&mut self, t: Vector3) {
        self.affine(t, UnitQuaternion::identity());
    }

    pub fn rotate(&mut self, r: UnitQuaternion) {
        self.affine(Vector3::zeros(), r);
    }

    pub fn affine(&mut self, t: Vector3, r: UnitQuaternion) {
        self.transducers.iter_mut().for_each(|tr| tr.affine(t, r));
        self.rotation = r * self.rotation;
        self.inv = self.rotation.inverse().to_rotation_matrix();
        self.x_direction = Self::get_direction(Vector3::x(), &self.rotation);
        self.y_direction = Self::get_direction(Vector3::y(), &self.rotation);
        self.axial_direction = if cfg!(feature = "left_handed") {
            Self::get_direction(-Vector3::z(), &self.rotation) // GRCOV_EXCL_LINE
        } else {
            Self::get_direction(Vector3::z(), &self.rotation)
        };
    }

    pub fn set_sound_speed_from_temp(&mut self, temp: f32) {
        self.set_sound_speed_from_temp_with(temp, 1.4, 8.314_463, 28.9647e-3);
    }

    pub fn set_sound_speed_from_temp_with(&mut self, temp: f32, k: f32, r: f32, m: f32) {
        self.sound_speed = (k * r * (273.15 + temp) / m).sqrt() * METER;
    }

    pub fn wavelength(&self) -> f32 {
        self.sound_speed / ULTRASOUND_FREQ.hz() as f32
    }

    pub fn wavenumber(&self) -> f32 {
        2.0 * PI * ULTRASOUND_FREQ.hz() as f32 / self.sound_speed
    }

    fn get_direction(dir: Vector3, rotation: &UnitQuaternion) -> Vector3 {
        let dir: UnitQuaternion = UnitQuaternion::from_quaternion(Quaternion::from_imag(dir));
        (rotation * dir * rotation.conjugate()).imag().normalize()
    }
}

pub trait IntoDevice {
    fn into_device(self, dev_idx: usize, global_idx_offset: usize) -> Device;
}

#[cfg(test)]
pub mod tests {
    use rand::Rng;

    use super::*;
    use crate::{
        defined::{mm, PI},
        geometry::tests::create_device,
    };

    macro_rules! assert_approx_eq_vec3 {
        ($a:expr, $b:expr) => {
            approx::assert_abs_diff_eq!($a.x, $b.x, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.y, $b.y, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.z, $b.z, epsilon = 1e-3);
        };
    }

    macro_rules! assert_approx_eq_quat {
        ($a:expr, $b:expr) => {
            approx::assert_abs_diff_eq!($a.w, $b.w, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.i, $b.i, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.j, $b.j, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.k, $b.k, epsilon = 1e-3);
        };
    }

    #[rstest::rstest]
    #[test]
    #[case(0)]
    #[case(1)]
    #[cfg_attr(miri, ignore)]
    fn test_idx(#[case] idx: usize) {
        assert_eq!(idx, create_device(idx, 249).idx());
    }

    #[rstest::rstest]
    #[test]
    #[case(1)]
    #[case(249)]
    #[cfg_attr(miri, ignore)]
    fn test_num_transducers(#[case] n: usize) {
        assert_eq!(n, create_device(0, n).num_transducers());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_center() {
        let transducers = itertools::iproduct!(0..18, 0..14)
            .enumerate()
            .map(|(i, (y, x))| Transducer::new(i, i, 10.16 * Vector3::new(x as f32, y as f32, 0.)))
            .collect::<Vec<_>>();
        let expected =
            transducers.iter().map(|t| t.position()).sum::<Vector3>() / transducers.len() as f32;
        let device = Device::new(0, UnitQuaternion::identity(), transducers);
        assert_approx_eq_vec3!(expected, device.center());
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Vector3::new(10., 20., 30.),
        Vector3::new(10., 20., 30.),
        Vector3::zeros(),
        UnitQuaternion::identity()
    )]
    #[case(
        Vector3::zeros(),
        Vector3::new(10., 20., 30.),
        Vector3::new(10., 20., 30.),
        UnitQuaternion::identity()
    )]
    #[case(
        Vector3::new(20., -10., 30.),
        Vector3::new(10., 20., 30.),
        Vector3::zeros(),
        UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.)
    )]
    #[case(
        Vector3::new(30., 30., -30.),
        Vector3::new(40., 50., 60.),
        Vector3::new(10., 20., 30.),
        UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.)
    )]
    #[cfg_attr(miri, ignore)]
    fn test_to_local(
        #[case] expected: Vector3,
        #[case] target: Vector3,
        #[case] origin: Vector3,
        #[case] quat: UnitQuaternion,
    ) {
        let device = Device::new(
            0,
            quat,
            itertools::iproduct!(0..18, 0..14)
                .enumerate()
                .map(|(i, (y, x))| {
                    Transducer::new(i, i, origin + 10.16 * Vector3::new(x as f32, y as f32, 0.))
                })
                .collect::<Vec<_>>(),
        );
        assert_approx_eq_vec3!(expected, device.to_local(&target));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_translate_to() {
        let mut rng = rand::thread_rng();
        let origin = Vector3::new(rng.gen(), rng.gen(), rng.gen());

        let transducers = itertools::iproduct!(0..18, 0..14)
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(i, i, origin + 10.16 * Vector3::new(x as f32, y as f32, 0.))
            })
            .collect::<Vec<_>>();

        let mut device = Device::new(0, UnitQuaternion::identity(), transducers);

        let t = Vector3::new(40., 50., 60.);
        device.translate_to(t);

        itertools::iproduct!(0..18, 0..14)
            .map(|(y, x)| 10.16 * Vector3::new(x as f32, y as f32, 0.) + t)
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rotate_to() {
        let mut device = {
            let mut device = Device::new(
                0,
                UnitQuaternion::identity(),
                itertools::iproduct!(0..18, 0..14)
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(i, i, 10.16 * Vector3::new(x as f32, y as f32, 0.))
                    })
                    .collect::<Vec<_>>(),
            );
            let mut rng = rand::thread_rng();
            let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), rng.gen())
                * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), rng.gen())
                * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), rng.gen());
            device.rotate(rot);
            device
        };

        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.);
        device.rotate_to(rot);

        let expect_x = Vector3::new(0., 1., 0.);
        let expect_y = Vector3::new(-1., 0., 0.);
        let expect_z = if cfg!(feature = "left_handed") {
            Vector3::new(0., 0., -1.) // GRCOV_EXCL_LINE
        } else {
            Vector3::new(0., 0., 1.)
        };
        assert_approx_eq_quat!(rot, device.rotation());
        assert_approx_eq_vec3!(expect_x, device.x_direction());
        assert_approx_eq_vec3!(expect_y, device.y_direction());
        assert_approx_eq_vec3!(expect_z, device.axial_direction());
        itertools::iproduct!(0..18, 0..14)
            .map(|(y, x)| 10.16 * Vector3::new(-y as f32, x as f32, 0.))
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_translate() {
        let mut rng = rand::thread_rng();
        let origin = Vector3::new(rng.gen(), rng.gen(), rng.gen());

        let transducers = itertools::iproduct!(0..18, 0..14)
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(i, i, origin + 10.16 * Vector3::new(x as f32, y as f32, 0.))
            })
            .collect::<Vec<_>>();

        let mut device = Device::new(0, UnitQuaternion::identity(), transducers.clone());

        let t = Vector3::new(40., 50., 60.);
        device.translate(t);
        transducers
            .iter()
            .zip(device.iter())
            .for_each(|(orig, tr)| {
                assert_approx_eq_vec3!(orig.position() + t, tr.position());
            });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_rotate() {
        let transducers = itertools::iproduct!(0..18, 0..14)
            .enumerate()
            .map(|(i, (y, x))| Transducer::new(i, i, 10.16 * Vector3::new(x as f32, y as f32, 0.)))
            .collect::<Vec<_>>();

        let mut device = Device::new(0, UnitQuaternion::identity(), transducers);

        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.);
        device.rotate(rot);
        let expect_x = Vector3::new(0., 1., 0.);
        let expect_y = Vector3::new(-1., 0., 0.);
        let expect_z = if cfg!(feature = "left_handed") {
            Vector3::new(0., 0., -1.) // GRCOV_EXCL_LINE
        } else {
            Vector3::new(0., 0., 1.)
        };
        assert_approx_eq_quat!(rot, device.rotation());
        assert_approx_eq_vec3!(expect_x, device.x_direction());
        assert_approx_eq_vec3!(expect_y, device.y_direction());
        assert_approx_eq_vec3!(expect_z, device.axial_direction());
        itertools::iproduct!(0..18, 0..14)
            .map(|(y, x)| 10.16 * Vector3::new(-y as f32, x as f32, 0.))
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_affine() {
        let transducers = itertools::iproduct!(0..18, 0..14)
            .enumerate()
            .map(|(i, (y, x))| Transducer::new(i, i, 10.16 * Vector3::new(x as f32, y as f32, 0.)))
            .collect::<Vec<_>>();

        let mut device = Device::new(0, UnitQuaternion::identity(), transducers);

        let t = Vector3::new(40., 50., 60.);
        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.);
        device.affine(t, rot);

        let expect_x = Vector3::new(0., 1., 0.);
        let expect_y = Vector3::new(-1., 0., 0.);
        let expect_z = if cfg!(feature = "left_handed") {
            Vector3::new(0., 0., -1.) // GRCOV_EXCL_LINE
        } else {
            Vector3::new(0., 0., 1.)
        };
        assert_approx_eq_quat!(rot, device.rotation());
        assert_approx_eq_vec3!(expect_x, device.x_direction());
        assert_approx_eq_vec3!(expect_y, device.y_direction());
        assert_approx_eq_vec3!(expect_z, device.axial_direction());

        itertools::iproduct!(0..18, 0..14)
            .map(|(y, x)| 10.16 * Vector3::new(-y as f32, x as f32, 0.) + t)
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[rstest::rstest]
    #[test]
    #[case(340.29525e3, 15.)]
    #[case(343.23497e3, 20.)]
    #[case(349.04013e3, 30.)]
    #[cfg_attr(miri, ignore)]
    fn test_set_sound_speed_from_temp(#[case] expected: f32, #[case] temp: f32) {
        let mut device = create_device(0, 249);
        device.set_sound_speed_from_temp(temp);
        approx::assert_abs_diff_eq!(expected * mm, device.sound_speed, epsilon = 1e-3);
    }

    #[rstest::rstest]
    #[test]
    #[case(8.5, 340e3)]
    #[case(10., 400e3)]
    #[cfg_attr(miri, ignore)]
    fn wavelength(#[case] expect: f32, #[case] c: f32) {
        let mut device = create_device(0, 249);
        device.sound_speed = c;
        approx::assert_abs_diff_eq!(expect, device.wavelength());
    }

    #[allow(unused_variables)]
    #[rstest::rstest]
    #[test]
    #[case(0.739_198_27, 340e3)]
    #[case(0.628_318_55, 400e3)]
    #[cfg_attr(miri, ignore)]
    fn wavenumber(#[case] expect: f32, #[case] c: f32) {
        let mut device = create_device(0, 249);
        device.sound_speed = c;
        approx::assert_abs_diff_eq!(expect, device.wavenumber());
    }
}
