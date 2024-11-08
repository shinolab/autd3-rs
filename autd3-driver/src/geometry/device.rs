use std::f32::consts::PI;

use autd3_derive::Builder;
use bvh::aabb::Aabb;
use derive_more::{Deref, IntoIterator};

use crate::defined::{METER, ULTRASOUND_FREQ};

use super::{Quaternion, Transducer, UnitQuaternion, Vector3};

#[derive(Builder, Deref, IntoIterator)]
pub struct Device {
    idx: u16,
    #[deref]
    #[into_iterator(ref)]
    transducers: Vec<Transducer>,
    pub enable: bool,
    pub sound_speed: f32,
    #[get(ref)]
    rotation: UnitQuaternion,
    #[get(ref)]
    center: Vector3,
    #[get(ref)]
    x_direction: Vector3,
    #[get(ref)]
    y_direction: Vector3,
    #[get(ref)]
    axial_direction: Vector3,
    #[get(ref)]
    inv: nalgebra::Isometry3<f32>,
    #[get(ref)]
    aabb: Aabb<f32, 3>,
}

impl Device {
    fn init(&mut self) {
        self.center = self
            .transducers
            .iter()
            .map(|tr| tr.position())
            .sum::<Vector3>()
            / self.transducers.len() as f32;
        self.x_direction = Self::get_direction(Vector3::x(), &self.rotation);
        self.y_direction = Self::get_direction(Vector3::y(), &self.rotation);
        self.axial_direction = if cfg!(feature = "left_handed") {
            Self::get_direction(-Vector3::z(), &self.rotation) // GRCOV_EXCL_LINE
        } else {
            Self::get_direction(Vector3::z(), &self.rotation)
        };
        self.inv = (nalgebra::Translation3::<f32>::from(*self.transducers[0].position())
            * self.rotation)
            .inverse();
        self.aabb = self.transducers.iter().fold(Aabb::empty(), |aabb, tr| {
            aabb.grow(&nalgebra::Point3 {
                coords: *tr.position(),
            })
        });
    }

    #[doc(hidden)]
    pub fn new(idx: u16, rot: UnitQuaternion, transducers: Vec<Transducer>) -> Self {
        let mut dev = Self {
            idx,
            transducers,
            enable: true,
            sound_speed: 340.0 * METER,
            rotation: rot,
            center: Vector3::zeros(),
            x_direction: Vector3::zeros(),
            y_direction: Vector3::zeros(),
            axial_direction: Vector3::zeros(),
            inv: nalgebra::Isometry3::identity(),
            aabb: Aabb::empty(),
        };
        dev.init();
        dev
    }

    pub const fn idx(&self) -> usize {
        self.idx as _
    }

    pub fn num_transducers(&self) -> usize {
        self.transducers.len()
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
        self.init();
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
    fn into_device(self, dev_idx: u16) -> Device;
}

impl IntoDevice for Device {
    fn into_device(mut self, dev_idx: u16) -> Device {
        self.idx = dev_idx;
        self
    }
}

#[cfg(test)]
pub mod tests {
    use nalgebra::Point3;
    use rand::Rng;

    use super::*;
    use crate::{
        autd3_device::AUTD3,
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
    fn idx(#[case] expect: u16) {
        assert_eq!(expect, create_device(expect, 249).idx() as _);
    }

    #[rstest::rstest]
    #[test]
    #[case(1)]
    #[case(249)]
    fn num_transducers(#[case] n: u8) {
        assert_eq!(n, create_device(0, n).num_transducers() as _);
    }

    #[test]
    fn center() {
        let device = AUTD3::new(Vector3::zeros()).into_device(0);
        let expected = device.iter().map(|t| t.position()).sum::<Vector3>() / device.len() as f32;
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
    fn inv(
        #[case] expected: Vector3,
        #[case] target: Vector3,
        #[case] origin: Vector3,
        #[case] rot: UnitQuaternion,
    ) {
        let device = AUTD3::new(origin).with_rotation(rot).into_device(0);
        assert_approx_eq_vec3!(expected, device.inv.transform_point(&Point3::from(target)));
    }

    #[test]
    fn translate_to() {
        let mut rng = rand::thread_rng();
        let origin = Vector3::new(rng.gen(), rng.gen(), rng.gen());

        let mut device = AUTD3::new(origin).into_device(0);

        let t = Vector3::new(40., 50., 60.);
        device.translate_to(t);

        AUTD3::new(t)
            .into_device(0)
            .iter()
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect.position(), tr.position());
            });
    }

