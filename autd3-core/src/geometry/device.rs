use bvh::aabb::Aabb;
use derive_more::{Deref, IntoIterator};
use getset::Getters;

use super::{Isometry, Point3, Quaternion, Transducer, UnitQuaternion, UnitVector3, Vector3};

/// An AUTD device unit.
#[derive(Getters, Deref, IntoIterator)]
pub struct Device {
    pub(crate) idx: u16,
    #[deref]
    #[into_iterator(ref)]
    pub(crate) transducers: Vec<Transducer>,
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
    pub const fn num_transducers(&self) -> usize {
        self.transducers.len()
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
}
