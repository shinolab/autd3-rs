pub(crate) mod device;
mod rotation;
mod transducer;

pub type Vector3 = nalgebra::Vector3<f32>;
pub type UnitVector3 = nalgebra::UnitVector3<f32>;
pub type Point3 = nalgebra::Point3<f32>;
pub type Vector4 = nalgebra::Vector4<f32>;
pub type Quaternion = nalgebra::Quaternion<f32>;
pub type UnitQuaternion = nalgebra::UnitQuaternion<f32>;
pub type Matrix3 = nalgebra::Matrix3<f32>;
pub type Matrix4 = nalgebra::Matrix4<f32>;
pub type Affine = nalgebra::Affine3<f32>;

use autd3_derive::Builder;
use bvh::aabb::Aabb;
pub use device::*;
pub use rotation::*;
pub use transducer::*;

use derive_more::{Deref, IntoIterator};
use derive_new::new;

#[derive(Deref, Builder, IntoIterator, new)]
pub struct Geometry {
    #[deref]
    #[into_iterator(ref)]
    pub(crate) devices: Vec<Device>,
    #[new(default)]
    #[get]
    version: usize,
    #[get]
    default_parallel_threshold: usize,
}

impl Geometry {
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

    pub fn aabb(&self) -> Aabb<f32, 3> {
        self.devices()
            .fold(Aabb::empty(), |aabb, dev| aabb.join(dev.aabb()))
    }

    pub fn parallel(&self, threshold: Option<usize>) -> bool {
        self.num_devices() > threshold.unwrap_or(self.default_parallel_threshold)
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
pub mod tests {
    use nalgebra::Point3;

    use crate::{
        autd3_device::AUTD3,
        defined::{deg, mm},
    };

    use super::*;

    macro_rules! assert_approx_eq_vec3 {
        ($a:expr, $b:expr) => {
            approx::assert_abs_diff_eq!($a.x, $b.x, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.y, $b.y, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.z, $b.z, epsilon = 1e-3);
        };
    }

    pub fn create_device(idx: u16, n: u8) -> Device {
        Device::new(
            idx,
            UnitQuaternion::identity(),
            (0..n)
                .map(|i| Transducer::new(i, idx, Vector3::zeros()))
                .collect(),
        )
    }

    pub fn create_geometry(n: u16, num_trans_in_unit: u8) -> Geometry {
        Geometry::new(
            (0..n)
                .map(|i| create_device(i, num_trans_in_unit))
                .collect(),
            4,
        )
    }

    #[rstest::rstest]
    #[test]
    #[case(1, vec![create_device(0, 249)])]
    #[case(2, vec![create_device(0, 249), create_device(0, 249)])]
    fn test_num_devices(#[case] expected: usize, #[case] devices: Vec<Device>) {
        let geometry = Geometry::new(devices, 4);
        assert_eq!(0, geometry.version());
        assert_eq!(expected, geometry.num_devices());
        assert_eq!(0, geometry.version());
    }

    #[rstest::rstest]
    #[test]
    #[case(249, vec![create_device(0, 249)])]
    #[case(498, vec![create_device(0, 249), create_device(0, 249)])]
    fn test_num_transducers(#[case] expected: usize, #[case] devices: Vec<Device>) {
        let geometry = Geometry::new(devices, 4);
        assert_eq!(0, geometry.version());
        assert_eq!(expected, geometry.num_transducers());
        assert_eq!(0, geometry.version());
    }

    #[test]
    fn test_center() {
        let geometry = Geometry::new(
            vec![
                Device::new(
                    0,
                    UnitQuaternion::identity(),
                    itertools::iproduct!(0..18, 0..14)
                        .enumerate()
                        .map(|(i, (y, x))| {
                            Transducer::new(
                                i as _,
                                i as _,
                                10.16 * Vector3::new(x as f32, y as f32, 0.),
                            )
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
                                i as _,
                                i as _,
                                10.16 * Vector3::new(x as f32, y as f32, 0.)
                                    + Vector3::new(10., 20., 30.),
                            )
                        })
                        .collect::<Vec<_>>(),
                ),
            ],
            4,
        );
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
    #[case(Aabb{min: Point3::origin(), max: Point3::new(172.72 * mm, 132.08 * mm, 0.)}, vec![AUTD3::new(Vector3::zeros()).into_device(0)])]
    #[case(Aabb{min: Point3::new(10. * mm, 20. * mm, 30. * mm), max: Point3::new(182.72 * mm, 152.08 * mm, 30. * mm)}, vec![AUTD3::new(Vector3::new(10. * mm, 20. * mm, 30. * mm)).into_device(0)])]
    #[case(Aabb{min: Point3::new(-132.08 * mm, 0., 0.), max: Point3::new(0., 172.72 * mm, 0.)}, vec![AUTD3::new(Vector3::zeros()).with_rotation(EulerAngle::ZYZ(90. * deg, 0. * deg, 0. * deg)).into_device(0)])]
    #[case(Aabb{min: Point3::new(-132.08 * mm, -10. * mm, 0.), max: Point3::new(172.72 * mm, 162.72 * mm, 10. * mm)}, vec![AUTD3::new(Vector3::zeros()).into_device(0), AUTD3::new(Vector3::new(0., -10. * mm, 10. * mm)).with_rotation(EulerAngle::ZYZ(90. * deg, 0. * deg, 0. * deg)).into_device(1)])]
    fn aabb(#[case] expect: Aabb<f32, 3>, #[case] dev: Vec<Device>) {
        let geometry = Geometry::new(dev, 4);
        assert_approx_eq_vec3!(expect.min, geometry.aabb().min);
        assert_approx_eq_vec3!(expect.max, geometry.aabb().max);
    }
}