    #[test]
    fn rotate_to() {
        let mut rng = rand::thread_rng();
        let mut device = AUTD3::new(Vector3::zeros())
            .with_rotation(
                UnitQuaternion::from_axis_angle(&Vector3::x_axis(), rng.gen())
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), rng.gen())
                    * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), rng.gen()),
            )
            .into_device(0);

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
        AUTD3::new(Vector3::zeros())
            .with_rotation(rot)
            .into_device(0)
            .iter()
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect.position(), tr.position());
            });
    }

    #[test]
    fn translate() {
        let mut rng = rand::thread_rng();
        let origin = Vector3::new(rng.gen(), rng.gen(), rng.gen());

        let mut device = AUTD3::new(origin).into_device(0);

        let t = Vector3::new(40., 50., 60.);
        device.translate(t);
        AUTD3::new(origin + t)
            .into_device(0)
            .iter()
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect.position(), tr.position());
            });
    }

    #[test]
    fn rotate() {
        let mut device = AUTD3::new(Vector3::zeros()).into_device(0);

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
        AUTD3::new(Vector3::zeros())
            .with_rotation(rot)
            .into_device(0)
            .iter()
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect.position(), tr.position());
            });
    }

    #[test]
    fn affine() {
        let mut device = AUTD3::new(Vector3::zeros()).into_device(0);

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

        AUTD3::new(t)
            .with_rotation(rot)
            .into_device(0)
            .iter()
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect.position(), tr.position());
            });
    }

    #[rstest::rstest]
    #[test]
    #[case(340.29525e3, 15.)]
    #[case(343.23497e3, 20.)]
    #[case(349.04013e3, 30.)]
    fn set_sound_speed_from_temp(#[case] expected: f32, #[case] temp: f32) {
        let mut device = create_device(0, 249);
        device.set_sound_speed_from_temp(temp);
        approx::assert_abs_diff_eq!(expected * mm, device.sound_speed, epsilon = 1e-3);
    }

    #[rstest::rstest]
    #[test]
    #[case(8.5, 340e3)]
    #[case(10., 400e3)]
    fn wavelength(#[case] expect: f32, #[case] c: f32) {
        let mut device = create_device(0, 249);
        device.sound_speed = c;
        approx::assert_abs_diff_eq!(expect, device.wavelength());
    }

    #[rstest::rstest]
    #[test]
    #[case(0.739_198_27, 340e3)]
    #[case(0.628_318_55, 400e3)]
    fn wavenumber(#[case] expect: f32, #[case] c: f32) {
        let mut device = create_device(0, 249);
        device.sound_speed = c;
        approx::assert_abs_diff_eq!(expect, device.wavenumber());
    }

    #[test]
    fn into_device() {
        let mut rng = rand::thread_rng();
        let t = Vector3::new(rng.gen(), rng.gen(), rng.gen());
        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), rng.gen())
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), rng.gen())
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), rng.gen());
        let Device {
            idx: _idx,
            transducers: expect_transducers,
            enable: expect_enable,
            sound_speed: expect_sound_speed,
            rotation: expect_rotation,
            center: expect_center,
            x_direction: expect_x_direction,
            y_direction: expect_y_direction,
            axial_direction: expect_axial_direction,
            inv: expect_inv,
            aabb: expect_aabb,
        } = AUTD3::new(t).with_rotation(rot).into_device(0);
        let dev = AUTD3::new(t)
            .with_rotation(rot)
            .into_device(0)
            .into_device(1);
        assert_eq!(1, dev.idx());
        assert_eq!(expect_transducers, dev.transducers);
        assert_eq!(expect_enable, dev.enable);
        assert_eq!(expect_sound_speed, dev.sound_speed);
        assert_eq!(expect_rotation, dev.rotation);
        assert_eq!(expect_center, dev.center);
        assert_eq!(expect_x_direction, dev.x_direction);
        assert_eq!(expect_y_direction, dev.y_direction);
        assert_eq!(expect_axial_direction, dev.axial_direction);
        assert_eq!(expect_inv, dev.inv);
        assert_eq!(expect_aabb.min, dev.aabb.min);
        assert_eq!(expect_aabb.max, dev.aabb.max);
    }
}
