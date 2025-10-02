use alloc::vec::Vec;

use super::{Isometry, Point3, Quaternion, Transducer, UnitQuaternion, UnitVector3, Vector3};

/// An AUTD device unit.
pub struct Device {
    pub(crate) idx: u16,
    pub(crate) transducers: Vec<Transducer>,
    rotation: UnitQuaternion,
    center: Point3,
    x_direction: UnitVector3,
    y_direction: UnitVector3,
    axial_direction: UnitVector3,
    inv: Isometry,
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
        self.inv = (nalgebra::Translation3::<f32>::from(self.transducers[0].position())
            * self.rotation)
            .inverse();
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
            rotation: rot,
            center: Point3::origin(),
            x_direction: Vector3::x_axis(),
            y_direction: Vector3::y_axis(),
            axial_direction: Vector3::z_axis(),
            inv: nalgebra::Isometry3::identity(),
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
    pub const fn num_transducers(&self) -> usize {
        self.transducers.len()
    }

    /// Gets the rotation of the device.
    #[must_use]
    pub const fn rotation(&self) -> UnitQuaternion {
        self.rotation
    }

    /// Gets the center of the device.
    #[must_use]
    pub const fn center(&self) -> Point3 {
        self.center
    }

    /// Gets the x-direction of the device.
    #[must_use]
    pub const fn x_direction(&self) -> UnitVector3 {
        self.x_direction
    }

    /// Gets the y-direction of the device.
    #[must_use]
    pub const fn y_direction(&self) -> UnitVector3 {
        self.y_direction
    }

    /// Gets the axial direction of the device.
    #[must_use]
    pub const fn axial_direction(&self) -> UnitVector3 {
        self.axial_direction
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn inv(&self) -> &Isometry {
        &self.inv
    }

    #[must_use]
    fn get_direction(dir: Vector3, rotation: &UnitQuaternion) -> UnitVector3 {
        let dir: UnitQuaternion = UnitQuaternion::from_quaternion(Quaternion::from_imag(dir));
        UnitVector3::new_normalize((rotation * dir * rotation.conjugate()).imag())
    }
}

impl core::ops::Deref for Device {
    type Target = [Transducer];

    fn deref(&self) -> &Self::Target {
        &self.transducers
    }
}

impl core::iter::IntoIterator for Device {
    type Item = Transducer;
    type IntoIter = alloc::vec::IntoIter<Transducer>;

    fn into_iter(self) -> Self::IntoIter {
        self.transducers.into_iter()
    }
}

impl<'a> IntoIterator for &'a Device {
    type Item = &'a Transducer;
    type IntoIter = core::slice::Iter<'a, Transducer>;

    fn into_iter(self) -> Self::IntoIter {
        self.transducers.iter()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        common::PI,
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
    fn into_iter() {
        let device = create_device(249);
        let mut count = 0;
        for tr in &device {
            assert_eq!(count, tr.idx());
            count += 1;
        }
        assert_eq!(249, count);

        let mut count = 0;
        device.into_iter().for_each(|tr| {
            assert_eq!(count, tr.idx());
            count += 1;
        });
        assert_eq!(249, count);
    }

    #[test]
    fn idx() {
        assert_eq!(0, create_device(249).idx());
    }

    #[rstest::rstest]
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

    #[test]
    fn direction() {
        let device: Device = TestDevice::new_autd3(Point3::origin()).into();
        assert_approx_eq_vec3!(Vector3::x(), device.x_direction().into_inner());
        assert_approx_eq_vec3!(Vector3::y(), device.y_direction().into_inner());
        if cfg!(feature = "left_handed") {
            assert_approx_eq_vec3!(-Vector3::z(), device.axial_direction().into_inner()); // GRCOV_EXCL_LINE
        } else {
            assert_approx_eq_vec3!(Vector3::z(), device.axial_direction().into_inner());
        }
    }

    #[rstest::rstest]
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
}
