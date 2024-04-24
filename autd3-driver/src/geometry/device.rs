use std::ops::Deref;

use crate::defined::METER;

use super::{Matrix3, Transducer, UnitQuaternion, Vector3};

pub struct Device {
    idx: usize,
    transducers: Vec<Transducer>,
    pub enable: bool,
    pub sound_speed: f64,
    pub attenuation: f64,
    inv: Matrix3,
}

impl Device {
    pub(crate) fn new(idx: usize, transducers: Vec<Transducer>) -> Self {
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

    pub const fn idx(&self) -> usize {
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
            / self.transducers.len() as f64
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
    pub fn set_sound_speed_from_temp(&mut self, temp: f64) {
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
    pub fn set_sound_speed_from_temp_with(&mut self, temp: f64, k: f64, r: f64, m: f64) {
        self.sound_speed = (k * r * (273.15 + temp) / m).sqrt() * METER;
    }
}

#[cfg(feature = "variable_frequency")]
impl Device {
    pub fn set_frequency(&mut self, frequency: f64) {
        self.transducers
            .iter_mut()
            .for_each(|tr| tr.set_frequency(frequency));
    }

    pub fn frequency(&self) -> f64 {
        self.transducers[0].frequency()
    }
}

impl Deref for Device {
    type Target = [Transducer];

    fn deref(&self) -> &Self::Target {
        &self.transducers
    }
}

// GRCOV_EXCL_START
impl<'a> IntoIterator for &'a Device {
    type Item = &'a Transducer;
    type IntoIter = std::slice::Iter<'a, Transducer>;

    fn into_iter(self) -> Self::IntoIter {
        self.transducers.iter()
    }
}
// GRCOV_EXCL_STOP

pub trait IntoDevice {
    fn into_device(self, dev_idx: usize) -> Device;
}

#[cfg(test)]
pub mod tests {
    use rand::Rng;

    use crate::defined::{MILLIMETER, PI};

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

    #[rstest::rstest]
    #[test]
    #[case(0)]
    #[case(1)]
    fn test_idx(#[case] idx: usize) {
        assert_eq!(idx, create_device(idx, 249).idx());
    }

    #[rstest::rstest]
    #[test]
    #[case(1)]
    #[case(249)]
    fn test_num_transducers(#[case] n: usize) {
        assert_eq!(n, create_device(0, n).num_transducers());
    }

    #[test]
    fn test_center() {
        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as f64, y as f64, 0.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();
        let expected =
            transducers.iter().map(|t| t.position()).sum::<Vector3>() / transducers.len() as f64;
        let device = Device::new(0, transducers);
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
    fn test_to_local(
        #[case] expected: Vector3,
        #[case] target: Vector3,
        #[case] origin: Vector3,
        #[case] quat: UnitQuaternion,
    ) {
        let device = Device::new(
            0,
            itertools::iproduct!((0..18), (0..14))
                .enumerate()
                .map(|(i, (y, x))| {
                    Transducer::new(
                        i,
                        origin + 10.16 * Vector3::new(x as f64, y as f64, 0.),
                        quat,
                    )
                })
                .collect::<Vec<_>>(),
        );
        assert_approx_eq_vec3!(expected, device.to_local(&target));
    }

    #[test]
    fn test_translate_to() {
        let mut rng = rand::thread_rng();
        let origin = Vector3::new(rng.gen(), rng.gen(), rng.gen());

        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    origin + 10.16 * Vector3::new(x as f64, y as f64, 0.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();

        let mut device = Device::new(0, transducers);

        let t = Vector3::new(40., 50., 60.);
        device.translate_to(t);

        itertools::iproduct!((0..18), (0..14))
            .map(|(y, x)| 10.16 * Vector3::new(x as f64, y as f64, 0.) + t)
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[test]
    fn test_rotate_to() {
        let mut device = {
            let mut device = Device::new(
                0,
                itertools::iproduct!((0..18), (0..14))
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(
                            i,
                            10.16 * Vector3::new(x as f64, y as f64, 0.),
                            UnitQuaternion::identity(),
                        )
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
        let expect_z = Vector3::new(0., 0., 1.);
        device.iter().for_each(|tr| {
            assert_approx_eq_quat!(rot, tr.rotation());
            assert_approx_eq_vec3!(expect_x, tr.x_direction());
            assert_approx_eq_vec3!(expect_y, tr.y_direction());
            assert_approx_eq_vec3!(expect_z, tr.z_direction());
        });
        itertools::iproduct!((0..18), (0..14))
            .map(|(y, x)| 10.16 * Vector3::new(-y as f64, x as f64, 0.))
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[test]
    fn test_translate() {
        let mut rng = rand::thread_rng();
        let origin = Vector3::new(rng.gen(), rng.gen(), rng.gen());

        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    origin + 10.16 * Vector3::new(x as f64, y as f64, 0.),
                    UnitQuaternion::identity(),
                )
            })
            .collect::<Vec<_>>();

        let mut device = Device::new(0, transducers.clone());

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
    fn test_rotate() {
        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as f64, y as f64, 0.),
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
            .map(|(y, x)| 10.16 * Vector3::new(-y as f64, x as f64, 0.))
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[test]
    fn test_affine() {
        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| {
                Transducer::new(
                    i,
                    10.16 * Vector3::new(x as f64, y as f64, 0.),
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
            .map(|(y, x)| 10.16 * Vector3::new(-y as f64, x as f64, 0.) + t)
            .zip(device.iter())
            .for_each(|(expect, tr)| {
                assert_approx_eq_vec3!(expect, tr.position());
            });
    }

    #[rstest::rstest]
    #[test]
    #[case(340.29527186788846e3, 15.)]
    #[case(343.23498846612807e3, 20.)]
    #[case(349.0401521469255e3, 30.)]
    fn test_set_sound_speed_from_temp(#[case] expected: f64, #[case] temp: f64) {
        let mut device = create_device(0, 249);
        device.set_sound_speed_from_temp(temp);
        assert_approx_eq::assert_approx_eq!(expected * MILLIMETER, device.sound_speed, 1e-3);
    }
}
