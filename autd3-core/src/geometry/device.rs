use std::f32::consts::PI;

use bvh::aabb::Aabb;
use derive_more::{Deref, IntoIterator};
use getset::Getters;

use crate::defined::{METER, ultrasound_freq};

use super::{Isometry, Point3, Quaternion, Transducer, UnitQuaternion, UnitVector3, Vector3};

/// An AUTD device unit.
#[derive(Getters, Deref, IntoIterator)]
pub struct Device {
    pub(crate) idx: u16,
    #[deref]
    #[into_iterator(ref)]
    pub(crate) transducers: Vec<Transducer>,
    /// enable flag
    pub enable: bool,
    /// speed of sound
    pub sound_speed: f32,
    #[getset(get = "pub")]
    /// The rotation of the device.
    rotation: UnitQuaternion,
    #[getset(get = "pub")]
    /// The center of the device.
    center: Point3,
    #[getset(get = "pub")]
    /// The x-direction of the device.
    x_direction: UnitVector3,
    #[getset(get = "pub")]
    /// The y-direction of the device.
    y_direction: UnitVector3,
    #[getset(get = "pub")]
    /// The axial direction of the device.
    axial_direction: UnitVector3,
    #[doc(hidden)]
    #[getset(get = "pub")]
    inv: Isometry,
    #[getset(get = "pub")]
    /// The Axis Aligned Bounding Box of the device.
    aabb: Aabb<f32, 3>,
}

impl Device {
    fn init(&mut self) {
        self.center = Point3::from(
            self.transducers
                .iter()
                .map(|tr| tr.position().coords)
                .sum::<Vector3>()
                / self.transducers.len() as f32,
        );
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
        self.aabb = self
            .transducers
            .iter()
            .fold(Aabb::empty(), |aabb, tr| aabb.grow(tr.position()));
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new(rot: UnitQuaternion, transducers: Vec<Transducer>) -> Self {
        let mut transducers = transducers;
        transducers.iter_mut().enumerate().for_each(|(tr_idx, tr)| {
            tr.idx = tr_idx as _;
        });
        let mut dev = Self {
            idx: 0,
            transducers,
            enable: true,
            sound_speed: 340.0 * METER,
            rotation: rot,
            center: Point3::origin(),
            x_direction: Vector3::x_axis(),
            y_direction: Vector3::y_axis(),
            axial_direction: Vector3::z_axis(),
            inv: nalgebra::Isometry3::identity(),
            aabb: Aabb::empty(),
        };
        dev.init();
        dev
    }

    /// Gets the index of the device.
    #[must_use]
    pub const fn idx(&self) -> usize {
        self.idx as _
    }

    /// Gets the number of transducers of the device.
    #[must_use]
    pub fn num_transducers(&self) -> usize {
        self.transducers.len()
    }

    /// Sets the sound speed of enabled devices from the temperature `t`.
    ///
    /// This is equivalent to `Self::set_sound_speed_from_temp_with(t, 1.4, 8.314_463, 28.9647e-3)`.
    pub fn set_sound_speed_from_temp(&mut self, temp: f32) {
        self.set_sound_speed_from_temp_with(temp, 1.4, 8.314_463, 28.9647e-3);
    }

    /// Sets the sound speed of enabled devices from the temperature `t`, heat capacity ratio `k`, gas constant `r`, and molar mass `m` [kg/mol].
    pub fn set_sound_speed_from_temp_with(&mut self, temp: f32, k: f32, r: f32, m: f32) {
        self.sound_speed = (k * r * (273.15 + temp) / m).sqrt() * METER;
    }

    /// Gets the wavelength of the ultrasound.
    #[must_use]
    #[cfg_attr(not(feature = "dynamic_freq"), const_fn::const_fn)]
    pub fn wavelength(&self) -> f32 {
        self.sound_speed / ultrasound_freq().hz() as f32
    }

    /// Gets the wavenumber of the ultrasound.
    #[must_use]
    #[cfg_attr(not(feature = "dynamic_freq"), const_fn::const_fn)]
    pub fn wavenumber(&self) -> f32 {
        2.0 * PI * ultrasound_freq().hz() as f32 / self.sound_speed
    }

    #[must_use]
    fn get_direction(dir: Vector3, rotation: &UnitQuaternion) -> UnitVector3 {
        let dir: UnitQuaternion = UnitQuaternion::from_quaternion(Quaternion::from_imag(dir));
        UnitVector3::new_normalize((rotation * dir * rotation.conjugate()).imag())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        defined::{PI, mm},
        geometry::tests::{TestDevice, create_device},
    };

    macro_rules! assert_approx_eq_vec3 {
        ($a:expr, $b:expr) => {
            approx::assert_abs_diff_eq!($a.x, $b.x, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.y, $b.y, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.z, $b.z, epsilon = 1e-3);
        };
    }

    #[test]
    fn idx() {
        assert_eq!(0, create_device(249).idx());
    }

    #[rstest::rstest]
    #[test]
    #[case(1)]
    #[case(249)]
    fn num_transducers(#[case] n: u8) {
        assert_eq!(n, create_device(n).num_transducers() as u8);
    }

    #[test]
    fn center() {
        let device: Device = TestDevice::new_autd3(Point3::origin()).into();
        let expected =
            device.iter().map(|t| t.position().coords).sum::<Vector3>() / device.len() as f32;
        assert_approx_eq_vec3!(expected, device.center());
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Vector3::new(10., 20., 30.),
        Vector3::new(10., 20., 30.),
        Point3::origin(),
        UnitQuaternion::identity()
    )]
    #[case(
        Vector3::zeros(),
        Vector3::new(10., 20., 30.),
        Point3::new(10., 20., 30.),
        UnitQuaternion::identity()
    )]
    #[case(
        Vector3::new(20., -10., 30.),
        Vector3::new(10., 20., 30.),
        Point3::origin(),
        UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.)
    )]
    #[case(
        Vector3::new(30., 30., -30.),
        Vector3::new(40., 50., 60.),
        Point3::new(10., 20., 30.),
        UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.)
    )]
    fn inv(
        #[case] expected: Vector3,
        #[case] target: Vector3,
        #[case] origin: Point3,
        #[case] rot: UnitQuaternion,
    ) {
        let device: Device = TestDevice::new_autd3_with_rot(origin, rot).into();
        assert_approx_eq_vec3!(expected, device.inv.transform_point(&Point3::from(target)));
    }

    #[rstest::rstest]
    #[test]
    #[case(340.29525e3, 15.)]
    #[case(343.23497e3, 20.)]
    #[case(349.04013e3, 30.)]
    fn set_sound_speed_from_temp(#[case] expected: f32, #[case] temp: f32) {
        let mut device = create_device(249);
        device.set_sound_speed_from_temp(temp);
        approx::assert_abs_diff_eq!(expected * mm, device.sound_speed, epsilon = 1e-3);
    }

    #[rstest::rstest]
    #[test]
    #[case(8.5, 340e3)]
    #[case(10., 400e3)]
    fn wavelength(#[case] expect: f32, #[case] c: f32) {
        let mut device = create_device(249);
        device.sound_speed = c;
        approx::assert_abs_diff_eq!(expect, device.wavelength());
    }

    #[rstest::rstest]
    #[test]
    #[case(0.739_198_27, 340e3)]
    #[case(0.628_318_55, 400e3)]
    fn wavenumber(#[case] expect: f32, #[case] c: f32) {
        let mut device = create_device(249);
        device.sound_speed = c;
        approx::assert_abs_diff_eq!(expect, device.wavenumber());
    }
}
