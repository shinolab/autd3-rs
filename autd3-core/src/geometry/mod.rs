pub(crate) mod device;
mod rotation;
mod transducer;

#[cfg(feature = "use_nalgebra")]
mod math {
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
    pub type Translation3 = nalgebra::Translation3<f32>;
    /// A 3-dimensional isometry.
    pub type Isometry3 = nalgebra::Isometry3<f32>;
}

#[cfg(not(feature = "use_nalgebra"))]
mod math;

pub use math::*;

use alloc::vec::Vec;

pub use device::*;
pub use rotation::*;
pub use transducer::*;

/// Geometry of the devices.
pub struct Geometry {
    pub(crate) devices: Vec<Device>,
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

    /// Gets the number of devices.
    #[must_use]
    pub fn num_devices(&self) -> usize {
        self.devices.len()
    }

    /// Gets the number of transducers.
    #[must_use]
    pub fn num_transducers(&self) -> usize {
        self.iter().map(|dev| dev.num_transducers()).sum()
    }

    /// Gets the center of the transducers.
    #[must_use]
    pub fn center(&self) -> Point3 {
        Point3::from(
            self.iter().map(|d| d.center().coords).sum::<Vector3>() / self.devices.len() as f32,
        )
    }

    #[doc(hidden)]
    pub fn version(&self) -> usize {
        self.version
    }

    /// Reconfigure the geometry.
    pub fn reconfigure<D: Into<Device>, F: Fn(Device) -> D>(&mut self, f: F) {
        self.devices = self.devices.drain(..).map(|dev| f(dev).into()).collect();
        self.assign_idx();
        self.version += 1;
    }
}

impl<'a> IntoIterator for &'a Geometry {
    type Item = &'a Device;
    type IntoIter = core::slice::Iter<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.devices.iter()
    }
}

impl<'a> IntoIterator for &'a mut Geometry {
    type Item = &'a mut Device;
    type IntoIter = core::slice::IterMut<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.version += 1;
        self.devices.iter_mut()
    }
}

impl core::ops::Deref for Geometry {
    type Target = Vec<Device>;

    fn deref(&self) -> &Self::Target {
        &self.devices
    }
}

impl core::ops::DerefMut for Geometry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.version += 1;
        &mut self.devices
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use alloc::vec::Vec;
    use rand::Rng;

    use crate::common::mm;

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
            let isometry = Isometry3 {
                rotation,
                translation: Translation3::from(pos),
            };
            Self {
                rotation,
                transducers: (0..14)
                    .flat_map(|y| {
                        (0..18).map(move |x| {
                            Transducer::new(
                                isometry * (10.16 * mm * Point3::new(x as f32, y as f32, 0.)),
                            )
                        })
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
    #[case(1, alloc::vec![create_device(249)])]
    #[case(2, alloc::vec![create_device(249), create_device(249)])]
    fn test_num_devices(#[case] expected: usize, #[case] devices: Vec<Device>) {
        let geometry = Geometry::new(devices);
        assert_eq!(0, geometry.version());
        assert_eq!(expected, geometry.num_devices());
        assert_eq!(0, geometry.version());
    }

    #[rstest::rstest]
    #[case(249, alloc::vec![create_device(249)])]
    #[case(498, alloc::vec![create_device(249), create_device(249)])]
    fn test_num_transducers(#[case] expected: usize, #[case] devices: Vec<Device>) {
        let geometry = Geometry::new(devices);
        assert_eq!(0, geometry.version());
        assert_eq!(expected, geometry.num_transducers());
        assert_eq!(0, geometry.version());
    }

    #[test]
    fn test_center() {
        let geometry = Geometry::new(alloc::vec![
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

    #[test]
    fn into_iter() {
        let mut geometry = create_geometry(1, 1);
        assert_eq!(0, geometry.version());
        (&mut geometry).into_iter().for_each(|dev| {
            _ = dev;
        });
        assert_eq!(1, geometry.version());
    }

    #[test]
    fn idx() {
        let geometry = Geometry::new(alloc::vec![
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
        let mut geometry = Geometry::new(alloc::vec![
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
            _ => TestDevice::new_autd3_with_rot(dev[0].position(), dev.rotation()),
        });

        assert_eq!(1, geometry.version());
        assert_eq!(t, geometry[0][0].position());
        assert_eq!(rot, geometry[0].rotation());
        assert_eq!(Point3::origin(), geometry[1][0].position());
        assert_eq!(UnitQuaternion::identity(), geometry[1].rotation());
    }
}
