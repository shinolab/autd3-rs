/*
 * File: transducer.rs
 * Project: geometry
 * Created Date: 04/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 02/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

use super::{Matrix4, Quaternion, UnitQuaternion, Vector3, Vector4};

use crate::{
    common::Phase,
    defined::{float, PI, ULTRASOUND_FREQUENCY},
};

pub struct Transducer {
    idx: u8,
    pos: Vector3,
    rot: UnitQuaternion,
}

impl Transducer {
    /// Create transducer
    pub const fn new(idx: usize, pos: Vector3, rot: UnitQuaternion) -> Self {
        assert!(idx < 256);
        Self {
            idx: idx as u8,
            pos,
            rot,
        }
    }

    /// Affine transformation
    pub fn affine(&mut self, t: Vector3, r: UnitQuaternion) {
        let rot_mat: Matrix4 = From::from(r);
        let trans_mat = rot_mat.append_translation(&t);
        let homo = Vector4::new(self.pos[0], self.pos[1], self.pos[2], 1.0);
        let new_pos = trans_mat * homo;
        self.pos = Vector3::new(new_pos[0], new_pos[1], new_pos[2]);
        self.rot = r * self.rot;
    }

    /// Calculate the phase of the transducer to align the phase at the specified position
    pub fn align_phase_at(&self, pos: Vector3, sound_speed: float) -> Phase {
        Phase::from_rad((pos - self.position()).norm() * self.wavenumber(sound_speed))
    }

    /// Get the position of the transducer
    pub const fn position(&self) -> &Vector3 {
        &self.pos
    }

    /// Get the rotation of the transducer
    pub const fn rotation(&self) -> &UnitQuaternion {
        &self.rot
    }

    fn get_direction(dir: Vector3, rotation: &UnitQuaternion) -> Vector3 {
        let dir: UnitQuaternion = UnitQuaternion::from_quaternion(Quaternion::from_imag(dir));
        (rotation * dir * rotation.conjugate()).imag().normalize()
    }

    /// Get the local index of the transducer
    pub fn x_direction(&self) -> Vector3 {
        Self::get_direction(Vector3::x(), self.rotation())
    }
    /// Get the y-direction of the transducer
    pub fn y_direction(&self) -> Vector3 {
        Self::get_direction(Vector3::y(), self.rotation())
    }
    /// Get the z-direction (axial direction) of the transducer
    pub fn z_direction(&self) -> Vector3 {
        Self::get_direction(Vector3::z(), self.rotation())
    }

    /// Get the local transducer index
    pub const fn idx(&self) -> usize {
        self.idx as usize
    }

    /// Get the wavelength of the transducer
    pub fn wavelength(&self, sound_speed: float) -> float {
        sound_speed / ULTRASOUND_FREQUENCY
    }
    /// Get the wavenumber of the transducer
    pub fn wavenumber(&self, sound_speed: float) -> float {
        2.0 * PI * ULTRASOUND_FREQUENCY / sound_speed
    }
}

#[cfg(test)]
mod tests {
    use crate::defined::PI;
    use assert_approx_eq::assert_approx_eq;

    use super::*;

    macro_rules! assert_vec3_approx_eq {
        ($a:expr, $b:expr) => {
            assert_approx_eq!($a.x, $b.x, 1e-3);
            assert_approx_eq!($a.y, $b.y, 1e-3);
            assert_approx_eq!($a.z, $b.z, 1e-3);
        };
    }

    #[test]
    fn idx() {
        let tr = Transducer::new(0, Vector3::zeros(), UnitQuaternion::identity());
        assert_eq!(0, tr.idx());

        let tr = Transducer::new(1, Vector3::zeros(), UnitQuaternion::identity());
        assert_eq!(1, tr.idx());
    }

    #[test]
    fn affine() {
        let mut tr = Transducer::new(0, Vector3::zeros(), UnitQuaternion::identity());

        let t = Vector3::new(40., 50., 60.);
        let rot = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.);
        tr.affine(t, rot);

        let expect_x = Vector3::new(0., 1., 0.);
        let expect_y = Vector3::new(-1., 0., 0.);
        let expect_z = Vector3::new(0., 0., 1.);
        assert_vec3_approx_eq!(expect_x, tr.x_direction());
        assert_vec3_approx_eq!(expect_y, tr.y_direction());
        assert_vec3_approx_eq!(expect_z, tr.z_direction());

        let expect_pos = Vector3::zeros() + t;
        assert_vec3_approx_eq!(expect_pos, tr.position());
    }

    #[test]
    fn wavelength() {
        let tr = Transducer::new(0, Vector3::zeros(), UnitQuaternion::identity());
        let c = 340e3;
        assert_approx_eq!(c / ULTRASOUND_FREQUENCY, tr.wavelength(c));
    }

    #[test]
    fn wavenumber() {
        let tr = Transducer::new(0, Vector3::zeros(), UnitQuaternion::identity());
        let c = 340e3;
        assert_approx_eq!(2. * PI * ULTRASOUND_FREQUENCY / c, tr.wavenumber(c));
    }

    #[test]
    fn align_phase_at() {
        let tr = Transducer::new(0, Vector3::zeros(), UnitQuaternion::identity());

        let c = 340e3;
        let wavelength = tr.wavelength(c);

        let p = Vector3::zeros();
        assert_eq!(0, tr.align_phase_at(p, c).value());

        let p = Vector3::new(wavelength, 0., 0.);
        assert_eq!(0, tr.align_phase_at(p, c).value());

        let p = Vector3::new(0., -wavelength, 0.);
        assert_eq!(0, tr.align_phase_at(p, c).value());

        let p = Vector3::new(0., 0., wavelength / 2.);
        assert_eq!(128, tr.align_phase_at(p, c).value());
    }
}
