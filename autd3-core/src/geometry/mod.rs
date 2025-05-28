pub(crate) mod device;
mod rotation;
mod transducer;

/// a complex number
pub type Complex = nalgebra::Complex<f32>;
/// 3-dimensional column vector.
pub type Vector3 = nalgebra::Vector3<f32>;
/// 3-dimensional unit vector.
pub type UnitVector3 = nalgebra::UnitVector3<f32>;
/// 3-dimensional point.
pub type Point3 = nalgebra::Point3<f32>;
/// A quaternion.
pub type Quaternion = nalgebra::Quaternion<f32>;
/// A unit quaternion.
pub type UnitQuaternion = nalgebra::UnitQuaternion<f32>;
/// A 3-dimensional translation.
pub type Translation = nalgebra::Translation3<f32>;
/// A 3-dimensional isometry.
pub type Isometry = nalgebra::Isometry3<f32>;

pub use bvh::aabb::Aabb;
pub use device::*;
use getset::CopyGetters;
pub use rotation::*;
pub use transducer::*;

use derive_more::{Deref, IntoIterator};

/// Geometry of the devices.
#[derive(Deref, CopyGetters, IntoIterator)]
pub struct Geometry {
    #[deref]
    #[into_iterator(ref)]
    pub(crate) devices: Vec<Device>,
    #[doc(hidden)]
    #[getset(get_copy = "pub")]
    version: usize,
}

impl Geometry {
    /// Creates a new [`Geometry`].
    #[must_use]
    pub fn new(devices: Vec<Device>) -> Self {
        let mut geometry = Self {
            devices,
            version: 0,
        };
        geometry.assign_idx();
        geometry
    }

    fn assign_idx(&mut self) {
        self.devices
            .iter_mut()
            .enumerate()
            .for_each(|(dev_idx, dev)| {
                dev.idx = dev_idx as _;
                dev.transducers.iter_mut().for_each(|tr| {
                    tr.dev_idx = dev_idx as _;
                });
            });
    }

    /// Gets the number of enabled devices.
    #[must_use]
    pub fn num_devices(&self) -> usize {
        self.devices().count()
    }

    /// Gets the number of enabled transducers.
    #[must_use]
    pub fn num_transducers(&self) -> usize {
        self.devices().map(|dev| dev.num_transducers()).sum()
    }

    /// Gets the center of the enabled transducers.
    #[must_use]
    pub fn center(&self) -> Point3 {
        Point3::from(
            self.devices().map(|d| d.center().coords).sum::<Vector3>() / self.devices.len() as f32,
        )
    }

    /// Gets the iterator of enabled devices.
    pub fn devices(&self) -> impl Iterator<Item = &Device> {
        self.iter().filter(|dev| dev.enable)
    }

    /// Gets the mutable iterator of enabled devices.
    pub fn devices_mut(&mut self) -> impl Iterator<Item = &mut Device> {
        self.iter_mut().filter(|dev| dev.enable)
    }

    /// Sets the sound speed of enabled devices.
    pub fn set_sound_speed(&mut self, c: f32) {
        self.devices_mut().for_each(|dev| dev.sound_speed = c);
    }

    /// Sets the sound speed of enabled devices from the temperature `t`.
    ///
    /// This is equivalent to `Self::set_sound_speed_from_temp_with(t, 1.4, 8.314_463, 28.9647e-3)`.
    pub fn set_sound_speed_from_temp(&mut self, t: f32) {
        self.set_sound_speed_from_temp_with(t, 1.4, 8.314_463, 28.9647e-3);
    }

    /// Sets the sound speed of enabled devices from the temperature `t`, heat capacity ratio `k`, gas constant `r`, and molar mass `m` [kg/mol].
    pub fn set_sound_speed_from_temp_with(&mut self, t: f32, k: f32, r: f32, m: f32) {
        self.devices_mut()
            .for_each(|dev| dev.set_sound_speed_from_temp_with(t, k, r, m));
    }

    /// Axis Aligned Bounding Box of enabled devices.
    #[must_use]
    pub fn aabb(&self) -> Aabb<f32, 3> {
        self.devices()
            .fold(Aabb::empty(), |aabb, dev| aabb.join(dev.aabb()))
    }

