pub(crate) mod device;
mod rotation;
mod transducer;

pub type Vector3 = nalgebra::Vector3<f32>;
pub type UnitVector3 = nalgebra::UnitVector3<f32>;
pub type Vector4 = nalgebra::Vector4<f32>;
pub type Quaternion = nalgebra::Quaternion<f32>;
pub type UnitQuaternion = nalgebra::UnitQuaternion<f32>;
pub type Matrix3 = nalgebra::Matrix3<f32>;
pub type Matrix4 = nalgebra::Matrix4<f32>;
pub type Affine = nalgebra::Affine3<f32>;

use autd3_derive::Builder;
pub use device::*;
pub use rotation::*;
pub use transducer::*;

use derive_more::Deref;

#[derive(Deref, Builder)]
pub struct Geometry {
    #[deref]
    pub(crate) devices: Vec<Device>,
    #[get]
    version: usize,
}

impl Geometry {
    #[doc(hidden)]
    pub const fn new(devices: Vec<Device>) -> Geometry {
        Self {
            devices,
            version: 0,
        }
    }

    pub fn num_devices(&self) -> usize {
        self.devices().count()
    }

    pub fn num_transducers(&self) -> usize {
        self.devices().map(|dev| dev.num_transducers()).sum()
    }

    pub fn center(&self) -> Vector3 {
        self.devices().map(|d| d.center()).sum::<Vector3>() / self.devices.len() as f32
    }

    pub fn devices(&self) -> impl Iterator<Item = &Device> {
        self.iter().filter(|dev| dev.enable)
    }

    pub fn devices_mut(&mut self) -> impl Iterator<Item = &mut Device> {
        self.iter_mut().filter(|dev| dev.enable)
    }

    pub fn set_sound_speed(&mut self, c: f32) {
        self.devices_mut().for_each(|dev| dev.sound_speed = c);
    }

    pub fn set_sound_speed_from_temp(&mut self, temp: f32) {
        self.set_sound_speed_from_temp_with(temp, 1.4, 8.314_463, 28.9647e-3);
    }

    pub fn set_sound_speed_from_temp_with(&mut self, temp: f32, k: f32, r: f32, m: f32) {
        self.devices_mut()
            .for_each(|dev| dev.set_sound_speed_from_temp_with(temp, k, r, m));
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
// GRCOV_EXCL_STOP

#[cfg(test)]
pub mod tests {
    use crate::defined::mm;

    use super::*;

    macro_rules! assert_approx_eq_vec3 {
        ($a:expr, $b:expr) => {
            assert_approx_eq::assert_approx_eq!($a.x, $b.x, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.y, $b.y, 1e-3);
            assert_approx_eq::assert_approx_eq!($a.z, $b.z, 1e-3);
        };
    }

    pub fn create_device(idx: usize, n: usize) -> Device {
        Device::new(
            idx,
            UnitQuaternion::identity(),
            (0..n)
                .map(|i| Transducer::new(i, Vector3::zeros()))
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

    #[rstest::rstest]
    #[test]
    #[case(1, vec![create_device(0, 249)])]
    #[case(2, vec![create_device(0, 249), create_device(0, 249)])]
    #[cfg_attr(miri, ignore)]
    fn test_num_devices(#[case] expected: usize, #[case] devices: Vec<Device>) {
        let geometry = Geometry::new(devices);
        assert_eq!(0, geometry.version());
        assert_eq!(expected, geometry.num_devices());
        assert_eq!(0, geometry.version());
    }

    #[rstest::rstest]
    #[test]
    #[case(249, vec![create_device(0, 249)])]
    #[case(498, vec![create_device(0, 249), create_device(0, 249)])]
    #[cfg_attr(miri, ignore)]
    fn test_num_transducers(#[case] expected: usize, #[case] devices: Vec<Device>) {
        let geometry = Geometry::new(devices);
        assert_eq!(0, geometry.version());
        assert_eq!(expected, geometry.num_transducers());
        assert_eq!(0, geometry.version());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_center() {
        let geometry = Geometry::new(vec![
            Device::new(
                0,
                UnitQuaternion::identity(),
                itertools::iproduct!(0..18, 0..14)
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(i, 10.16 * Vector3::new(x as f32, y as f32, 0.))
                    })
                    .collect::<Vec<_>>(),
            ),
            Device::new(
                1,
                UnitQuaternion::identity(),
                itertools::iproduct!(0..18, 0..14)
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(
                            i,
                            10.16 * Vector3::new(x as f32, y as f32, 0.)
                                + Vector3::new(10., 20., 30.),
                        )
                    })
                    .collect::<Vec<_>>(),
            ),
        ]);
        let expect = geometry.iter().map(|dev| dev.center()).sum::<Vector3>()
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
    #[cfg_attr(miri, ignore)]
    fn test_set_sound_speed_from_temp(#[case] expected: f32, #[case] temp: f32) {
        let mut geometry = Geometry::new(vec![
            Device::new(
                0,
                UnitQuaternion::identity(),
                itertools::iproduct!(0..18, 0..14)
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(i, 10.16 * Vector3::new(x as f32, y as f32, 0.))
                    })
                    .collect::<Vec<_>>(),
            ),
            Device::new(
                1,
                UnitQuaternion::identity(),
                itertools::iproduct!(0..18, 0..14)
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(
                            i,
                            10.16 * Vector3::new(x as f32, y as f32, 0.)
                                + Vector3::new(10., 20., 30.),
                        )
                    })
                    .collect::<Vec<_>>(),
            ),
        ]);
        assert_eq!(0, geometry.version());
        geometry.set_sound_speed_from_temp(temp);
        assert_eq!(1, geometry.version());
        geometry.iter().for_each(|dev| {
            assert_approx_eq::assert_approx_eq!(expected * mm, dev.sound_speed, 1e-3);
        });
    }

    #[rstest::rstest]
    #[test]
    #[case(3.402_952_8e5)]
    #[case(3.432_35e5)]
    #[case(3.490_401_6e5)]
    #[cfg_attr(miri, ignore)]
    fn test_set_sound_speed(#[case] temp: f32) {
        let mut geometry = Geometry::new(vec![
            Device::new(
                0,
                UnitQuaternion::identity(),
                itertools::iproduct!(0..18, 0..14)
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(i, 10.16 * Vector3::new(x as f32, y as f32, 0.))
                    })
                    .collect::<Vec<_>>(),
            ),
            Device::new(
                1,
                UnitQuaternion::identity(),
                itertools::iproduct!(0..18, 0..14)
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(
                            i,
                            10.16 * Vector3::new(x as f32, y as f32, 0.)
                                + Vector3::new(10., 20., 30.),
                        )
                    })
                    .collect::<Vec<_>>(),
            ),
        ]);
        assert_eq!(0, geometry.version());
        geometry.set_sound_speed(temp * mm);
        assert_eq!(1, geometry.version());
        geometry.iter().for_each(|dev| {
            assert_eq!(dev.sound_speed, temp * mm);
        });
    }
}
