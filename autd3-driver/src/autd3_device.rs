/*
 * File: autd3_device.rs
 * Project: src
 * Created Date: 06/12/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 29/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use crate::{
    defined::{float, MILLIMETER},
    geometry::{Device, IntoDevice, Matrix4, Transducer, UnitQuaternion, Vector3, Vector4},
};

/// AUTD3 device
pub struct AUTD3 {
    position: Vector3,
    rotation: UnitQuaternion,
}

impl AUTD3 {
    /// Number of transducer in an AUTD3 device
    pub const NUM_TRANS_IN_UNIT: usize = 249;
    /// Number of transducer in x-axis of AUTD3 device
    pub const NUM_TRANS_X: usize = 18;
    /// Number of transducer in y-axis of AUTD3 device
    pub const NUM_TRANS_Y: usize = 14;
    /// Spacing between transducers in mm
    pub const TRANS_SPACING_MM: float = 10.16;
    /// Spacing between transducers
    pub const TRANS_SPACING: float = Self::TRANS_SPACING_MM * MILLIMETER;
    /// Device width including substrate
    pub const DEVICE_WIDTH: float = 192.0 * MILLIMETER;
    /// Device height including substrate
    pub const DEVICE_HEIGHT: float = 151.4 * MILLIMETER;

    /// Constructor
    ///
    /// # Arguments
    ///
    /// * `position` - Global position
    ///
    pub fn new(position: Vector3) -> Self {
        Self {
            position,
            rotation: UnitQuaternion::identity(),
        }
    }

    pub fn with_rotation<Q: Into<UnitQuaternion>>(self, rotation: Q) -> Self {
        Self {
            rotation: rotation.into(),
            ..self
        }
    }

    fn is_missing_transducer<T1, T2>(x: T1, y: T2) -> bool
    where
        T1: TryInto<u8> + PartialEq<T1>,
        T2: TryInto<u8> + PartialEq<T2>,
    {
        let x = match x.try_into() {
            Ok(v) => v,
            Err(_) => return true,
        };
        let y = match y.try_into() {
            Ok(v) => v,
            Err(_) => return true,
        };
        if 17 < x || 14 < y {
            return true;
        }

        y == 1 && (x == 1 || x == 2 || x == 16)
    }

    /// Get grid id from transducer id
    ///
    /// # Arguments
    ///
    /// * `idx` - Transducer index
    ///
    /// # Examples
    ///
    /// ```
    /// use autd3_driver::autd3_device::AUTD3;
    ///
    /// let (x, y) = AUTD3::grid_id(0);
    /// assert_eq!(x, 0);
    /// assert_eq!(y, 0);
    ///
    /// let (x, y) = AUTD3::grid_id(248);
    /// assert_eq!(x, 17);
    /// assert_eq!(y, 13);
    /// ```
    ///
    pub const fn grid_id(idx: usize) -> (usize, usize) {
        let local_id = idx % Self::NUM_TRANS_IN_UNIT;
        let mut offset = 0;
        if local_id >= 19 {
            offset += 2;
        }
        if local_id >= 32 {
            offset += 1;
        }
        let uid = local_id + offset;
        (uid % Self::NUM_TRANS_X, uid / Self::NUM_TRANS_X)
    }
}

impl IntoDevice for AUTD3 {
    fn into_device(self, dev_idx: usize) -> Device {
        let rot_mat: Matrix4 = From::from(self.rotation);
        let trans_mat = rot_mat.append_translation(&self.position);
        Device::new(
            dev_idx,
            (0..Self::NUM_TRANS_Y)
                .flat_map(|y| (0..Self::NUM_TRANS_X).map(move |x| (x, y)))
                .filter(|&(x, y)| !Self::is_missing_transducer(x, y))
                .map(|(x, y)| {
                    Vector4::new(
                        x as float * Self::TRANS_SPACING,
                        y as float * Self::TRANS_SPACING,
                        0.,
                        1.,
                    )
                })
                .map(|p| trans_mat * p)
                .zip(0..)
                .map(|(p, i)| Transducer::new(i, Vector3::new(p.x, p.y, p.z), self.rotation))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::geometry::{Deg, EulerAngle};

    use super::*;

    #[test]
    fn autd3_device() {
        let dev = AUTD3::new(Vector3::zeros());
        let dev: Device = dev.into_device(0);
        assert_eq!(dev.num_transducers(), 249);

        assert_approx_eq::assert_approx_eq!(dev[0].position().x, 0.);
        assert_approx_eq::assert_approx_eq!(dev[0].position().y, 0.);
        assert_approx_eq::assert_approx_eq!(dev[1].position().x, AUTD3::TRANS_SPACING);
        assert_approx_eq::assert_approx_eq!(dev[1].position().y, 0.);
        assert_approx_eq::assert_approx_eq!(dev[18].position().x, 0.);
        assert_approx_eq::assert_approx_eq!(dev[18].position().y, AUTD3::TRANS_SPACING);
    }

    #[test]
    fn autd3_device_with_rotation() {
        let mut rng = rand::thread_rng();
        let q = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), rng.gen())
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), rng.gen())
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), rng.gen());
        let dev = AUTD3::new(Vector3::zeros()).with_rotation(q);
        assert_eq!(dev.rotation, q);

        let dev = AUTD3::new(Vector3::zeros()).with_rotation(EulerAngle::ZYZ(
            0. * Deg,
            0. * Deg,
            0. * Deg,
        ));
        assert_eq!(dev.rotation, UnitQuaternion::identity());
    }

    #[test]
    fn autd3_is_missing_transducer() {
        assert!(AUTD3::is_missing_transducer(1, 1));
        assert!(AUTD3::is_missing_transducer(2, 1));
        assert!(AUTD3::is_missing_transducer(16, 1));

        assert!(!AUTD3::is_missing_transducer(0, 0));
        assert!(!AUTD3::is_missing_transducer(1, 0));
        assert!(!AUTD3::is_missing_transducer(2, 0));
        assert!(!AUTD3::is_missing_transducer(3, 0));
        assert!(!AUTD3::is_missing_transducer(4, 0));
        assert!(!AUTD3::is_missing_transducer(5, 0));
        assert!(!AUTD3::is_missing_transducer(6, 0));
        assert!(!AUTD3::is_missing_transducer(7, 0));
        assert!(!AUTD3::is_missing_transducer(8, 0));
        assert!(!AUTD3::is_missing_transducer(9, 0));
        assert!(!AUTD3::is_missing_transducer(10, 0));
        assert!(!AUTD3::is_missing_transducer(11, 0));
        assert!(!AUTD3::is_missing_transducer(12, 0));
        assert!(!AUTD3::is_missing_transducer(13, 0));
        assert!(!AUTD3::is_missing_transducer(14, 0));
        assert!(!AUTD3::is_missing_transducer(15, 0));
        assert!(!AUTD3::is_missing_transducer(16, 0));
        assert!(!AUTD3::is_missing_transducer(17, 0));
        assert!(!AUTD3::is_missing_transducer(0, 1));
        assert!(AUTD3::is_missing_transducer(1, 1));
        assert!(AUTD3::is_missing_transducer(2, 1));
        assert!(!AUTD3::is_missing_transducer(3, 1));
        assert!(!AUTD3::is_missing_transducer(4, 1));
        assert!(!AUTD3::is_missing_transducer(5, 1));
        assert!(!AUTD3::is_missing_transducer(6, 1));
        assert!(!AUTD3::is_missing_transducer(7, 1));
        assert!(!AUTD3::is_missing_transducer(8, 1));
        assert!(!AUTD3::is_missing_transducer(9, 1));
        assert!(!AUTD3::is_missing_transducer(10, 1));
        assert!(!AUTD3::is_missing_transducer(11, 1));
        assert!(!AUTD3::is_missing_transducer(12, 1));
        assert!(!AUTD3::is_missing_transducer(13, 1));
        assert!(!AUTD3::is_missing_transducer(14, 1));
        assert!(!AUTD3::is_missing_transducer(15, 1));
        assert!(AUTD3::is_missing_transducer(16, 1));
        assert!(!AUTD3::is_missing_transducer(17, 1));
        assert!(!AUTD3::is_missing_transducer(0, 2));
        assert!(!AUTD3::is_missing_transducer(1, 2));
        assert!(!AUTD3::is_missing_transducer(2, 2));
        assert!(!AUTD3::is_missing_transducer(3, 2));
        assert!(!AUTD3::is_missing_transducer(4, 2));
        assert!(!AUTD3::is_missing_transducer(5, 2));
        assert!(!AUTD3::is_missing_transducer(6, 2));
        assert!(!AUTD3::is_missing_transducer(7, 2));
        assert!(!AUTD3::is_missing_transducer(8, 2));
        assert!(!AUTD3::is_missing_transducer(9, 2));
        assert!(!AUTD3::is_missing_transducer(10, 2));
        assert!(!AUTD3::is_missing_transducer(11, 2));
        assert!(!AUTD3::is_missing_transducer(12, 2));
        assert!(!AUTD3::is_missing_transducer(13, 2));
        assert!(!AUTD3::is_missing_transducer(14, 2));
        assert!(!AUTD3::is_missing_transducer(15, 2));
        assert!(!AUTD3::is_missing_transducer(16, 2));
        assert!(!AUTD3::is_missing_transducer(17, 2));
        assert!(!AUTD3::is_missing_transducer(0, 3));
        assert!(!AUTD3::is_missing_transducer(1, 3));
        assert!(!AUTD3::is_missing_transducer(2, 3));
        assert!(!AUTD3::is_missing_transducer(3, 3));
        assert!(!AUTD3::is_missing_transducer(4, 3));
        assert!(!AUTD3::is_missing_transducer(5, 3));
        assert!(!AUTD3::is_missing_transducer(6, 3));
        assert!(!AUTD3::is_missing_transducer(7, 3));
        assert!(!AUTD3::is_missing_transducer(8, 3));
        assert!(!AUTD3::is_missing_transducer(9, 3));
        assert!(!AUTD3::is_missing_transducer(10, 3));
        assert!(!AUTD3::is_missing_transducer(11, 3));
        assert!(!AUTD3::is_missing_transducer(12, 3));
        assert!(!AUTD3::is_missing_transducer(13, 3));
        assert!(!AUTD3::is_missing_transducer(14, 3));
        assert!(!AUTD3::is_missing_transducer(15, 3));
        assert!(!AUTD3::is_missing_transducer(16, 3));
        assert!(!AUTD3::is_missing_transducer(17, 3));
        assert!(!AUTD3::is_missing_transducer(0, 4));
        assert!(!AUTD3::is_missing_transducer(1, 4));
        assert!(!AUTD3::is_missing_transducer(2, 4));
        assert!(!AUTD3::is_missing_transducer(3, 4));
        assert!(!AUTD3::is_missing_transducer(4, 4));
        assert!(!AUTD3::is_missing_transducer(5, 4));
        assert!(!AUTD3::is_missing_transducer(6, 4));
        assert!(!AUTD3::is_missing_transducer(7, 4));
        assert!(!AUTD3::is_missing_transducer(8, 4));
        assert!(!AUTD3::is_missing_transducer(9, 4));
        assert!(!AUTD3::is_missing_transducer(10, 4));
        assert!(!AUTD3::is_missing_transducer(11, 4));
        assert!(!AUTD3::is_missing_transducer(12, 4));
        assert!(!AUTD3::is_missing_transducer(13, 4));
        assert!(!AUTD3::is_missing_transducer(14, 4));
        assert!(!AUTD3::is_missing_transducer(15, 4));
        assert!(!AUTD3::is_missing_transducer(16, 4));
        assert!(!AUTD3::is_missing_transducer(17, 4));
        assert!(!AUTD3::is_missing_transducer(0, 5));
        assert!(!AUTD3::is_missing_transducer(1, 5));
        assert!(!AUTD3::is_missing_transducer(2, 5));
        assert!(!AUTD3::is_missing_transducer(3, 5));
        assert!(!AUTD3::is_missing_transducer(4, 5));
        assert!(!AUTD3::is_missing_transducer(5, 5));
        assert!(!AUTD3::is_missing_transducer(6, 5));
        assert!(!AUTD3::is_missing_transducer(7, 5));
        assert!(!AUTD3::is_missing_transducer(8, 5));
        assert!(!AUTD3::is_missing_transducer(9, 5));
        assert!(!AUTD3::is_missing_transducer(10, 5));
        assert!(!AUTD3::is_missing_transducer(11, 5));
        assert!(!AUTD3::is_missing_transducer(12, 5));
        assert!(!AUTD3::is_missing_transducer(13, 5));
        assert!(!AUTD3::is_missing_transducer(14, 5));
        assert!(!AUTD3::is_missing_transducer(15, 5));
        assert!(!AUTD3::is_missing_transducer(16, 5));
        assert!(!AUTD3::is_missing_transducer(17, 5));
        assert!(!AUTD3::is_missing_transducer(0, 6));
        assert!(!AUTD3::is_missing_transducer(1, 6));
        assert!(!AUTD3::is_missing_transducer(2, 6));
        assert!(!AUTD3::is_missing_transducer(3, 6));
        assert!(!AUTD3::is_missing_transducer(4, 6));
        assert!(!AUTD3::is_missing_transducer(5, 6));
        assert!(!AUTD3::is_missing_transducer(6, 6));
        assert!(!AUTD3::is_missing_transducer(7, 6));
        assert!(!AUTD3::is_missing_transducer(8, 6));
        assert!(!AUTD3::is_missing_transducer(9, 6));
        assert!(!AUTD3::is_missing_transducer(10, 6));
        assert!(!AUTD3::is_missing_transducer(11, 6));
        assert!(!AUTD3::is_missing_transducer(12, 6));
        assert!(!AUTD3::is_missing_transducer(13, 6));
        assert!(!AUTD3::is_missing_transducer(14, 6));
        assert!(!AUTD3::is_missing_transducer(15, 6));
        assert!(!AUTD3::is_missing_transducer(16, 6));
        assert!(!AUTD3::is_missing_transducer(17, 6));
        assert!(!AUTD3::is_missing_transducer(0, 7));
        assert!(!AUTD3::is_missing_transducer(1, 7));
        assert!(!AUTD3::is_missing_transducer(2, 7));
        assert!(!AUTD3::is_missing_transducer(3, 7));
        assert!(!AUTD3::is_missing_transducer(4, 7));
        assert!(!AUTD3::is_missing_transducer(5, 7));
        assert!(!AUTD3::is_missing_transducer(6, 7));
        assert!(!AUTD3::is_missing_transducer(7, 7));
        assert!(!AUTD3::is_missing_transducer(8, 7));
        assert!(!AUTD3::is_missing_transducer(9, 7));
        assert!(!AUTD3::is_missing_transducer(10, 7));
        assert!(!AUTD3::is_missing_transducer(11, 7));
        assert!(!AUTD3::is_missing_transducer(12, 7));
        assert!(!AUTD3::is_missing_transducer(13, 7));
        assert!(!AUTD3::is_missing_transducer(14, 7));
        assert!(!AUTD3::is_missing_transducer(15, 7));
        assert!(!AUTD3::is_missing_transducer(16, 7));
        assert!(!AUTD3::is_missing_transducer(17, 7));
        assert!(!AUTD3::is_missing_transducer(0, 8));
        assert!(!AUTD3::is_missing_transducer(1, 8));
        assert!(!AUTD3::is_missing_transducer(2, 8));
        assert!(!AUTD3::is_missing_transducer(3, 8));
        assert!(!AUTD3::is_missing_transducer(4, 8));
        assert!(!AUTD3::is_missing_transducer(5, 8));
        assert!(!AUTD3::is_missing_transducer(6, 8));
        assert!(!AUTD3::is_missing_transducer(7, 8));
        assert!(!AUTD3::is_missing_transducer(8, 8));
        assert!(!AUTD3::is_missing_transducer(9, 8));
        assert!(!AUTD3::is_missing_transducer(10, 8));
        assert!(!AUTD3::is_missing_transducer(11, 8));
        assert!(!AUTD3::is_missing_transducer(12, 8));
        assert!(!AUTD3::is_missing_transducer(13, 8));
        assert!(!AUTD3::is_missing_transducer(14, 8));
        assert!(!AUTD3::is_missing_transducer(15, 8));
        assert!(!AUTD3::is_missing_transducer(16, 8));
        assert!(!AUTD3::is_missing_transducer(17, 8));
        assert!(!AUTD3::is_missing_transducer(0, 9));
        assert!(!AUTD3::is_missing_transducer(1, 9));
        assert!(!AUTD3::is_missing_transducer(2, 9));
        assert!(!AUTD3::is_missing_transducer(3, 9));
        assert!(!AUTD3::is_missing_transducer(4, 9));
        assert!(!AUTD3::is_missing_transducer(5, 9));
        assert!(!AUTD3::is_missing_transducer(6, 9));
        assert!(!AUTD3::is_missing_transducer(7, 9));
        assert!(!AUTD3::is_missing_transducer(8, 9));
        assert!(!AUTD3::is_missing_transducer(9, 9));
        assert!(!AUTD3::is_missing_transducer(10, 9));
        assert!(!AUTD3::is_missing_transducer(11, 9));
        assert!(!AUTD3::is_missing_transducer(12, 9));
        assert!(!AUTD3::is_missing_transducer(13, 9));
        assert!(!AUTD3::is_missing_transducer(14, 9));
        assert!(!AUTD3::is_missing_transducer(15, 9));
        assert!(!AUTD3::is_missing_transducer(16, 9));
        assert!(!AUTD3::is_missing_transducer(17, 9));
        assert!(!AUTD3::is_missing_transducer(0, 10));
        assert!(!AUTD3::is_missing_transducer(1, 10));
        assert!(!AUTD3::is_missing_transducer(2, 10));
        assert!(!AUTD3::is_missing_transducer(3, 10));
        assert!(!AUTD3::is_missing_transducer(4, 10));
        assert!(!AUTD3::is_missing_transducer(5, 10));
        assert!(!AUTD3::is_missing_transducer(6, 10));
        assert!(!AUTD3::is_missing_transducer(7, 10));
        assert!(!AUTD3::is_missing_transducer(8, 10));
        assert!(!AUTD3::is_missing_transducer(9, 10));
        assert!(!AUTD3::is_missing_transducer(10, 10));
        assert!(!AUTD3::is_missing_transducer(11, 10));
        assert!(!AUTD3::is_missing_transducer(12, 10));
        assert!(!AUTD3::is_missing_transducer(13, 10));
        assert!(!AUTD3::is_missing_transducer(14, 10));
        assert!(!AUTD3::is_missing_transducer(15, 10));
        assert!(!AUTD3::is_missing_transducer(16, 10));
        assert!(!AUTD3::is_missing_transducer(17, 10));
        assert!(!AUTD3::is_missing_transducer(0, 11));
        assert!(!AUTD3::is_missing_transducer(1, 11));
        assert!(!AUTD3::is_missing_transducer(2, 11));
        assert!(!AUTD3::is_missing_transducer(3, 11));
        assert!(!AUTD3::is_missing_transducer(4, 11));
        assert!(!AUTD3::is_missing_transducer(5, 11));
        assert!(!AUTD3::is_missing_transducer(6, 11));
        assert!(!AUTD3::is_missing_transducer(7, 11));
        assert!(!AUTD3::is_missing_transducer(8, 11));
        assert!(!AUTD3::is_missing_transducer(9, 11));
        assert!(!AUTD3::is_missing_transducer(10, 11));
        assert!(!AUTD3::is_missing_transducer(11, 11));
        assert!(!AUTD3::is_missing_transducer(12, 11));
        assert!(!AUTD3::is_missing_transducer(13, 11));
        assert!(!AUTD3::is_missing_transducer(14, 11));
        assert!(!AUTD3::is_missing_transducer(15, 11));
        assert!(!AUTD3::is_missing_transducer(16, 11));
        assert!(!AUTD3::is_missing_transducer(17, 11));
        assert!(!AUTD3::is_missing_transducer(0, 12));
        assert!(!AUTD3::is_missing_transducer(1, 12));
        assert!(!AUTD3::is_missing_transducer(2, 12));
        assert!(!AUTD3::is_missing_transducer(3, 12));
        assert!(!AUTD3::is_missing_transducer(4, 12));
        assert!(!AUTD3::is_missing_transducer(5, 12));
        assert!(!AUTD3::is_missing_transducer(6, 12));
        assert!(!AUTD3::is_missing_transducer(7, 12));
        assert!(!AUTD3::is_missing_transducer(8, 12));
        assert!(!AUTD3::is_missing_transducer(9, 12));
        assert!(!AUTD3::is_missing_transducer(10, 12));
        assert!(!AUTD3::is_missing_transducer(11, 12));
        assert!(!AUTD3::is_missing_transducer(12, 12));
        assert!(!AUTD3::is_missing_transducer(13, 12));
        assert!(!AUTD3::is_missing_transducer(14, 12));
        assert!(!AUTD3::is_missing_transducer(15, 12));
        assert!(!AUTD3::is_missing_transducer(16, 12));
        assert!(!AUTD3::is_missing_transducer(17, 12));
        assert!(!AUTD3::is_missing_transducer(1, 13));
        assert!(!AUTD3::is_missing_transducer(2, 13));
        assert!(!AUTD3::is_missing_transducer(3, 13));
        assert!(!AUTD3::is_missing_transducer(4, 13));
        assert!(!AUTD3::is_missing_transducer(5, 13));
        assert!(!AUTD3::is_missing_transducer(6, 13));
        assert!(!AUTD3::is_missing_transducer(7, 13));
        assert!(!AUTD3::is_missing_transducer(8, 13));
        assert!(!AUTD3::is_missing_transducer(9, 13));
        assert!(!AUTD3::is_missing_transducer(10, 13));
        assert!(!AUTD3::is_missing_transducer(11, 13));
        assert!(!AUTD3::is_missing_transducer(12, 13));
        assert!(!AUTD3::is_missing_transducer(13, 13));
        assert!(!AUTD3::is_missing_transducer(14, 13));
        assert!(!AUTD3::is_missing_transducer(15, 13));
        assert!(!AUTD3::is_missing_transducer(16, 13));
        assert!(!AUTD3::is_missing_transducer(17, 13));

        for x in 18..=255 {
            for y in 14..=255 {
                assert!(AUTD3::is_missing_transducer(x, y));
            }
        }

        assert!(!AUTD3::is_missing_transducer(0i8, 0i8));
        assert!(!AUTD3::is_missing_transducer(0i16, 0i16));
        assert!(!AUTD3::is_missing_transducer(0i32, 0i32));
        assert!(!AUTD3::is_missing_transducer(0i64, 0i64));
        assert!(!AUTD3::is_missing_transducer(0i128, 0i128));
        assert!(!AUTD3::is_missing_transducer(0u8, 0u8));
        assert!(!AUTD3::is_missing_transducer(0u16, 0u16));
        assert!(!AUTD3::is_missing_transducer(0u32, 0u32));
        assert!(!AUTD3::is_missing_transducer(0u64, 0u64));
        assert!(!AUTD3::is_missing_transducer(0u128, 0u128));
    }

    #[test]
    fn autd3_grid_id() {
        assert_eq!(AUTD3::grid_id(0).0, 0);
        assert_eq!(AUTD3::grid_id(0).1, 0);
        assert_eq!(AUTD3::grid_id(1).0, 1);
        assert_eq!(AUTD3::grid_id(1).1, 0);
        assert_eq!(AUTD3::grid_id(2).0, 2);
        assert_eq!(AUTD3::grid_id(2).1, 0);
        assert_eq!(AUTD3::grid_id(3).0, 3);
        assert_eq!(AUTD3::grid_id(3).1, 0);
        assert_eq!(AUTD3::grid_id(4).0, 4);
        assert_eq!(AUTD3::grid_id(4).1, 0);
        assert_eq!(AUTD3::grid_id(5).0, 5);
        assert_eq!(AUTD3::grid_id(5).1, 0);
        assert_eq!(AUTD3::grid_id(6).0, 6);
        assert_eq!(AUTD3::grid_id(6).1, 0);
        assert_eq!(AUTD3::grid_id(7).0, 7);
        assert_eq!(AUTD3::grid_id(7).1, 0);
        assert_eq!(AUTD3::grid_id(8).0, 8);
        assert_eq!(AUTD3::grid_id(8).1, 0);
        assert_eq!(AUTD3::grid_id(9).0, 9);
        assert_eq!(AUTD3::grid_id(9).1, 0);
        assert_eq!(AUTD3::grid_id(10).0, 10);
        assert_eq!(AUTD3::grid_id(10).1, 0);
        assert_eq!(AUTD3::grid_id(11).0, 11);
        assert_eq!(AUTD3::grid_id(11).1, 0);
        assert_eq!(AUTD3::grid_id(12).0, 12);
        assert_eq!(AUTD3::grid_id(12).1, 0);
        assert_eq!(AUTD3::grid_id(13).0, 13);
        assert_eq!(AUTD3::grid_id(13).1, 0);
        assert_eq!(AUTD3::grid_id(14).0, 14);
        assert_eq!(AUTD3::grid_id(14).1, 0);
        assert_eq!(AUTD3::grid_id(15).0, 15);
        assert_eq!(AUTD3::grid_id(15).1, 0);
        assert_eq!(AUTD3::grid_id(16).0, 16);
        assert_eq!(AUTD3::grid_id(16).1, 0);
        assert_eq!(AUTD3::grid_id(17).0, 17);
        assert_eq!(AUTD3::grid_id(17).1, 0);
        assert_eq!(AUTD3::grid_id(18).0, 0);
        assert_eq!(AUTD3::grid_id(18).1, 1);
        assert_eq!(AUTD3::grid_id(19).0, 3);
        assert_eq!(AUTD3::grid_id(19).1, 1);
        assert_eq!(AUTD3::grid_id(20).0, 4);
        assert_eq!(AUTD3::grid_id(20).1, 1);
        assert_eq!(AUTD3::grid_id(21).0, 5);
        assert_eq!(AUTD3::grid_id(21).1, 1);
        assert_eq!(AUTD3::grid_id(22).0, 6);
        assert_eq!(AUTD3::grid_id(22).1, 1);
        assert_eq!(AUTD3::grid_id(23).0, 7);
        assert_eq!(AUTD3::grid_id(23).1, 1);
        assert_eq!(AUTD3::grid_id(24).0, 8);
        assert_eq!(AUTD3::grid_id(24).1, 1);
        assert_eq!(AUTD3::grid_id(25).0, 9);
        assert_eq!(AUTD3::grid_id(25).1, 1);
        assert_eq!(AUTD3::grid_id(26).0, 10);
        assert_eq!(AUTD3::grid_id(26).1, 1);
        assert_eq!(AUTD3::grid_id(27).0, 11);
        assert_eq!(AUTD3::grid_id(27).1, 1);
        assert_eq!(AUTD3::grid_id(28).0, 12);
        assert_eq!(AUTD3::grid_id(28).1, 1);
        assert_eq!(AUTD3::grid_id(29).0, 13);
        assert_eq!(AUTD3::grid_id(29).1, 1);
        assert_eq!(AUTD3::grid_id(30).0, 14);
        assert_eq!(AUTD3::grid_id(30).1, 1);
        assert_eq!(AUTD3::grid_id(31).0, 15);
        assert_eq!(AUTD3::grid_id(31).1, 1);
        assert_eq!(AUTD3::grid_id(32).0, 17);
        assert_eq!(AUTD3::grid_id(32).1, 1);
        assert_eq!(AUTD3::grid_id(33).0, 0);
        assert_eq!(AUTD3::grid_id(33).1, 2);
        assert_eq!(AUTD3::grid_id(34).0, 1);
        assert_eq!(AUTD3::grid_id(34).1, 2);
        assert_eq!(AUTD3::grid_id(35).0, 2);
        assert_eq!(AUTD3::grid_id(35).1, 2);
        assert_eq!(AUTD3::grid_id(36).0, 3);
        assert_eq!(AUTD3::grid_id(36).1, 2);
        assert_eq!(AUTD3::grid_id(37).0, 4);
        assert_eq!(AUTD3::grid_id(37).1, 2);
        assert_eq!(AUTD3::grid_id(38).0, 5);
        assert_eq!(AUTD3::grid_id(38).1, 2);
        assert_eq!(AUTD3::grid_id(39).0, 6);
        assert_eq!(AUTD3::grid_id(39).1, 2);
        assert_eq!(AUTD3::grid_id(40).0, 7);
        assert_eq!(AUTD3::grid_id(40).1, 2);
        assert_eq!(AUTD3::grid_id(41).0, 8);
        assert_eq!(AUTD3::grid_id(41).1, 2);
        assert_eq!(AUTD3::grid_id(42).0, 9);
        assert_eq!(AUTD3::grid_id(42).1, 2);
        assert_eq!(AUTD3::grid_id(43).0, 10);
        assert_eq!(AUTD3::grid_id(43).1, 2);
        assert_eq!(AUTD3::grid_id(44).0, 11);
        assert_eq!(AUTD3::grid_id(44).1, 2);
        assert_eq!(AUTD3::grid_id(45).0, 12);
        assert_eq!(AUTD3::grid_id(45).1, 2);
        assert_eq!(AUTD3::grid_id(46).0, 13);
        assert_eq!(AUTD3::grid_id(46).1, 2);
        assert_eq!(AUTD3::grid_id(47).0, 14);
        assert_eq!(AUTD3::grid_id(47).1, 2);
        assert_eq!(AUTD3::grid_id(48).0, 15);
        assert_eq!(AUTD3::grid_id(48).1, 2);
        assert_eq!(AUTD3::grid_id(49).0, 16);
        assert_eq!(AUTD3::grid_id(49).1, 2);
        assert_eq!(AUTD3::grid_id(50).0, 17);
        assert_eq!(AUTD3::grid_id(50).1, 2);
        assert_eq!(AUTD3::grid_id(51).0, 0);
        assert_eq!(AUTD3::grid_id(51).1, 3);
        assert_eq!(AUTD3::grid_id(52).0, 1);
        assert_eq!(AUTD3::grid_id(52).1, 3);
        assert_eq!(AUTD3::grid_id(53).0, 2);
        assert_eq!(AUTD3::grid_id(53).1, 3);
        assert_eq!(AUTD3::grid_id(54).0, 3);
        assert_eq!(AUTD3::grid_id(54).1, 3);
        assert_eq!(AUTD3::grid_id(55).0, 4);
        assert_eq!(AUTD3::grid_id(55).1, 3);
        assert_eq!(AUTD3::grid_id(56).0, 5);
        assert_eq!(AUTD3::grid_id(56).1, 3);
        assert_eq!(AUTD3::grid_id(57).0, 6);
        assert_eq!(AUTD3::grid_id(57).1, 3);
        assert_eq!(AUTD3::grid_id(58).0, 7);
        assert_eq!(AUTD3::grid_id(58).1, 3);
        assert_eq!(AUTD3::grid_id(59).0, 8);
        assert_eq!(AUTD3::grid_id(59).1, 3);
        assert_eq!(AUTD3::grid_id(60).0, 9);
        assert_eq!(AUTD3::grid_id(60).1, 3);
        assert_eq!(AUTD3::grid_id(61).0, 10);
        assert_eq!(AUTD3::grid_id(61).1, 3);
        assert_eq!(AUTD3::grid_id(62).0, 11);
        assert_eq!(AUTD3::grid_id(62).1, 3);
        assert_eq!(AUTD3::grid_id(63).0, 12);
        assert_eq!(AUTD3::grid_id(63).1, 3);
        assert_eq!(AUTD3::grid_id(64).0, 13);
        assert_eq!(AUTD3::grid_id(64).1, 3);
        assert_eq!(AUTD3::grid_id(65).0, 14);
        assert_eq!(AUTD3::grid_id(65).1, 3);
        assert_eq!(AUTD3::grid_id(66).0, 15);
        assert_eq!(AUTD3::grid_id(66).1, 3);
        assert_eq!(AUTD3::grid_id(67).0, 16);
        assert_eq!(AUTD3::grid_id(67).1, 3);
        assert_eq!(AUTD3::grid_id(68).0, 17);
        assert_eq!(AUTD3::grid_id(68).1, 3);
        assert_eq!(AUTD3::grid_id(69).0, 0);
        assert_eq!(AUTD3::grid_id(69).1, 4);
        assert_eq!(AUTD3::grid_id(70).0, 1);
        assert_eq!(AUTD3::grid_id(70).1, 4);
        assert_eq!(AUTD3::grid_id(71).0, 2);
        assert_eq!(AUTD3::grid_id(71).1, 4);
        assert_eq!(AUTD3::grid_id(72).0, 3);
        assert_eq!(AUTD3::grid_id(72).1, 4);
        assert_eq!(AUTD3::grid_id(73).0, 4);
        assert_eq!(AUTD3::grid_id(73).1, 4);
        assert_eq!(AUTD3::grid_id(74).0, 5);
        assert_eq!(AUTD3::grid_id(74).1, 4);
        assert_eq!(AUTD3::grid_id(75).0, 6);
        assert_eq!(AUTD3::grid_id(75).1, 4);
        assert_eq!(AUTD3::grid_id(76).0, 7);
        assert_eq!(AUTD3::grid_id(76).1, 4);
        assert_eq!(AUTD3::grid_id(77).0, 8);
        assert_eq!(AUTD3::grid_id(77).1, 4);
        assert_eq!(AUTD3::grid_id(78).0, 9);
        assert_eq!(AUTD3::grid_id(78).1, 4);
        assert_eq!(AUTD3::grid_id(79).0, 10);
        assert_eq!(AUTD3::grid_id(79).1, 4);
        assert_eq!(AUTD3::grid_id(80).0, 11);
        assert_eq!(AUTD3::grid_id(80).1, 4);
        assert_eq!(AUTD3::grid_id(81).0, 12);
        assert_eq!(AUTD3::grid_id(81).1, 4);
        assert_eq!(AUTD3::grid_id(82).0, 13);
        assert_eq!(AUTD3::grid_id(82).1, 4);
        assert_eq!(AUTD3::grid_id(83).0, 14);
        assert_eq!(AUTD3::grid_id(83).1, 4);
        assert_eq!(AUTD3::grid_id(84).0, 15);
        assert_eq!(AUTD3::grid_id(84).1, 4);
        assert_eq!(AUTD3::grid_id(85).0, 16);
        assert_eq!(AUTD3::grid_id(85).1, 4);
        assert_eq!(AUTD3::grid_id(86).0, 17);
        assert_eq!(AUTD3::grid_id(86).1, 4);
        assert_eq!(AUTD3::grid_id(87).0, 0);
        assert_eq!(AUTD3::grid_id(87).1, 5);
        assert_eq!(AUTD3::grid_id(88).0, 1);
        assert_eq!(AUTD3::grid_id(88).1, 5);
        assert_eq!(AUTD3::grid_id(89).0, 2);
        assert_eq!(AUTD3::grid_id(89).1, 5);
        assert_eq!(AUTD3::grid_id(90).0, 3);
        assert_eq!(AUTD3::grid_id(90).1, 5);
        assert_eq!(AUTD3::grid_id(91).0, 4);
        assert_eq!(AUTD3::grid_id(91).1, 5);
        assert_eq!(AUTD3::grid_id(92).0, 5);
        assert_eq!(AUTD3::grid_id(92).1, 5);
        assert_eq!(AUTD3::grid_id(93).0, 6);
        assert_eq!(AUTD3::grid_id(93).1, 5);
        assert_eq!(AUTD3::grid_id(94).0, 7);
        assert_eq!(AUTD3::grid_id(94).1, 5);
        assert_eq!(AUTD3::grid_id(95).0, 8);
        assert_eq!(AUTD3::grid_id(95).1, 5);
        assert_eq!(AUTD3::grid_id(96).0, 9);
        assert_eq!(AUTD3::grid_id(96).1, 5);
        assert_eq!(AUTD3::grid_id(97).0, 10);
        assert_eq!(AUTD3::grid_id(97).1, 5);
        assert_eq!(AUTD3::grid_id(98).0, 11);
        assert_eq!(AUTD3::grid_id(98).1, 5);
        assert_eq!(AUTD3::grid_id(99).0, 12);
        assert_eq!(AUTD3::grid_id(99).1, 5);
        assert_eq!(AUTD3::grid_id(100).0, 13);
        assert_eq!(AUTD3::grid_id(100).1, 5);
        assert_eq!(AUTD3::grid_id(101).0, 14);
        assert_eq!(AUTD3::grid_id(101).1, 5);
        assert_eq!(AUTD3::grid_id(102).0, 15);
        assert_eq!(AUTD3::grid_id(102).1, 5);
        assert_eq!(AUTD3::grid_id(103).0, 16);
        assert_eq!(AUTD3::grid_id(103).1, 5);
        assert_eq!(AUTD3::grid_id(104).0, 17);
        assert_eq!(AUTD3::grid_id(104).1, 5);
        assert_eq!(AUTD3::grid_id(105).0, 0);
        assert_eq!(AUTD3::grid_id(105).1, 6);
        assert_eq!(AUTD3::grid_id(106).0, 1);
        assert_eq!(AUTD3::grid_id(106).1, 6);
        assert_eq!(AUTD3::grid_id(107).0, 2);
        assert_eq!(AUTD3::grid_id(107).1, 6);
        assert_eq!(AUTD3::grid_id(108).0, 3);
        assert_eq!(AUTD3::grid_id(108).1, 6);
        assert_eq!(AUTD3::grid_id(109).0, 4);
        assert_eq!(AUTD3::grid_id(109).1, 6);
        assert_eq!(AUTD3::grid_id(110).0, 5);
        assert_eq!(AUTD3::grid_id(110).1, 6);
        assert_eq!(AUTD3::grid_id(111).0, 6);
        assert_eq!(AUTD3::grid_id(111).1, 6);
        assert_eq!(AUTD3::grid_id(112).0, 7);
        assert_eq!(AUTD3::grid_id(112).1, 6);
        assert_eq!(AUTD3::grid_id(113).0, 8);
        assert_eq!(AUTD3::grid_id(113).1, 6);
        assert_eq!(AUTD3::grid_id(114).0, 9);
        assert_eq!(AUTD3::grid_id(114).1, 6);
        assert_eq!(AUTD3::grid_id(115).0, 10);
        assert_eq!(AUTD3::grid_id(115).1, 6);
        assert_eq!(AUTD3::grid_id(116).0, 11);
        assert_eq!(AUTD3::grid_id(116).1, 6);
        assert_eq!(AUTD3::grid_id(117).0, 12);
        assert_eq!(AUTD3::grid_id(117).1, 6);
        assert_eq!(AUTD3::grid_id(118).0, 13);
        assert_eq!(AUTD3::grid_id(118).1, 6);
        assert_eq!(AUTD3::grid_id(119).0, 14);
        assert_eq!(AUTD3::grid_id(119).1, 6);
        assert_eq!(AUTD3::grid_id(120).0, 15);
        assert_eq!(AUTD3::grid_id(120).1, 6);
        assert_eq!(AUTD3::grid_id(121).0, 16);
        assert_eq!(AUTD3::grid_id(121).1, 6);
        assert_eq!(AUTD3::grid_id(122).0, 17);
        assert_eq!(AUTD3::grid_id(122).1, 6);
        assert_eq!(AUTD3::grid_id(123).0, 0);
        assert_eq!(AUTD3::grid_id(123).1, 7);
        assert_eq!(AUTD3::grid_id(124).0, 1);
        assert_eq!(AUTD3::grid_id(124).1, 7);
        assert_eq!(AUTD3::grid_id(125).0, 2);
        assert_eq!(AUTD3::grid_id(125).1, 7);
        assert_eq!(AUTD3::grid_id(126).0, 3);
        assert_eq!(AUTD3::grid_id(126).1, 7);
        assert_eq!(AUTD3::grid_id(127).0, 4);
        assert_eq!(AUTD3::grid_id(127).1, 7);
        assert_eq!(AUTD3::grid_id(128).0, 5);
        assert_eq!(AUTD3::grid_id(128).1, 7);
        assert_eq!(AUTD3::grid_id(129).0, 6);
        assert_eq!(AUTD3::grid_id(129).1, 7);
        assert_eq!(AUTD3::grid_id(130).0, 7);
        assert_eq!(AUTD3::grid_id(130).1, 7);
        assert_eq!(AUTD3::grid_id(131).0, 8);
        assert_eq!(AUTD3::grid_id(131).1, 7);
        assert_eq!(AUTD3::grid_id(132).0, 9);
        assert_eq!(AUTD3::grid_id(132).1, 7);
        assert_eq!(AUTD3::grid_id(133).0, 10);
        assert_eq!(AUTD3::grid_id(133).1, 7);
        assert_eq!(AUTD3::grid_id(134).0, 11);
        assert_eq!(AUTD3::grid_id(134).1, 7);
        assert_eq!(AUTD3::grid_id(135).0, 12);
        assert_eq!(AUTD3::grid_id(135).1, 7);
        assert_eq!(AUTD3::grid_id(136).0, 13);
        assert_eq!(AUTD3::grid_id(136).1, 7);
        assert_eq!(AUTD3::grid_id(137).0, 14);
        assert_eq!(AUTD3::grid_id(137).1, 7);
        assert_eq!(AUTD3::grid_id(138).0, 15);
        assert_eq!(AUTD3::grid_id(138).1, 7);
        assert_eq!(AUTD3::grid_id(139).0, 16);
        assert_eq!(AUTD3::grid_id(139).1, 7);
        assert_eq!(AUTD3::grid_id(140).0, 17);
        assert_eq!(AUTD3::grid_id(140).1, 7);
        assert_eq!(AUTD3::grid_id(141).0, 0);
        assert_eq!(AUTD3::grid_id(141).1, 8);
        assert_eq!(AUTD3::grid_id(142).0, 1);
        assert_eq!(AUTD3::grid_id(142).1, 8);
        assert_eq!(AUTD3::grid_id(143).0, 2);
        assert_eq!(AUTD3::grid_id(143).1, 8);
        assert_eq!(AUTD3::grid_id(144).0, 3);
        assert_eq!(AUTD3::grid_id(144).1, 8);
        assert_eq!(AUTD3::grid_id(145).0, 4);
        assert_eq!(AUTD3::grid_id(145).1, 8);
        assert_eq!(AUTD3::grid_id(146).0, 5);
        assert_eq!(AUTD3::grid_id(146).1, 8);
        assert_eq!(AUTD3::grid_id(147).0, 6);
        assert_eq!(AUTD3::grid_id(147).1, 8);
        assert_eq!(AUTD3::grid_id(148).0, 7);
        assert_eq!(AUTD3::grid_id(148).1, 8);
        assert_eq!(AUTD3::grid_id(149).0, 8);
        assert_eq!(AUTD3::grid_id(149).1, 8);
        assert_eq!(AUTD3::grid_id(150).0, 9);
        assert_eq!(AUTD3::grid_id(150).1, 8);
        assert_eq!(AUTD3::grid_id(151).0, 10);
        assert_eq!(AUTD3::grid_id(151).1, 8);
        assert_eq!(AUTD3::grid_id(152).0, 11);
        assert_eq!(AUTD3::grid_id(152).1, 8);
        assert_eq!(AUTD3::grid_id(153).0, 12);
        assert_eq!(AUTD3::grid_id(153).1, 8);
        assert_eq!(AUTD3::grid_id(154).0, 13);
        assert_eq!(AUTD3::grid_id(154).1, 8);
        assert_eq!(AUTD3::grid_id(155).0, 14);
        assert_eq!(AUTD3::grid_id(155).1, 8);
        assert_eq!(AUTD3::grid_id(156).0, 15);
        assert_eq!(AUTD3::grid_id(156).1, 8);
        assert_eq!(AUTD3::grid_id(157).0, 16);
        assert_eq!(AUTD3::grid_id(157).1, 8);
        assert_eq!(AUTD3::grid_id(158).0, 17);
        assert_eq!(AUTD3::grid_id(158).1, 8);
        assert_eq!(AUTD3::grid_id(159).0, 0);
        assert_eq!(AUTD3::grid_id(159).1, 9);
        assert_eq!(AUTD3::grid_id(160).0, 1);
        assert_eq!(AUTD3::grid_id(160).1, 9);
        assert_eq!(AUTD3::grid_id(161).0, 2);
        assert_eq!(AUTD3::grid_id(161).1, 9);
        assert_eq!(AUTD3::grid_id(162).0, 3);
        assert_eq!(AUTD3::grid_id(162).1, 9);
        assert_eq!(AUTD3::grid_id(163).0, 4);
        assert_eq!(AUTD3::grid_id(163).1, 9);
        assert_eq!(AUTD3::grid_id(164).0, 5);
        assert_eq!(AUTD3::grid_id(164).1, 9);
        assert_eq!(AUTD3::grid_id(165).0, 6);
        assert_eq!(AUTD3::grid_id(165).1, 9);
        assert_eq!(AUTD3::grid_id(166).0, 7);
        assert_eq!(AUTD3::grid_id(166).1, 9);
        assert_eq!(AUTD3::grid_id(167).0, 8);
        assert_eq!(AUTD3::grid_id(167).1, 9);
        assert_eq!(AUTD3::grid_id(168).0, 9);
        assert_eq!(AUTD3::grid_id(168).1, 9);
        assert_eq!(AUTD3::grid_id(169).0, 10);
        assert_eq!(AUTD3::grid_id(169).1, 9);
        assert_eq!(AUTD3::grid_id(170).0, 11);
        assert_eq!(AUTD3::grid_id(170).1, 9);
        assert_eq!(AUTD3::grid_id(171).0, 12);
        assert_eq!(AUTD3::grid_id(171).1, 9);
        assert_eq!(AUTD3::grid_id(172).0, 13);
        assert_eq!(AUTD3::grid_id(172).1, 9);
        assert_eq!(AUTD3::grid_id(173).0, 14);
        assert_eq!(AUTD3::grid_id(173).1, 9);
        assert_eq!(AUTD3::grid_id(174).0, 15);
        assert_eq!(AUTD3::grid_id(174).1, 9);
        assert_eq!(AUTD3::grid_id(175).0, 16);
        assert_eq!(AUTD3::grid_id(175).1, 9);
        assert_eq!(AUTD3::grid_id(176).0, 17);
        assert_eq!(AUTD3::grid_id(176).1, 9);
        assert_eq!(AUTD3::grid_id(177).0, 0);
        assert_eq!(AUTD3::grid_id(177).1, 10);
        assert_eq!(AUTD3::grid_id(178).0, 1);
        assert_eq!(AUTD3::grid_id(178).1, 10);
        assert_eq!(AUTD3::grid_id(179).0, 2);
        assert_eq!(AUTD3::grid_id(179).1, 10);
        assert_eq!(AUTD3::grid_id(180).0, 3);
        assert_eq!(AUTD3::grid_id(180).1, 10);
        assert_eq!(AUTD3::grid_id(181).0, 4);
        assert_eq!(AUTD3::grid_id(181).1, 10);
        assert_eq!(AUTD3::grid_id(182).0, 5);
        assert_eq!(AUTD3::grid_id(182).1, 10);
        assert_eq!(AUTD3::grid_id(183).0, 6);
        assert_eq!(AUTD3::grid_id(183).1, 10);
        assert_eq!(AUTD3::grid_id(184).0, 7);
        assert_eq!(AUTD3::grid_id(184).1, 10);
        assert_eq!(AUTD3::grid_id(185).0, 8);
        assert_eq!(AUTD3::grid_id(185).1, 10);
        assert_eq!(AUTD3::grid_id(186).0, 9);
        assert_eq!(AUTD3::grid_id(186).1, 10);
        assert_eq!(AUTD3::grid_id(187).0, 10);
        assert_eq!(AUTD3::grid_id(187).1, 10);
        assert_eq!(AUTD3::grid_id(188).0, 11);
        assert_eq!(AUTD3::grid_id(188).1, 10);
        assert_eq!(AUTD3::grid_id(189).0, 12);
        assert_eq!(AUTD3::grid_id(189).1, 10);
        assert_eq!(AUTD3::grid_id(190).0, 13);
        assert_eq!(AUTD3::grid_id(190).1, 10);
        assert_eq!(AUTD3::grid_id(191).0, 14);
        assert_eq!(AUTD3::grid_id(191).1, 10);
        assert_eq!(AUTD3::grid_id(192).0, 15);
        assert_eq!(AUTD3::grid_id(192).1, 10);
        assert_eq!(AUTD3::grid_id(193).0, 16);
        assert_eq!(AUTD3::grid_id(193).1, 10);
        assert_eq!(AUTD3::grid_id(194).0, 17);
        assert_eq!(AUTD3::grid_id(194).1, 10);
        assert_eq!(AUTD3::grid_id(195).0, 0);
        assert_eq!(AUTD3::grid_id(195).1, 11);
        assert_eq!(AUTD3::grid_id(196).0, 1);
        assert_eq!(AUTD3::grid_id(196).1, 11);
        assert_eq!(AUTD3::grid_id(197).0, 2);
        assert_eq!(AUTD3::grid_id(197).1, 11);
        assert_eq!(AUTD3::grid_id(198).0, 3);
        assert_eq!(AUTD3::grid_id(198).1, 11);
        assert_eq!(AUTD3::grid_id(199).0, 4);
        assert_eq!(AUTD3::grid_id(199).1, 11);
        assert_eq!(AUTD3::grid_id(200).0, 5);
        assert_eq!(AUTD3::grid_id(200).1, 11);
        assert_eq!(AUTD3::grid_id(201).0, 6);
        assert_eq!(AUTD3::grid_id(201).1, 11);
        assert_eq!(AUTD3::grid_id(202).0, 7);
        assert_eq!(AUTD3::grid_id(202).1, 11);
        assert_eq!(AUTD3::grid_id(203).0, 8);
        assert_eq!(AUTD3::grid_id(203).1, 11);
        assert_eq!(AUTD3::grid_id(204).0, 9);
        assert_eq!(AUTD3::grid_id(204).1, 11);
        assert_eq!(AUTD3::grid_id(205).0, 10);
        assert_eq!(AUTD3::grid_id(205).1, 11);
        assert_eq!(AUTD3::grid_id(206).0, 11);
        assert_eq!(AUTD3::grid_id(206).1, 11);
        assert_eq!(AUTD3::grid_id(207).0, 12);
        assert_eq!(AUTD3::grid_id(207).1, 11);
        assert_eq!(AUTD3::grid_id(208).0, 13);
        assert_eq!(AUTD3::grid_id(208).1, 11);
        assert_eq!(AUTD3::grid_id(209).0, 14);
        assert_eq!(AUTD3::grid_id(209).1, 11);
        assert_eq!(AUTD3::grid_id(210).0, 15);
        assert_eq!(AUTD3::grid_id(210).1, 11);
        assert_eq!(AUTD3::grid_id(211).0, 16);
        assert_eq!(AUTD3::grid_id(211).1, 11);
        assert_eq!(AUTD3::grid_id(212).0, 17);
        assert_eq!(AUTD3::grid_id(212).1, 11);
        assert_eq!(AUTD3::grid_id(213).0, 0);
        assert_eq!(AUTD3::grid_id(213).1, 12);
        assert_eq!(AUTD3::grid_id(214).0, 1);
        assert_eq!(AUTD3::grid_id(214).1, 12);
        assert_eq!(AUTD3::grid_id(215).0, 2);
        assert_eq!(AUTD3::grid_id(215).1, 12);
        assert_eq!(AUTD3::grid_id(216).0, 3);
        assert_eq!(AUTD3::grid_id(216).1, 12);
        assert_eq!(AUTD3::grid_id(217).0, 4);
        assert_eq!(AUTD3::grid_id(217).1, 12);
        assert_eq!(AUTD3::grid_id(218).0, 5);
        assert_eq!(AUTD3::grid_id(218).1, 12);
        assert_eq!(AUTD3::grid_id(219).0, 6);
        assert_eq!(AUTD3::grid_id(219).1, 12);
        assert_eq!(AUTD3::grid_id(220).0, 7);
        assert_eq!(AUTD3::grid_id(220).1, 12);
        assert_eq!(AUTD3::grid_id(221).0, 8);
        assert_eq!(AUTD3::grid_id(221).1, 12);
        assert_eq!(AUTD3::grid_id(222).0, 9);
        assert_eq!(AUTD3::grid_id(222).1, 12);
        assert_eq!(AUTD3::grid_id(223).0, 10);
        assert_eq!(AUTD3::grid_id(223).1, 12);
        assert_eq!(AUTD3::grid_id(224).0, 11);
        assert_eq!(AUTD3::grid_id(224).1, 12);
        assert_eq!(AUTD3::grid_id(225).0, 12);
        assert_eq!(AUTD3::grid_id(225).1, 12);
        assert_eq!(AUTD3::grid_id(226).0, 13);
        assert_eq!(AUTD3::grid_id(226).1, 12);
        assert_eq!(AUTD3::grid_id(227).0, 14);
        assert_eq!(AUTD3::grid_id(227).1, 12);
        assert_eq!(AUTD3::grid_id(228).0, 15);
        assert_eq!(AUTD3::grid_id(228).1, 12);
        assert_eq!(AUTD3::grid_id(229).0, 16);
        assert_eq!(AUTD3::grid_id(229).1, 12);
        assert_eq!(AUTD3::grid_id(230).0, 17);
        assert_eq!(AUTD3::grid_id(230).1, 12);
        assert_eq!(AUTD3::grid_id(231).0, 0);
        assert_eq!(AUTD3::grid_id(231).1, 13);
        assert_eq!(AUTD3::grid_id(232).0, 1);
        assert_eq!(AUTD3::grid_id(232).1, 13);
        assert_eq!(AUTD3::grid_id(233).0, 2);
        assert_eq!(AUTD3::grid_id(233).1, 13);
        assert_eq!(AUTD3::grid_id(234).0, 3);
        assert_eq!(AUTD3::grid_id(234).1, 13);
        assert_eq!(AUTD3::grid_id(235).0, 4);
        assert_eq!(AUTD3::grid_id(235).1, 13);
        assert_eq!(AUTD3::grid_id(236).0, 5);
        assert_eq!(AUTD3::grid_id(236).1, 13);
        assert_eq!(AUTD3::grid_id(237).0, 6);
        assert_eq!(AUTD3::grid_id(237).1, 13);
        assert_eq!(AUTD3::grid_id(238).0, 7);
        assert_eq!(AUTD3::grid_id(238).1, 13);
        assert_eq!(AUTD3::grid_id(239).0, 8);
        assert_eq!(AUTD3::grid_id(239).1, 13);
        assert_eq!(AUTD3::grid_id(240).0, 9);
        assert_eq!(AUTD3::grid_id(240).1, 13);
        assert_eq!(AUTD3::grid_id(241).0, 10);
        assert_eq!(AUTD3::grid_id(241).1, 13);
        assert_eq!(AUTD3::grid_id(242).0, 11);
        assert_eq!(AUTD3::grid_id(242).1, 13);
        assert_eq!(AUTD3::grid_id(243).0, 12);
        assert_eq!(AUTD3::grid_id(243).1, 13);
        assert_eq!(AUTD3::grid_id(244).0, 13);
        assert_eq!(AUTD3::grid_id(244).1, 13);
        assert_eq!(AUTD3::grid_id(245).0, 14);
        assert_eq!(AUTD3::grid_id(245).1, 13);
        assert_eq!(AUTD3::grid_id(246).0, 15);
        assert_eq!(AUTD3::grid_id(246).1, 13);
        assert_eq!(AUTD3::grid_id(247).0, 16);
        assert_eq!(AUTD3::grid_id(247).1, 13);
        assert_eq!(AUTD3::grid_id(248).0, 17);
        assert_eq!(AUTD3::grid_id(248).1, 13);
        assert_eq!(AUTD3::grid_id(249).0, 0);
        assert_eq!(AUTD3::grid_id(249).1, 0);
        assert_eq!(AUTD3::grid_id(250).0, 1);
        assert_eq!(AUTD3::grid_id(250).1, 0);
        assert_eq!(AUTD3::grid_id(251).0, 2);
        assert_eq!(AUTD3::grid_id(251).1, 0);
        assert_eq!(AUTD3::grid_id(252).0, 3);
        assert_eq!(AUTD3::grid_id(252).1, 0);
        assert_eq!(AUTD3::grid_id(253).0, 4);
        assert_eq!(AUTD3::grid_id(253).1, 0);
        assert_eq!(AUTD3::grid_id(254).0, 5);
        assert_eq!(AUTD3::grid_id(254).1, 0);
        assert_eq!(AUTD3::grid_id(255).0, 6);
        assert_eq!(AUTD3::grid_id(255).1, 0);
        assert_eq!(AUTD3::grid_id(256).0, 7);
        assert_eq!(AUTD3::grid_id(256).1, 0);
        assert_eq!(AUTD3::grid_id(257).0, 8);
        assert_eq!(AUTD3::grid_id(257).1, 0);
        assert_eq!(AUTD3::grid_id(258).0, 9);
        assert_eq!(AUTD3::grid_id(258).1, 0);
        assert_eq!(AUTD3::grid_id(259).0, 10);
        assert_eq!(AUTD3::grid_id(259).1, 0);
        assert_eq!(AUTD3::grid_id(260).0, 11);
        assert_eq!(AUTD3::grid_id(260).1, 0);
        assert_eq!(AUTD3::grid_id(261).0, 12);
        assert_eq!(AUTD3::grid_id(261).1, 0);
        assert_eq!(AUTD3::grid_id(262).0, 13);
        assert_eq!(AUTD3::grid_id(262).1, 0);
        assert_eq!(AUTD3::grid_id(263).0, 14);
        assert_eq!(AUTD3::grid_id(263).1, 0);
        assert_eq!(AUTD3::grid_id(264).0, 15);
        assert_eq!(AUTD3::grid_id(264).1, 0);
        assert_eq!(AUTD3::grid_id(265).0, 16);
        assert_eq!(AUTD3::grid_id(265).1, 0);
        assert_eq!(AUTD3::grid_id(266).0, 17);
        assert_eq!(AUTD3::grid_id(266).1, 0);
        assert_eq!(AUTD3::grid_id(267).0, 0);
        assert_eq!(AUTD3::grid_id(267).1, 1);
        assert_eq!(AUTD3::grid_id(268).0, 3);
        assert_eq!(AUTD3::grid_id(268).1, 1);
        assert_eq!(AUTD3::grid_id(269).0, 4);
        assert_eq!(AUTD3::grid_id(269).1, 1);
        assert_eq!(AUTD3::grid_id(270).0, 5);
        assert_eq!(AUTD3::grid_id(270).1, 1);
        assert_eq!(AUTD3::grid_id(271).0, 6);
        assert_eq!(AUTD3::grid_id(271).1, 1);
        assert_eq!(AUTD3::grid_id(272).0, 7);
        assert_eq!(AUTD3::grid_id(272).1, 1);
        assert_eq!(AUTD3::grid_id(273).0, 8);
        assert_eq!(AUTD3::grid_id(273).1, 1);
        assert_eq!(AUTD3::grid_id(274).0, 9);
        assert_eq!(AUTD3::grid_id(274).1, 1);
        assert_eq!(AUTD3::grid_id(275).0, 10);
        assert_eq!(AUTD3::grid_id(275).1, 1);
        assert_eq!(AUTD3::grid_id(276).0, 11);
        assert_eq!(AUTD3::grid_id(276).1, 1);
        assert_eq!(AUTD3::grid_id(277).0, 12);
        assert_eq!(AUTD3::grid_id(277).1, 1);
        assert_eq!(AUTD3::grid_id(278).0, 13);
        assert_eq!(AUTD3::grid_id(278).1, 1);
        assert_eq!(AUTD3::grid_id(279).0, 14);
        assert_eq!(AUTD3::grid_id(279).1, 1);
        assert_eq!(AUTD3::grid_id(280).0, 15);
        assert_eq!(AUTD3::grid_id(280).1, 1);
        assert_eq!(AUTD3::grid_id(281).0, 17);
        assert_eq!(AUTD3::grid_id(281).1, 1);
        assert_eq!(AUTD3::grid_id(282).0, 0);
        assert_eq!(AUTD3::grid_id(282).1, 2);
        assert_eq!(AUTD3::grid_id(283).0, 1);
        assert_eq!(AUTD3::grid_id(283).1, 2);
        assert_eq!(AUTD3::grid_id(284).0, 2);
        assert_eq!(AUTD3::grid_id(284).1, 2);
        assert_eq!(AUTD3::grid_id(285).0, 3);
        assert_eq!(AUTD3::grid_id(285).1, 2);
        assert_eq!(AUTD3::grid_id(286).0, 4);
        assert_eq!(AUTD3::grid_id(286).1, 2);
        assert_eq!(AUTD3::grid_id(287).0, 5);
        assert_eq!(AUTD3::grid_id(287).1, 2);
        assert_eq!(AUTD3::grid_id(288).0, 6);
        assert_eq!(AUTD3::grid_id(288).1, 2);
        assert_eq!(AUTD3::grid_id(289).0, 7);
        assert_eq!(AUTD3::grid_id(289).1, 2);
        assert_eq!(AUTD3::grid_id(290).0, 8);
        assert_eq!(AUTD3::grid_id(290).1, 2);
        assert_eq!(AUTD3::grid_id(291).0, 9);
        assert_eq!(AUTD3::grid_id(291).1, 2);
        assert_eq!(AUTD3::grid_id(292).0, 10);
        assert_eq!(AUTD3::grid_id(292).1, 2);
        assert_eq!(AUTD3::grid_id(293).0, 11);
        assert_eq!(AUTD3::grid_id(293).1, 2);
        assert_eq!(AUTD3::grid_id(294).0, 12);
        assert_eq!(AUTD3::grid_id(294).1, 2);
        assert_eq!(AUTD3::grid_id(295).0, 13);
        assert_eq!(AUTD3::grid_id(295).1, 2);
        assert_eq!(AUTD3::grid_id(296).0, 14);
        assert_eq!(AUTD3::grid_id(296).1, 2);
        assert_eq!(AUTD3::grid_id(297).0, 15);
        assert_eq!(AUTD3::grid_id(297).1, 2);
        assert_eq!(AUTD3::grid_id(298).0, 16);
        assert_eq!(AUTD3::grid_id(298).1, 2);
        assert_eq!(AUTD3::grid_id(299).0, 17);
        assert_eq!(AUTD3::grid_id(299).1, 2);
        assert_eq!(AUTD3::grid_id(300).0, 0);
        assert_eq!(AUTD3::grid_id(300).1, 3);
        assert_eq!(AUTD3::grid_id(301).0, 1);
        assert_eq!(AUTD3::grid_id(301).1, 3);
        assert_eq!(AUTD3::grid_id(302).0, 2);
        assert_eq!(AUTD3::grid_id(302).1, 3);
        assert_eq!(AUTD3::grid_id(303).0, 3);
        assert_eq!(AUTD3::grid_id(303).1, 3);
        assert_eq!(AUTD3::grid_id(304).0, 4);
        assert_eq!(AUTD3::grid_id(304).1, 3);
        assert_eq!(AUTD3::grid_id(305).0, 5);
        assert_eq!(AUTD3::grid_id(305).1, 3);
        assert_eq!(AUTD3::grid_id(306).0, 6);
        assert_eq!(AUTD3::grid_id(306).1, 3);
        assert_eq!(AUTD3::grid_id(307).0, 7);
        assert_eq!(AUTD3::grid_id(307).1, 3);
        assert_eq!(AUTD3::grid_id(308).0, 8);
        assert_eq!(AUTD3::grid_id(308).1, 3);
        assert_eq!(AUTD3::grid_id(309).0, 9);
        assert_eq!(AUTD3::grid_id(309).1, 3);
        assert_eq!(AUTD3::grid_id(310).0, 10);
        assert_eq!(AUTD3::grid_id(310).1, 3);
        assert_eq!(AUTD3::grid_id(311).0, 11);
        assert_eq!(AUTD3::grid_id(311).1, 3);
        assert_eq!(AUTD3::grid_id(312).0, 12);
        assert_eq!(AUTD3::grid_id(312).1, 3);
        assert_eq!(AUTD3::grid_id(313).0, 13);
        assert_eq!(AUTD3::grid_id(313).1, 3);
        assert_eq!(AUTD3::grid_id(314).0, 14);
        assert_eq!(AUTD3::grid_id(314).1, 3);
        assert_eq!(AUTD3::grid_id(315).0, 15);
        assert_eq!(AUTD3::grid_id(315).1, 3);
        assert_eq!(AUTD3::grid_id(316).0, 16);
        assert_eq!(AUTD3::grid_id(316).1, 3);
        assert_eq!(AUTD3::grid_id(317).0, 17);
        assert_eq!(AUTD3::grid_id(317).1, 3);
        assert_eq!(AUTD3::grid_id(318).0, 0);
        assert_eq!(AUTD3::grid_id(318).1, 4);
        assert_eq!(AUTD3::grid_id(319).0, 1);
        assert_eq!(AUTD3::grid_id(319).1, 4);
        assert_eq!(AUTD3::grid_id(320).0, 2);
        assert_eq!(AUTD3::grid_id(320).1, 4);
        assert_eq!(AUTD3::grid_id(321).0, 3);
        assert_eq!(AUTD3::grid_id(321).1, 4);
        assert_eq!(AUTD3::grid_id(322).0, 4);
        assert_eq!(AUTD3::grid_id(322).1, 4);
        assert_eq!(AUTD3::grid_id(323).0, 5);
        assert_eq!(AUTD3::grid_id(323).1, 4);
        assert_eq!(AUTD3::grid_id(324).0, 6);
        assert_eq!(AUTD3::grid_id(324).1, 4);
        assert_eq!(AUTD3::grid_id(325).0, 7);
        assert_eq!(AUTD3::grid_id(325).1, 4);
        assert_eq!(AUTD3::grid_id(326).0, 8);
        assert_eq!(AUTD3::grid_id(326).1, 4);
        assert_eq!(AUTD3::grid_id(327).0, 9);
        assert_eq!(AUTD3::grid_id(327).1, 4);
        assert_eq!(AUTD3::grid_id(328).0, 10);
        assert_eq!(AUTD3::grid_id(328).1, 4);
        assert_eq!(AUTD3::grid_id(329).0, 11);
        assert_eq!(AUTD3::grid_id(329).1, 4);
        assert_eq!(AUTD3::grid_id(330).0, 12);
        assert_eq!(AUTD3::grid_id(330).1, 4);
        assert_eq!(AUTD3::grid_id(331).0, 13);
        assert_eq!(AUTD3::grid_id(331).1, 4);
        assert_eq!(AUTD3::grid_id(332).0, 14);
        assert_eq!(AUTD3::grid_id(332).1, 4);
        assert_eq!(AUTD3::grid_id(333).0, 15);
        assert_eq!(AUTD3::grid_id(333).1, 4);
        assert_eq!(AUTD3::grid_id(334).0, 16);
        assert_eq!(AUTD3::grid_id(334).1, 4);
        assert_eq!(AUTD3::grid_id(335).0, 17);
        assert_eq!(AUTD3::grid_id(335).1, 4);
        assert_eq!(AUTD3::grid_id(336).0, 0);
        assert_eq!(AUTD3::grid_id(336).1, 5);
        assert_eq!(AUTD3::grid_id(337).0, 1);
        assert_eq!(AUTD3::grid_id(337).1, 5);
        assert_eq!(AUTD3::grid_id(338).0, 2);
        assert_eq!(AUTD3::grid_id(338).1, 5);
        assert_eq!(AUTD3::grid_id(339).0, 3);
        assert_eq!(AUTD3::grid_id(339).1, 5);
        assert_eq!(AUTD3::grid_id(340).0, 4);
        assert_eq!(AUTD3::grid_id(340).1, 5);
        assert_eq!(AUTD3::grid_id(341).0, 5);
        assert_eq!(AUTD3::grid_id(341).1, 5);
        assert_eq!(AUTD3::grid_id(342).0, 6);
        assert_eq!(AUTD3::grid_id(342).1, 5);
        assert_eq!(AUTD3::grid_id(343).0, 7);
        assert_eq!(AUTD3::grid_id(343).1, 5);
        assert_eq!(AUTD3::grid_id(344).0, 8);
        assert_eq!(AUTD3::grid_id(344).1, 5);
        assert_eq!(AUTD3::grid_id(345).0, 9);
        assert_eq!(AUTD3::grid_id(345).1, 5);
        assert_eq!(AUTD3::grid_id(346).0, 10);
        assert_eq!(AUTD3::grid_id(346).1, 5);
        assert_eq!(AUTD3::grid_id(347).0, 11);
        assert_eq!(AUTD3::grid_id(347).1, 5);
        assert_eq!(AUTD3::grid_id(348).0, 12);
        assert_eq!(AUTD3::grid_id(348).1, 5);
        assert_eq!(AUTD3::grid_id(349).0, 13);
        assert_eq!(AUTD3::grid_id(349).1, 5);
        assert_eq!(AUTD3::grid_id(350).0, 14);
        assert_eq!(AUTD3::grid_id(350).1, 5);
        assert_eq!(AUTD3::grid_id(351).0, 15);
        assert_eq!(AUTD3::grid_id(351).1, 5);
        assert_eq!(AUTD3::grid_id(352).0, 16);
        assert_eq!(AUTD3::grid_id(352).1, 5);
        assert_eq!(AUTD3::grid_id(353).0, 17);
        assert_eq!(AUTD3::grid_id(353).1, 5);
        assert_eq!(AUTD3::grid_id(354).0, 0);
        assert_eq!(AUTD3::grid_id(354).1, 6);
        assert_eq!(AUTD3::grid_id(355).0, 1);
        assert_eq!(AUTD3::grid_id(355).1, 6);
        assert_eq!(AUTD3::grid_id(356).0, 2);
        assert_eq!(AUTD3::grid_id(356).1, 6);
        assert_eq!(AUTD3::grid_id(357).0, 3);
        assert_eq!(AUTD3::grid_id(357).1, 6);
        assert_eq!(AUTD3::grid_id(358).0, 4);
        assert_eq!(AUTD3::grid_id(358).1, 6);
        assert_eq!(AUTD3::grid_id(359).0, 5);
        assert_eq!(AUTD3::grid_id(359).1, 6);
        assert_eq!(AUTD3::grid_id(360).0, 6);
        assert_eq!(AUTD3::grid_id(360).1, 6);
        assert_eq!(AUTD3::grid_id(361).0, 7);
        assert_eq!(AUTD3::grid_id(361).1, 6);
        assert_eq!(AUTD3::grid_id(362).0, 8);
        assert_eq!(AUTD3::grid_id(362).1, 6);
        assert_eq!(AUTD3::grid_id(363).0, 9);
        assert_eq!(AUTD3::grid_id(363).1, 6);
        assert_eq!(AUTD3::grid_id(364).0, 10);
        assert_eq!(AUTD3::grid_id(364).1, 6);
        assert_eq!(AUTD3::grid_id(365).0, 11);
        assert_eq!(AUTD3::grid_id(365).1, 6);
        assert_eq!(AUTD3::grid_id(366).0, 12);
        assert_eq!(AUTD3::grid_id(366).1, 6);
        assert_eq!(AUTD3::grid_id(367).0, 13);
        assert_eq!(AUTD3::grid_id(367).1, 6);
        assert_eq!(AUTD3::grid_id(368).0, 14);
        assert_eq!(AUTD3::grid_id(368).1, 6);
        assert_eq!(AUTD3::grid_id(369).0, 15);
        assert_eq!(AUTD3::grid_id(369).1, 6);
        assert_eq!(AUTD3::grid_id(370).0, 16);
        assert_eq!(AUTD3::grid_id(370).1, 6);
        assert_eq!(AUTD3::grid_id(371).0, 17);
        assert_eq!(AUTD3::grid_id(371).1, 6);
        assert_eq!(AUTD3::grid_id(372).0, 0);
        assert_eq!(AUTD3::grid_id(372).1, 7);
        assert_eq!(AUTD3::grid_id(373).0, 1);
        assert_eq!(AUTD3::grid_id(373).1, 7);
        assert_eq!(AUTD3::grid_id(374).0, 2);
        assert_eq!(AUTD3::grid_id(374).1, 7);
        assert_eq!(AUTD3::grid_id(375).0, 3);
        assert_eq!(AUTD3::grid_id(375).1, 7);
        assert_eq!(AUTD3::grid_id(376).0, 4);
        assert_eq!(AUTD3::grid_id(376).1, 7);
        assert_eq!(AUTD3::grid_id(377).0, 5);
        assert_eq!(AUTD3::grid_id(377).1, 7);
        assert_eq!(AUTD3::grid_id(378).0, 6);
        assert_eq!(AUTD3::grid_id(378).1, 7);
        assert_eq!(AUTD3::grid_id(379).0, 7);
        assert_eq!(AUTD3::grid_id(379).1, 7);
        assert_eq!(AUTD3::grid_id(380).0, 8);
        assert_eq!(AUTD3::grid_id(380).1, 7);
        assert_eq!(AUTD3::grid_id(381).0, 9);
        assert_eq!(AUTD3::grid_id(381).1, 7);
        assert_eq!(AUTD3::grid_id(382).0, 10);
        assert_eq!(AUTD3::grid_id(382).1, 7);
        assert_eq!(AUTD3::grid_id(383).0, 11);
        assert_eq!(AUTD3::grid_id(383).1, 7);
        assert_eq!(AUTD3::grid_id(384).0, 12);
        assert_eq!(AUTD3::grid_id(384).1, 7);
        assert_eq!(AUTD3::grid_id(385).0, 13);
        assert_eq!(AUTD3::grid_id(385).1, 7);
        assert_eq!(AUTD3::grid_id(386).0, 14);
        assert_eq!(AUTD3::grid_id(386).1, 7);
        assert_eq!(AUTD3::grid_id(387).0, 15);
        assert_eq!(AUTD3::grid_id(387).1, 7);
        assert_eq!(AUTD3::grid_id(388).0, 16);
        assert_eq!(AUTD3::grid_id(388).1, 7);
        assert_eq!(AUTD3::grid_id(389).0, 17);
        assert_eq!(AUTD3::grid_id(389).1, 7);
        assert_eq!(AUTD3::grid_id(390).0, 0);
        assert_eq!(AUTD3::grid_id(390).1, 8);
        assert_eq!(AUTD3::grid_id(391).0, 1);
        assert_eq!(AUTD3::grid_id(391).1, 8);
        assert_eq!(AUTD3::grid_id(392).0, 2);
        assert_eq!(AUTD3::grid_id(392).1, 8);
        assert_eq!(AUTD3::grid_id(393).0, 3);
        assert_eq!(AUTD3::grid_id(393).1, 8);
        assert_eq!(AUTD3::grid_id(394).0, 4);
        assert_eq!(AUTD3::grid_id(394).1, 8);
        assert_eq!(AUTD3::grid_id(395).0, 5);
        assert_eq!(AUTD3::grid_id(395).1, 8);
        assert_eq!(AUTD3::grid_id(396).0, 6);
        assert_eq!(AUTD3::grid_id(396).1, 8);
        assert_eq!(AUTD3::grid_id(397).0, 7);
        assert_eq!(AUTD3::grid_id(397).1, 8);
        assert_eq!(AUTD3::grid_id(398).0, 8);
        assert_eq!(AUTD3::grid_id(398).1, 8);
        assert_eq!(AUTD3::grid_id(399).0, 9);
        assert_eq!(AUTD3::grid_id(399).1, 8);
        assert_eq!(AUTD3::grid_id(400).0, 10);
        assert_eq!(AUTD3::grid_id(400).1, 8);
        assert_eq!(AUTD3::grid_id(401).0, 11);
        assert_eq!(AUTD3::grid_id(401).1, 8);
        assert_eq!(AUTD3::grid_id(402).0, 12);
        assert_eq!(AUTD3::grid_id(402).1, 8);
        assert_eq!(AUTD3::grid_id(403).0, 13);
        assert_eq!(AUTD3::grid_id(403).1, 8);
        assert_eq!(AUTD3::grid_id(404).0, 14);
        assert_eq!(AUTD3::grid_id(404).1, 8);
        assert_eq!(AUTD3::grid_id(405).0, 15);
        assert_eq!(AUTD3::grid_id(405).1, 8);
        assert_eq!(AUTD3::grid_id(406).0, 16);
        assert_eq!(AUTD3::grid_id(406).1, 8);
        assert_eq!(AUTD3::grid_id(407).0, 17);
        assert_eq!(AUTD3::grid_id(407).1, 8);
        assert_eq!(AUTD3::grid_id(408).0, 0);
        assert_eq!(AUTD3::grid_id(408).1, 9);
        assert_eq!(AUTD3::grid_id(409).0, 1);
        assert_eq!(AUTD3::grid_id(409).1, 9);
        assert_eq!(AUTD3::grid_id(410).0, 2);
        assert_eq!(AUTD3::grid_id(410).1, 9);
        assert_eq!(AUTD3::grid_id(411).0, 3);
        assert_eq!(AUTD3::grid_id(411).1, 9);
        assert_eq!(AUTD3::grid_id(412).0, 4);
        assert_eq!(AUTD3::grid_id(412).1, 9);
        assert_eq!(AUTD3::grid_id(413).0, 5);
        assert_eq!(AUTD3::grid_id(413).1, 9);
        assert_eq!(AUTD3::grid_id(414).0, 6);
        assert_eq!(AUTD3::grid_id(414).1, 9);
        assert_eq!(AUTD3::grid_id(415).0, 7);
        assert_eq!(AUTD3::grid_id(415).1, 9);
        assert_eq!(AUTD3::grid_id(416).0, 8);
        assert_eq!(AUTD3::grid_id(416).1, 9);
        assert_eq!(AUTD3::grid_id(417).0, 9);
        assert_eq!(AUTD3::grid_id(417).1, 9);
        assert_eq!(AUTD3::grid_id(418).0, 10);
        assert_eq!(AUTD3::grid_id(418).1, 9);
        assert_eq!(AUTD3::grid_id(419).0, 11);
        assert_eq!(AUTD3::grid_id(419).1, 9);
        assert_eq!(AUTD3::grid_id(420).0, 12);
        assert_eq!(AUTD3::grid_id(420).1, 9);
        assert_eq!(AUTD3::grid_id(421).0, 13);
        assert_eq!(AUTD3::grid_id(421).1, 9);
        assert_eq!(AUTD3::grid_id(422).0, 14);
        assert_eq!(AUTD3::grid_id(422).1, 9);
        assert_eq!(AUTD3::grid_id(423).0, 15);
        assert_eq!(AUTD3::grid_id(423).1, 9);
        assert_eq!(AUTD3::grid_id(424).0, 16);
        assert_eq!(AUTD3::grid_id(424).1, 9);
        assert_eq!(AUTD3::grid_id(425).0, 17);
        assert_eq!(AUTD3::grid_id(425).1, 9);
        assert_eq!(AUTD3::grid_id(426).0, 0);
        assert_eq!(AUTD3::grid_id(426).1, 10);
        assert_eq!(AUTD3::grid_id(427).0, 1);
        assert_eq!(AUTD3::grid_id(427).1, 10);
        assert_eq!(AUTD3::grid_id(428).0, 2);
        assert_eq!(AUTD3::grid_id(428).1, 10);
        assert_eq!(AUTD3::grid_id(429).0, 3);
        assert_eq!(AUTD3::grid_id(429).1, 10);
        assert_eq!(AUTD3::grid_id(430).0, 4);
        assert_eq!(AUTD3::grid_id(430).1, 10);
        assert_eq!(AUTD3::grid_id(431).0, 5);
        assert_eq!(AUTD3::grid_id(431).1, 10);
        assert_eq!(AUTD3::grid_id(432).0, 6);
        assert_eq!(AUTD3::grid_id(432).1, 10);
        assert_eq!(AUTD3::grid_id(433).0, 7);
        assert_eq!(AUTD3::grid_id(433).1, 10);
        assert_eq!(AUTD3::grid_id(434).0, 8);
        assert_eq!(AUTD3::grid_id(434).1, 10);
        assert_eq!(AUTD3::grid_id(435).0, 9);
        assert_eq!(AUTD3::grid_id(435).1, 10);
        assert_eq!(AUTD3::grid_id(436).0, 10);
        assert_eq!(AUTD3::grid_id(436).1, 10);
        assert_eq!(AUTD3::grid_id(437).0, 11);
        assert_eq!(AUTD3::grid_id(437).1, 10);
        assert_eq!(AUTD3::grid_id(438).0, 12);
        assert_eq!(AUTD3::grid_id(438).1, 10);
        assert_eq!(AUTD3::grid_id(439).0, 13);
        assert_eq!(AUTD3::grid_id(439).1, 10);
        assert_eq!(AUTD3::grid_id(440).0, 14);
        assert_eq!(AUTD3::grid_id(440).1, 10);
        assert_eq!(AUTD3::grid_id(441).0, 15);
        assert_eq!(AUTD3::grid_id(441).1, 10);
        assert_eq!(AUTD3::grid_id(442).0, 16);
        assert_eq!(AUTD3::grid_id(442).1, 10);
        assert_eq!(AUTD3::grid_id(443).0, 17);
        assert_eq!(AUTD3::grid_id(443).1, 10);
        assert_eq!(AUTD3::grid_id(444).0, 0);
        assert_eq!(AUTD3::grid_id(444).1, 11);
        assert_eq!(AUTD3::grid_id(445).0, 1);
        assert_eq!(AUTD3::grid_id(445).1, 11);
        assert_eq!(AUTD3::grid_id(446).0, 2);
        assert_eq!(AUTD3::grid_id(446).1, 11);
        assert_eq!(AUTD3::grid_id(447).0, 3);
        assert_eq!(AUTD3::grid_id(447).1, 11);
        assert_eq!(AUTD3::grid_id(448).0, 4);
        assert_eq!(AUTD3::grid_id(448).1, 11);
        assert_eq!(AUTD3::grid_id(449).0, 5);
        assert_eq!(AUTD3::grid_id(449).1, 11);
        assert_eq!(AUTD3::grid_id(450).0, 6);
        assert_eq!(AUTD3::grid_id(450).1, 11);
        assert_eq!(AUTD3::grid_id(451).0, 7);
        assert_eq!(AUTD3::grid_id(451).1, 11);
        assert_eq!(AUTD3::grid_id(452).0, 8);
        assert_eq!(AUTD3::grid_id(452).1, 11);
        assert_eq!(AUTD3::grid_id(453).0, 9);
        assert_eq!(AUTD3::grid_id(453).1, 11);
        assert_eq!(AUTD3::grid_id(454).0, 10);
        assert_eq!(AUTD3::grid_id(454).1, 11);
        assert_eq!(AUTD3::grid_id(455).0, 11);
        assert_eq!(AUTD3::grid_id(455).1, 11);
        assert_eq!(AUTD3::grid_id(456).0, 12);
        assert_eq!(AUTD3::grid_id(456).1, 11);
        assert_eq!(AUTD3::grid_id(457).0, 13);
        assert_eq!(AUTD3::grid_id(457).1, 11);
        assert_eq!(AUTD3::grid_id(458).0, 14);
        assert_eq!(AUTD3::grid_id(458).1, 11);
        assert_eq!(AUTD3::grid_id(459).0, 15);
        assert_eq!(AUTD3::grid_id(459).1, 11);
        assert_eq!(AUTD3::grid_id(460).0, 16);
        assert_eq!(AUTD3::grid_id(460).1, 11);
        assert_eq!(AUTD3::grid_id(461).0, 17);
        assert_eq!(AUTD3::grid_id(461).1, 11);
        assert_eq!(AUTD3::grid_id(462).0, 0);
        assert_eq!(AUTD3::grid_id(462).1, 12);
        assert_eq!(AUTD3::grid_id(463).0, 1);
        assert_eq!(AUTD3::grid_id(463).1, 12);
        assert_eq!(AUTD3::grid_id(464).0, 2);
        assert_eq!(AUTD3::grid_id(464).1, 12);
        assert_eq!(AUTD3::grid_id(465).0, 3);
        assert_eq!(AUTD3::grid_id(465).1, 12);
        assert_eq!(AUTD3::grid_id(466).0, 4);
        assert_eq!(AUTD3::grid_id(466).1, 12);
        assert_eq!(AUTD3::grid_id(467).0, 5);
        assert_eq!(AUTD3::grid_id(467).1, 12);
        assert_eq!(AUTD3::grid_id(468).0, 6);
        assert_eq!(AUTD3::grid_id(468).1, 12);
        assert_eq!(AUTD3::grid_id(469).0, 7);
        assert_eq!(AUTD3::grid_id(469).1, 12);
        assert_eq!(AUTD3::grid_id(470).0, 8);
        assert_eq!(AUTD3::grid_id(470).1, 12);
        assert_eq!(AUTD3::grid_id(471).0, 9);
        assert_eq!(AUTD3::grid_id(471).1, 12);
        assert_eq!(AUTD3::grid_id(472).0, 10);
        assert_eq!(AUTD3::grid_id(472).1, 12);
        assert_eq!(AUTD3::grid_id(473).0, 11);
        assert_eq!(AUTD3::grid_id(473).1, 12);
        assert_eq!(AUTD3::grid_id(474).0, 12);
        assert_eq!(AUTD3::grid_id(474).1, 12);
        assert_eq!(AUTD3::grid_id(475).0, 13);
        assert_eq!(AUTD3::grid_id(475).1, 12);
        assert_eq!(AUTD3::grid_id(476).0, 14);
        assert_eq!(AUTD3::grid_id(476).1, 12);
        assert_eq!(AUTD3::grid_id(477).0, 15);
        assert_eq!(AUTD3::grid_id(477).1, 12);
        assert_eq!(AUTD3::grid_id(478).0, 16);
        assert_eq!(AUTD3::grid_id(478).1, 12);
        assert_eq!(AUTD3::grid_id(479).0, 17);
        assert_eq!(AUTD3::grid_id(479).1, 12);
        assert_eq!(AUTD3::grid_id(480).0, 0);
        assert_eq!(AUTD3::grid_id(480).1, 13);
        assert_eq!(AUTD3::grid_id(481).0, 1);
        assert_eq!(AUTD3::grid_id(481).1, 13);
        assert_eq!(AUTD3::grid_id(482).0, 2);
        assert_eq!(AUTD3::grid_id(482).1, 13);
        assert_eq!(AUTD3::grid_id(483).0, 3);
        assert_eq!(AUTD3::grid_id(483).1, 13);
        assert_eq!(AUTD3::grid_id(484).0, 4);
        assert_eq!(AUTD3::grid_id(484).1, 13);
        assert_eq!(AUTD3::grid_id(485).0, 5);
        assert_eq!(AUTD3::grid_id(485).1, 13);
        assert_eq!(AUTD3::grid_id(486).0, 6);
        assert_eq!(AUTD3::grid_id(486).1, 13);
        assert_eq!(AUTD3::grid_id(487).0, 7);
        assert_eq!(AUTD3::grid_id(487).1, 13);
        assert_eq!(AUTD3::grid_id(488).0, 8);
        assert_eq!(AUTD3::grid_id(488).1, 13);
        assert_eq!(AUTD3::grid_id(489).0, 9);
        assert_eq!(AUTD3::grid_id(489).1, 13);
        assert_eq!(AUTD3::grid_id(490).0, 10);
        assert_eq!(AUTD3::grid_id(490).1, 13);
        assert_eq!(AUTD3::grid_id(491).0, 11);
        assert_eq!(AUTD3::grid_id(491).1, 13);
        assert_eq!(AUTD3::grid_id(492).0, 12);
        assert_eq!(AUTD3::grid_id(492).1, 13);
        assert_eq!(AUTD3::grid_id(493).0, 13);
        assert_eq!(AUTD3::grid_id(493).1, 13);
        assert_eq!(AUTD3::grid_id(494).0, 14);
        assert_eq!(AUTD3::grid_id(494).1, 13);
        assert_eq!(AUTD3::grid_id(495).0, 15);
        assert_eq!(AUTD3::grid_id(495).1, 13);
        assert_eq!(AUTD3::grid_id(496).0, 16);
        assert_eq!(AUTD3::grid_id(496).1, 13);
        assert_eq!(AUTD3::grid_id(497).0, 17);
        assert_eq!(AUTD3::grid_id(497).1, 13);
        assert_eq!(AUTD3::grid_id(498).0, 0);
        assert_eq!(AUTD3::grid_id(498).1, 0);
    }
}