    /// Reconfigure the geometry.
    pub fn reconfigure<D: Into<Device>, F: Fn(&Device) -> D>(&mut self, f: F) {
        self.devices.iter_mut().for_each(|dev| {
            let enable = dev.enable;
            let sound_speed = dev.sound_speed;
            *dev = f(dev).into();
            dev.enable = enable;
            dev.sound_speed = sound_speed;
        });
        self.assign_idx();
        self.version += 1;
    }

    #[doc(hidden)]
    pub fn lock_version<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let version = self.version;
        let r = f(self);
        self.version = version;
        r
    }
}

impl<'a> IntoIterator for &'a mut Geometry {
    type Item = &'a mut Device;
    type IntoIter = std::slice::IterMut<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.version += 1;
        self.devices.iter_mut()
    }
}

impl std::ops::DerefMut for Geometry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.version += 1;
        &mut self.devices
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use rand::Rng;

    use crate::common::{deg, mm};

    use super::*;

    macro_rules! assert_approx_eq_vec3 {
        ($a:expr, $b:expr) => {
            approx::assert_abs_diff_eq!($a.x, $b.x, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.y, $b.y, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.z, $b.z, epsilon = 1e-3);
        };
    }

    pub struct TestDevice {
        pub rotation: UnitQuaternion,
        pub transducers: Vec<Transducer>,
    }

    impl TestDevice {
        pub fn new_autd3(pos: Point3) -> Self {
            Self::new_autd3_with_rot(pos, UnitQuaternion::identity())
        }

        pub fn new_autd3_with_rot(pos: Point3, rot: impl Into<UnitQuaternion>) -> Self {
            let rotation = rot.into();
            let isometry = Isometry {
                rotation,
                translation: Translation::from(pos),
            };
            Self {
                rotation,
                transducers: itertools::iproduct!(0..14, 0..18)
                    .map(|(y, x)| {
                        Transducer::new(
                            (isometry * (10.16 * mm * Point3::new(x as f32, y as f32, 0.))).xyz(),
                        )
                    })
                    .collect(),
            }
        }
    }

    impl From<TestDevice> for Device {
        fn from(dev: TestDevice) -> Self {
            Self::new(dev.rotation, dev.transducers)
        }
    }

    pub fn create_device(n: u8) -> Device {
        Device::new(
            UnitQuaternion::identity(),
            (0..n).map(|_| Transducer::new(Point3::origin())).collect(),
        )
    }

    pub fn create_geometry(n: u16, num_trans_in_unit: u8) -> Geometry {
        Geometry::new((0..n).map(|_| create_device(num_trans_in_unit)).collect())
    }

    #[rstest::rstest]
    #[test]
    #[case(1, vec![create_device(249)])]
    #[case(2, vec![create_device(249), create_device(249)])]
    fn test_num_devices(#[case] expected: usize, #[case] devices: Vec<Device>) {
        let geometry = Geometry::new(devices);
        assert_eq!(0, geometry.version());
        assert_eq!(expected, geometry.num_devices());
        assert_eq!(0, geometry.version());
    }

    #[rstest::rstest]
    #[test]
    #[case(249, vec![create_device(249)])]
    #[case(498, vec![create_device(249), create_device(249)])]
    fn test_num_transducers(#[case] expected: usize, #[case] devices: Vec<Device>) {
        let geometry = Geometry::new(devices);
        assert_eq!(0, geometry.version());
        assert_eq!(expected, geometry.num_transducers());
        assert_eq!(0, geometry.version());
    }

    #[test]
    fn test_center() {
        let geometry = Geometry::new(vec![
            TestDevice::new_autd3(Point3::origin()).into(),
            TestDevice::new_autd3(Point3::new(10., 20., 30.)).into(),
        ]);
        let expect = geometry
            .iter()
            .map(|dev| dev.center().coords)
            .sum::<Vector3>()
            / geometry.num_devices() as f32;
        assert_eq!(0, geometry.version());
        assert_approx_eq_vec3!(expect, geometry.center());
        assert_eq!(0, geometry.version());
    }

    #[rstest::rstest]
    #[test]
    #[case(340.29525e3, 15.)]
    #[case(343.23497e3, 20.)]
    #[case(349.04013e3, 30.)]
    fn test_set_sound_speed_from_temp(#[case] expected: f32, #[case] temp: f32) {
        let mut geometry = create_geometry(2, 1);
        assert_eq!(0, geometry.version());
        geometry.set_sound_speed_from_temp(temp);
        assert_eq!(1, geometry.version());
        geometry.iter().for_each(|dev| {
            approx::assert_abs_diff_eq!(expected * mm, dev.sound_speed, epsilon = 1e-3);
        });
    }

    #[rstest::rstest]
    #[test]
    #[case(3.402_952_8e5)]
    #[case(3.432_35e5)]
    #[case(3.490_401_6e5)]
    fn test_set_sound_speed(#[case] temp: f32) {
        let mut geometry = create_geometry(2, 1);
        assert_eq!(0, geometry.version());
        geometry.set_sound_speed(temp * mm);
        assert_eq!(1, geometry.version());
        geometry.iter().for_each(|dev| {
            assert_eq!(dev.sound_speed, temp * mm);
        });
    }

    #[test]
    fn into_iter() {
        let mut geometry = create_geometry(1, 1);
        assert_eq!(0, geometry.version());
        for dev in &mut geometry {
            dev.enable = true;
        }
        assert_eq!(1, geometry.version());
    }

    #[rstest::rstest]
    #[test]
    #[case(Aabb{min: Point3::origin(), max: Point3::new(172.72 * mm, 132.08 * mm, 0.)}, vec![TestDevice::new_autd3(Point3::origin())])]
    #[case(Aabb{min: Point3::new(10. * mm, 20. * mm, 30. * mm), max: Point3::new(182.72 * mm, 152.08 * mm, 30. * mm)}, vec![TestDevice::new_autd3(Point3::new(10. * mm, 20. * mm, 30. * mm))])]
    #[case(Aabb{min: Point3::new(-132.08 * mm, 0., 0.), max: Point3::new(0., 172.72 * mm, 0.)}, vec![TestDevice::new_autd3_with_rot(Point3::origin(), EulerAngle::ZYZ(90. * deg, 0. * deg, 0. * deg))])]
    #[case(Aabb{min: Point3::new(-132.08 * mm, -10. * mm, 0.), max: Point3::new(172.72 * mm, 162.72 * mm, 10. * mm)}, vec![
        TestDevice::new_autd3(Point3::origin()),
        TestDevice::new_autd3_with_rot(Point3::new(0., -10. * mm, 10. * mm), EulerAngle::ZYZ(90. * deg, 0. * deg, 0. * deg))
    ])]
    fn aabb(#[case] expect: Aabb<f32, 3>, #[case] dev: Vec<TestDevice>) {
        let geometry = Geometry::new(dev.into_iter().map(|d| d.into()).collect());
        assert_approx_eq_vec3!(expect.min, geometry.aabb().min);
        assert_approx_eq_vec3!(expect.max, geometry.aabb().max);
    }

    #[test]
    fn idx() {
        let geometry = Geometry::new(vec![
            TestDevice::new_autd3_with_rot(Point3::origin(), UnitQuaternion::identity()).into(),
            TestDevice::new_autd3_with_rot(Point3::origin(), UnitQuaternion::identity()).into(),
        ]);
        (0..2).for_each(|dev_idx| {
            assert_eq!(dev_idx, geometry[dev_idx].idx());
            (0..14 * 18).for_each(|tr_idx| {
                assert_eq!(tr_idx, geometry[dev_idx][tr_idx].idx());
                assert_eq!(dev_idx, geometry[dev_idx][tr_idx].dev_idx());
            });
        });
    }

    #[test]
    fn reconfigure() {
        let mut geometry = Geometry::new(vec![
            TestDevice::new_autd3_with_rot(Point3::origin(), UnitQuaternion::identity()).into(),
            TestDevice::new_autd3_with_rot(Point3::origin(), UnitQuaternion::identity()).into(),
        ]);

        let mut rng = rand::rng();
        let t = Point3::new(rng.random(), rng.random(), rng.random());
        let rot = UnitQuaternion::new_normalize(Quaternion::new(
            rng.random(),
            rng.random(),
            rng.random(),
            rng.random(),
        ));

        geometry.reconfigure(|dev| match dev.idx() {
            0 => TestDevice::new_autd3_with_rot(t, rot),
            _ => TestDevice::new_autd3_with_rot(*dev[0].position(), *dev.rotation()),
        });

        assert_eq!(1, geometry.version());
        assert_eq!(t, *geometry[0][0].position());
        assert_eq!(rot, *geometry[0].rotation());
        assert_eq!(Point3::origin(), *geometry[1][0].position());
        assert_eq!(UnitQuaternion::identity(), *geometry[1].rotation());
    }
}
