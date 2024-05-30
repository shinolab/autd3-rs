use std::{f64::consts::PI, ops::Deref};

use crate::defined::{Freq, FREQ_40K, METER};

use super::{Matrix3, Quaternion, Transducer, UnitQuaternion, Vector3};

pub struct Device {
    idx: usize,
    transducers: Vec<Transducer>,
    pub enable: bool,
    pub sound_speed: f64,
    pub attenuation: f64,
    rot: UnitQuaternion,
    x_direction: Vector3,
    y_direction: Vector3,
    axial_direction: Vector3,
    inv: Matrix3,
    pub(crate) ultrasound_freq: Freq<u32>,
}

impl Device {
    #[doc(hidden)]
    pub fn new(idx: usize, rot: UnitQuaternion, transducers: Vec<Transducer>) -> Self {
        let inv = Matrix3::from_columns(&[
            Self::get_direction(Vector3::x(), &rot),
            Self::get_direction(Vector3::y(), &rot),
            Self::get_direction(Vector3::z(), &rot),
        ])
        .transpose();
        Self {
            idx,
            transducers,
            enable: true,
            sound_speed: 340.0 * METER,
            attenuation: 0.0,
            rot,
            x_direction: Self::get_direction(Vector3::x(), &rot),
            y_direction: Self::get_direction(Vector3::y(), &rot),
            #[cfg(feature = "left_handed")]
            axial_direction: Self::get_direction(-Vector3::z(), &rot),
            #[cfg(not(feature = "left_handed"))]
            axial_direction: Self::get_direction(Vector3::z(), &rot),
            inv,
            ultrasound_freq: FREQ_40K,
        }
    }

    pub const fn idx(&self) -> usize {
        self.idx
    }

    pub fn num_transducers(&self) -> usize {
        self.transducers.len()
    }

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

    pub fn translate_to(&mut self, t: Vector3) {
        let cur_pos = self.transducers[0].position();
        self.translate(t - cur_pos);
    }

    pub fn rotate_to(&mut self, r: UnitQuaternion) {
        let cur_rot = self.rot;
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
        self.rot = r * self.rot;
        self.inv = Matrix3::from_columns(&[
            Self::get_direction(Vector3::x(), &self.rot),
            Self::get_direction(Vector3::y(), &self.rot),
            Self::get_direction(Vector3::z(), &self.rot),
        ])
        .transpose();
        self.x_direction = Self::get_direction(Vector3::x(), &self.rot);
        self.y_direction = Self::get_direction(Vector3::y(), &self.rot);
        self.axial_direction = if cfg!(feature = "left_handed") {
            Self::get_direction(-Vector3::z(), &self.rot) // GRCOV_EXCL_LINE
        } else {
            Self::get_direction(Vector3::z(), &self.rot)
        };
    }

    pub fn set_sound_speed_from_temp(&mut self, temp: f64) {
        self.set_sound_speed_from_temp_with(temp, 1.4, 8.314_463, 28.9647e-3);
    }

    pub fn set_sound_speed_from_temp_with(&mut self, temp: f64, k: f64, r: f64, m: f64) {
        self.sound_speed = (k * r * (273.15 + temp) / m).sqrt() * METER;
    }

    pub(crate) const fn ultrasound_freq(&self) -> Freq<u32> {
        self.ultrasound_freq
    }

    pub fn wavelength(&self) -> f64 {
        self.sound_speed / self.ultrasound_freq.freq as f64
    }

    pub fn wavenumber(&self) -> f64 {
        2.0 * PI * self.ultrasound_freq.freq as f64 / self.sound_speed
    }

    fn get_direction(dir: Vector3, rotation: &UnitQuaternion) -> Vector3 {
        let dir: UnitQuaternion = UnitQuaternion::from_quaternion(Quaternion::from_imag(dir));
        (rotation * dir * rotation.conjugate()).imag().normalize()
    }

    pub const fn rotation(&self) -> &UnitQuaternion {
        &self.rot
    }

    pub const fn x_direction(&self) -> &Vector3 {
        &self.x_direction
    }

    pub const fn y_direction(&self) -> &Vector3 {
        &self.y_direction
    }

    pub const fn axial_direction(&self) -> &Vector3 {
        &self.axial_direction
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

    use super::*;
    use crate::{
        defined::Hz,
        defined::{mm, PI},
        geometry::tests::create_device,
    };

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
            .map(|(i, (y, x))| Transducer::new(i, 10.16 * Vector3::new(x as f64, y as f64, 0.)))
            .collect::<Vec<_>>();
        let expected =
            transducers.iter().map(|t| t.position()).sum::<Vector3>() / transducers.len() as f64;
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
    fn test_to_local(
        #[case] expected: Vector3,
        #[case] target: Vector3,
        #[case] origin: Vector3,
        #[case] quat: UnitQuaternion,
    ) {
        let device = Device::new(
            0,
            quat,
            itertools::iproduct!((0..18), (0..14))
                .enumerate()
                .map(|(i, (y, x))| {
                    Transducer::new(i, origin + 10.16 * Vector3::new(x as f64, y as f64, 0.))
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
                Transducer::new(i, origin + 10.16 * Vector3::new(x as f64, y as f64, 0.))
            })
            .collect::<Vec<_>>();

        let mut device = Device::new(0, UnitQuaternion::identity(), transducers);

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
                UnitQuaternion::identity(),
                itertools::iproduct!((0..18), (0..14))
                    .enumerate()
                    .map(|(i, (y, x))| {
                        Transducer::new(i, 10.16 * Vector3::new(x as f64, y as f64, 0.))
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
                Transducer::new(i, origin + 10.16 * Vector3::new(x as f64, y as f64, 0.))
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
    fn test_rotate() {
        let transducers = itertools::iproduct!((0..18), (0..14))
            .enumerate()
            .map(|(i, (y, x))| Transducer::new(i, 10.16 * Vector3::new(x as f64, y as f64, 0.)))
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
            .map(|(i, (y, x))| Transducer::new(i, 10.16 * Vector3::new(x as f64, y as f64, 0.)))
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
        assert_approx_eq::assert_approx_eq!(expected * mm, device.sound_speed, 1e-3);
    }

    #[rstest::rstest]
    #[test]
    #[case(8.5, 340e3, 40000*Hz)]
    #[case(10., 400e3, 40000*Hz)]
    #[case(4.25, 340e3, 80000*Hz)]
    #[case(5., 400e3, 80000*Hz)]
    fn wavelength(#[case] expect: f64, #[case] c: f64, #[case] freq: Freq<u32>) {
        let mut device = create_device(0, 249);
        device.ultrasound_freq = freq;
        device.sound_speed = c;
        assert_approx_eq::assert_approx_eq!(expect, device.wavelength());
    }

    #[rstest::rstest]
    #[test]
    #[case(0.7391982714328925, 340e3, 40000*Hz)]
    #[case(0.6283185307179586, 400e3, 40000*Hz)]
    #[case(1.478396542865785, 340e3, 80000*Hz)]
    #[case(1.2566370614359172, 400e3, 80000*Hz)]
    fn wavenumber(#[case] expect: f64, #[case] c: f64, #[case] freq: Freq<u32>) {
        let mut device = create_device(0, 249);
        device.ultrasound_freq = freq;
        device.sound_speed = c;
        assert_approx_eq::assert_approx_eq!(expect, device.wavenumber());
    }
}
