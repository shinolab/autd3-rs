use crate::{
    common::mm,
    geometry::{Device, Isometry, Point3, Transducer, Translation, UnitQuaternion},
};
use getset::Getters;
use std::fmt::Debug;

/// AUTD3 device.
#[derive(Clone, Copy, Debug, Getters)]
pub struct AUTD3<R: Into<UnitQuaternion> + Debug> {
    /// The global position of the AUTD3 device.
    pub pos: Point3,
    /// The rotation of the AUTD3 device.
    pub rot: R,
}

impl Default for AUTD3<UnitQuaternion> {
    fn default() -> Self {
        Self {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        }
    }
}

impl AUTD3<UnitQuaternion> {
    /// The number of transducers in x-axis.
    pub const NUM_TRANS_X: usize = 18;
    /// The number of transducers in y-axis.
    pub const NUM_TRANS_Y: usize = 14;
    /// The number of transducers in a unit.
    pub const NUM_TRANS_IN_UNIT: usize = Self::NUM_TRANS_X * Self::NUM_TRANS_Y - 3;
    /// The spacing between transducers.
    pub const TRANS_SPACING: f32 = 10.16 * mm;
    /// The width of the device (including the substrate).
    pub const DEVICE_WIDTH: f32 = 192.0 * mm;
    /// The height of the device (including the substrate).
    pub const DEVICE_HEIGHT: f32 = 151.4 * mm;

    #[must_use]
    const fn is_missing_transducer(x: usize, y: usize) -> bool {
        if Self::NUM_TRANS_X <= x || Self::NUM_TRANS_Y <= y {
            return true;
        }
        y == 1 && (x == 1 || x == 2 || x == 16)
    }

    /// Gets the index in x- and y-axis from the transducer index.
    #[must_use]
    pub const fn grid_id(idx: usize) -> (usize, usize) {
        let local_id = idx % Self::NUM_TRANS_IN_UNIT;
        let uid = match local_id {
            0..19 => local_id,
            19..32 => local_id + 2,
            _ => local_id + 3,
        };
        (uid % Self::NUM_TRANS_X, uid / Self::NUM_TRANS_X)
    }
}

impl<R: Into<UnitQuaternion> + Debug> AUTD3<R> {
    /// Create a new AUTD3 device.
    #[must_use]
    pub fn new(pos: Point3, rot: R) -> Self {
        Self { pos, rot }
    }
}

impl<R: Into<UnitQuaternion> + Debug> From<AUTD3<R>> for Device {
    fn from(autd3: AUTD3<R>) -> Self {
        let rot = autd3.rot.into();
        let isometry = Isometry {
            rotation: rot,
            translation: Translation::from(autd3.pos),
        };
        Self::new(
            rot,
            itertools::iproduct!(0..AUTD3::NUM_TRANS_Y, 0..AUTD3::NUM_TRANS_X)
                .filter(|&(y, x)| !AUTD3::is_missing_transducer(x, y))
                .map(|(y, x)| {
                    isometry
                        * Point3::new(
                            x as f32 * AUTD3::TRANS_SPACING,
                            y as f32 * AUTD3::TRANS_SPACING,
                            0.,
                        )
                })
                .map(|p| Transducer::new(p.xyz()))
                .collect(),
        )
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use autd3_core::geometry::{Geometry, Vector3};

    use super::*;

    pub fn create_device() -> Device {
        AUTD3::default().into()
    }

    pub fn create_geometry(n: usize) -> crate::geometry::Geometry {
        Geometry::new((0..n).map(|_| create_device()).collect())
    }

    #[test]
    fn num_devices() {
        let dev: Device = AUTD3::default().into();
        assert_eq!(AUTD3::NUM_TRANS_IN_UNIT, dev.num_transducers());
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Point3::new(0., 0., 0.),
        0,
        Point3::origin(),
        UnitQuaternion::identity()
    )]
    #[case(
        Point3::new(AUTD3::TRANS_SPACING, 0., 0.),
        1,
        Point3::origin(),
        UnitQuaternion::identity()
    )]
    #[case(
        Point3::new(0., AUTD3::TRANS_SPACING, 0.),
        18,
        Point3::origin(),
        UnitQuaternion::identity()
    )]
    #[case(Point3::new(17. * AUTD3::TRANS_SPACING, 13. * AUTD3::TRANS_SPACING, 0.), 248, Point3::origin(), UnitQuaternion::identity())]
    #[case(
        Point3::new(1., 2., 3.),
        0,
        Point3::new(1., 2., 3.),
        UnitQuaternion::identity()
    )]
    #[case(
        Point3::new(AUTD3::TRANS_SPACING + 1., 2., 3.),
        1,
        Point3::new(1., 2., 3.),
        UnitQuaternion::identity()
    )]
    #[case(
        Point3::new(1., AUTD3::TRANS_SPACING + 2., 3.),
        18,
        Point3::new(1., 2., 3.),
        UnitQuaternion::identity()
    )]
    #[case(Point3::new(17. * AUTD3::TRANS_SPACING + 1., 13. * AUTD3::TRANS_SPACING + 2., 3.), 248, Point3::new(1., 2., 3.), UnitQuaternion::identity())]
    #[case(
        Point3::new(0., 0., 0.),
        0,
        Point3::origin(),
        UnitQuaternion::new(Vector3::y() * std::f32::consts::FRAC_PI_2)
    )]
    #[case(
        Point3::new(0., 0., -AUTD3::TRANS_SPACING),
        1,
        Point3::origin(),
        UnitQuaternion::new(Vector3::y() * std::f32::consts::FRAC_PI_2)
    )]
    #[case(
        Point3::new(0., AUTD3::TRANS_SPACING, 0.),
        18,
        Point3::origin(),
        UnitQuaternion::new(Vector3::y() * std::f32::consts::FRAC_PI_2)
    )]
    #[case(Point3::new(0., 13. * AUTD3::TRANS_SPACING, -17. * AUTD3::TRANS_SPACING), 248, Point3::origin(), UnitQuaternion::new(Vector3::y() * std::f32::consts::FRAC_PI_2))]
    #[case(
        Point3::new(1., 2., 3.),
        0,
        Point3::new(1., 2., 3.),
        UnitQuaternion::new(Vector3::y() * std::f32::consts::FRAC_PI_2)
    )]
    #[case(
        Point3::new(1., 2., 3. - AUTD3::TRANS_SPACING),
        1,
        Point3::new(1., 2., 3.),
        UnitQuaternion::new(Vector3::y() * std::f32::consts::FRAC_PI_2)
    )]
    #[case(
        Point3::new(1., 2. + AUTD3::TRANS_SPACING, 3.),
        18,
        Point3::new(1., 2., 3.),
        UnitQuaternion::new(Vector3::y() * std::f32::consts::FRAC_PI_2)
    )]
    #[case(Point3::new(1., 2. + 13. * AUTD3::TRANS_SPACING, 3. - 17. * AUTD3::TRANS_SPACING), 248, Point3::new(1., 2., 3.), UnitQuaternion::new(Vector3::y() * std::f32::consts::FRAC_PI_2))]
    fn position(
        #[case] expected: Point3,
        #[case] idx: usize,
        #[case] pos: Point3,
        #[case] rot: impl Into<UnitQuaternion> + Debug,
    ) {
        let dev: Device = AUTD3 { pos, rot }.into();
        approx::assert_relative_eq!(expected.x, dev[idx].position().x, epsilon = 1e-6);
        approx::assert_relative_eq!(expected.y, dev[idx].position().y, epsilon = 1e-6);
        approx::assert_relative_eq!(expected.z, dev[idx].position().z, epsilon = 1e-6);
    }

    #[rstest::rstest]
    #[test]
    #[case(0, 0, false)]
    #[case(1, 0, false)]
    #[case(2, 0, false)]
    #[case(3, 0, false)]
    #[case(4, 0, false)]
    #[case(5, 0, false)]
    #[case(6, 0, false)]
    #[case(7, 0, false)]
    #[case(8, 0, false)]
    #[case(9, 0, false)]
    #[case(10, 0, false)]
    #[case(11, 0, false)]
    #[case(12, 0, false)]
    #[case(13, 0, false)]
    #[case(14, 0, false)]
    #[case(15, 0, false)]
    #[case(16, 0, false)]
    #[case(17, 0, false)]
    #[case(0, 1, false)]
    #[case(1, 1, true)]
    #[case(2, 1, true)]
    #[case(3, 1, false)]
    #[case(4, 1, false)]
    #[case(5, 1, false)]
    #[case(6, 1, false)]
    #[case(7, 1, false)]
    #[case(8, 1, false)]
    #[case(9, 1, false)]
    #[case(10, 1, false)]
    #[case(11, 1, false)]
    #[case(12, 1, false)]
    #[case(13, 1, false)]
    #[case(14, 1, false)]
    #[case(15, 1, false)]
    #[case(16, 1, true)]
    #[case(17, 1, false)]
    #[case(0, 2, false)]
    #[case(1, 2, false)]
    #[case(2, 2, false)]
    #[case(3, 2, false)]
    #[case(4, 2, false)]
    #[case(5, 2, false)]
    #[case(6, 2, false)]
    #[case(7, 2, false)]
    #[case(8, 2, false)]
    #[case(9, 2, false)]
    #[case(10, 2, false)]
    #[case(11, 2, false)]
    #[case(12, 2, false)]
    #[case(13, 2, false)]
    #[case(14, 2, false)]
    #[case(15, 2, false)]
    #[case(16, 2, false)]
    #[case(17, 2, false)]
    #[case(0, 3, false)]
    #[case(1, 3, false)]
    #[case(2, 3, false)]
    #[case(3, 3, false)]
    #[case(4, 3, false)]
    #[case(5, 3, false)]
    #[case(6, 3, false)]
    #[case(7, 3, false)]
    #[case(8, 3, false)]
    #[case(9, 3, false)]
    #[case(10, 3, false)]
    #[case(11, 3, false)]
    #[case(12, 3, false)]
    #[case(13, 3, false)]
    #[case(14, 3, false)]
    #[case(15, 3, false)]
    #[case(16, 3, false)]
    #[case(17, 3, false)]
    #[case(0, 4, false)]
    #[case(1, 4, false)]
    #[case(2, 4, false)]
    #[case(3, 4, false)]
    #[case(4, 4, false)]
    #[case(5, 4, false)]
    #[case(6, 4, false)]
    #[case(7, 4, false)]
    #[case(8, 4, false)]
    #[case(9, 4, false)]
    #[case(10, 4, false)]
    #[case(11, 4, false)]
    #[case(12, 4, false)]
    #[case(13, 4, false)]
    #[case(14, 4, false)]
    #[case(15, 4, false)]
    #[case(16, 4, false)]
    #[case(17, 4, false)]
    #[case(0, 5, false)]
    #[case(1, 5, false)]
    #[case(2, 5, false)]
    #[case(3, 5, false)]
    #[case(4, 5, false)]
    #[case(5, 5, false)]
    #[case(6, 5, false)]
    #[case(7, 5, false)]
    #[case(8, 5, false)]
    #[case(9, 5, false)]
    #[case(10, 5, false)]
    #[case(11, 5, false)]
    #[case(12, 5, false)]
    #[case(13, 5, false)]
    #[case(14, 5, false)]
    #[case(15, 5, false)]
    #[case(16, 5, false)]
    #[case(17, 5, false)]
    #[case(0, 6, false)]
    #[case(1, 6, false)]
    #[case(2, 6, false)]
    #[case(3, 6, false)]
    #[case(4, 6, false)]
    #[case(5, 6, false)]
    #[case(6, 6, false)]
    #[case(7, 6, false)]
    #[case(8, 6, false)]
    #[case(9, 6, false)]
    #[case(10, 6, false)]
    #[case(11, 6, false)]
    #[case(12, 6, false)]
    #[case(13, 6, false)]
    #[case(14, 6, false)]
    #[case(15, 6, false)]
    #[case(16, 6, false)]
    #[case(17, 6, false)]
    #[case(0, 7, false)]
    #[case(1, 7, false)]
    #[case(2, 7, false)]
    #[case(3, 7, false)]
    #[case(4, 7, false)]
    #[case(5, 7, false)]
    #[case(6, 7, false)]
    #[case(7, 7, false)]
    #[case(8, 7, false)]
    #[case(9, 7, false)]
    #[case(10, 7, false)]
    #[case(11, 7, false)]
    #[case(12, 7, false)]
    #[case(13, 7, false)]
    #[case(14, 7, false)]
    #[case(15, 7, false)]
    #[case(16, 7, false)]
    #[case(17, 7, false)]
    #[case(0, 8, false)]
    #[case(1, 8, false)]
    #[case(2, 8, false)]
    #[case(3, 8, false)]
    #[case(4, 8, false)]
    #[case(5, 8, false)]
    #[case(6, 8, false)]
    #[case(7, 8, false)]
    #[case(8, 8, false)]
    #[case(9, 8, false)]
    #[case(10, 8, false)]
    #[case(11, 8, false)]
    #[case(12, 8, false)]
    #[case(13, 8, false)]
    #[case(14, 8, false)]
    #[case(15, 8, false)]
    #[case(16, 8, false)]
    #[case(17, 8, false)]
    #[case(0, 9, false)]
    #[case(1, 9, false)]
    #[case(2, 9, false)]
    #[case(3, 9, false)]
    #[case(4, 9, false)]
    #[case(5, 9, false)]
    #[case(6, 9, false)]
    #[case(7, 9, false)]
    #[case(8, 9, false)]
    #[case(9, 9, false)]
    #[case(10, 9, false)]
    #[case(11, 9, false)]
    #[case(12, 9, false)]
    #[case(13, 9, false)]
    #[case(14, 9, false)]
    #[case(15, 9, false)]
    #[case(16, 9, false)]
    #[case(17, 9, false)]
    #[case(0, 10, false)]
    #[case(1, 10, false)]
    #[case(2, 10, false)]
    #[case(3, 10, false)]
    #[case(4, 10, false)]
    #[case(5, 10, false)]
    #[case(6, 10, false)]
    #[case(7, 10, false)]
    #[case(8, 10, false)]
    #[case(9, 10, false)]
    #[case(10, 10, false)]
    #[case(11, 10, false)]
    #[case(12, 10, false)]
    #[case(13, 10, false)]
    #[case(14, 10, false)]
    #[case(15, 10, false)]
    #[case(16, 10, false)]
    #[case(17, 10, false)]
    #[case(0, 11, false)]
    #[case(1, 11, false)]
    #[case(2, 11, false)]
    #[case(3, 11, false)]
    #[case(4, 11, false)]
    #[case(5, 11, false)]
    #[case(6, 11, false)]
    #[case(7, 11, false)]
    #[case(8, 11, false)]
    #[case(9, 11, false)]
    #[case(10, 11, false)]
    #[case(11, 11, false)]
    #[case(12, 11, false)]
    #[case(13, 11, false)]
    #[case(14, 11, false)]
    #[case(15, 11, false)]
    #[case(16, 11, false)]
    #[case(17, 11, false)]
    #[case(0, 12, false)]
    #[case(1, 12, false)]
    #[case(2, 12, false)]
    #[case(3, 12, false)]
    #[case(4, 12, false)]
    #[case(5, 12, false)]
    #[case(6, 12, false)]
    #[case(7, 12, false)]
    #[case(8, 12, false)]
    #[case(9, 12, false)]
    #[case(10, 12, false)]
    #[case(11, 12, false)]
    #[case(12, 12, false)]
    #[case(13, 12, false)]
    #[case(14, 12, false)]
    #[case(15, 12, false)]
    #[case(16, 12, false)]
    #[case(17, 12, false)]
    #[case(0, 13, false)]
    #[case(1, 13, false)]
    #[case(2, 13, false)]
    #[case(3, 13, false)]
    #[case(4, 13, false)]
    #[case(5, 13, false)]
    #[case(6, 13, false)]
    #[case(7, 13, false)]
    #[case(8, 13, false)]
    #[case(9, 13, false)]
    #[case(10, 13, false)]
    #[case(11, 13, false)]
    #[case(12, 13, false)]
    #[case(13, 13, false)]
    #[case(14, 13, false)]
    #[case(15, 13, false)]
    #[case(16, 13, false)]
    #[case(17, 13, false)]
    fn test_is_missing_transducer(#[case] x: usize, #[case] y: usize, #[case] expected: bool) {
        assert_eq!(expected, AUTD3::is_missing_transducer(x, y));
    }

    #[test]
    fn test_is_missing_transducer_out_of_range() {
        itertools::iproduct!(18..=256, 0..=256).for_each(|(x, y)| {
            assert!(AUTD3::is_missing_transducer(x, y));
        });
        itertools::iproduct!(0..=256, 14..=256).for_each(|(x, y)| {
            assert!(AUTD3::is_missing_transducer(x, y));
        });
    }

    #[rstest::rstest]
    #[test]
    #[case(0, (0, 0))]
    #[case(1, (1, 0))]
    #[case(2, (2, 0))]
    #[case(3, (3, 0))]
    #[case(4, (4, 0))]
    #[case(5, (5, 0))]
    #[case(6, (6, 0))]
    #[case(7, (7, 0))]
    #[case(8, (8, 0))]
    #[case(9, (9, 0))]
    #[case(10, (10, 0))]
    #[case(11, (11, 0))]
    #[case(12, (12, 0))]
    #[case(13, (13, 0))]
    #[case(14, (14, 0))]
    #[case(15, (15, 0))]
    #[case(16, (16, 0))]
    #[case(17, (17, 0))]
    #[case(18, (0, 1))]
    #[case(19, (3, 1))]
    #[case(20, (4, 1))]
    #[case(21, (5, 1))]
    #[case(22, (6, 1))]
    #[case(23, (7, 1))]
    #[case(24, (8, 1))]
    #[case(25, (9, 1))]
    #[case(26, (10, 1))]
    #[case(27, (11, 1))]
    #[case(28, (12, 1))]
    #[case(29, (13, 1))]
    #[case(30, (14, 1))]
    #[case(31, (15, 1))]
    #[case(32, (17, 1))]
    #[case(33, (0, 2))]
    #[case(34, (1, 2))]
    #[case(35, (2, 2))]
    #[case(36, (3, 2))]
    #[case(37, (4, 2))]
    #[case(38, (5, 2))]
    #[case(39, (6, 2))]
    #[case(40, (7, 2))]
    #[case(41, (8, 2))]
    #[case(42, (9, 2))]
    #[case(43, (10, 2))]
    #[case(44, (11, 2))]
    #[case(45, (12, 2))]
    #[case(46, (13, 2))]
    #[case(47, (14, 2))]
    #[case(48, (15, 2))]
    #[case(49, (16, 2))]
    #[case(50, (17, 2))]
    #[case(51, (0, 3))]
    #[case(52, (1, 3))]
    #[case(53, (2, 3))]
    #[case(54, (3, 3))]
    #[case(55, (4, 3))]
    #[case(56, (5, 3))]
    #[case(57, (6, 3))]
    #[case(58, (7, 3))]
    #[case(59, (8, 3))]
    #[case(60, (9, 3))]
    #[case(61, (10, 3))]
    #[case(62, (11, 3))]
    #[case(63, (12, 3))]
    #[case(64, (13, 3))]
    #[case(65, (14, 3))]
    #[case(66, (15, 3))]
    #[case(67, (16, 3))]
    #[case(68, (17, 3))]
    #[case(69, (0, 4))]
    #[case(70, (1, 4))]
    #[case(71, (2, 4))]
    #[case(72, (3, 4))]
    #[case(73, (4, 4))]
    #[case(74, (5, 4))]
    #[case(75, (6, 4))]
    #[case(76, (7, 4))]
    #[case(77, (8, 4))]
    #[case(78, (9, 4))]
    #[case(79, (10, 4))]
    #[case(80, (11, 4))]
    #[case(81, (12, 4))]
    #[case(82, (13, 4))]
    #[case(83, (14, 4))]
    #[case(84, (15, 4))]
    #[case(85, (16, 4))]
    #[case(86, (17, 4))]
    #[case(87, (0, 5))]
    #[case(88, (1, 5))]
    #[case(89, (2, 5))]
    #[case(90, (3, 5))]
    #[case(91, (4, 5))]
    #[case(92, (5, 5))]
    #[case(93, (6, 5))]
    #[case(94, (7, 5))]
    #[case(95, (8, 5))]
    #[case(96, (9, 5))]
    #[case(97, (10, 5))]
    #[case(98, (11, 5))]
    #[case(99, (12, 5))]
    #[case(100, (13, 5))]
    #[case(101, (14, 5))]
    #[case(102, (15, 5))]
    #[case(103, (16, 5))]
    #[case(104, (17, 5))]
    #[case(105, (0, 6))]
    #[case(106, (1, 6))]
    #[case(107, (2, 6))]
    #[case(108, (3, 6))]
    #[case(109, (4, 6))]
    #[case(110, (5, 6))]
    #[case(111, (6, 6))]
    #[case(112, (7, 6))]
    #[case(113, (8, 6))]
    #[case(114, (9, 6))]
    #[case(115, (10, 6))]
    #[case(116, (11, 6))]
    #[case(117, (12, 6))]
    #[case(118, (13, 6))]
    #[case(119, (14, 6))]
    #[case(120, (15, 6))]
    #[case(121, (16, 6))]
    #[case(122, (17, 6))]
    #[case(123, (0, 7))]
    #[case(124, (1, 7))]
    #[case(125, (2, 7))]
    #[case(126, (3, 7))]
    #[case(127, (4, 7))]
    #[case(128, (5, 7))]
    #[case(129, (6, 7))]
    #[case(130, (7, 7))]
    #[case(131, (8, 7))]
    #[case(132, (9, 7))]
    #[case(133, (10, 7))]
    #[case(134, (11, 7))]
    #[case(135, (12, 7))]
    #[case(136, (13, 7))]
    #[case(137, (14, 7))]
    #[case(138, (15, 7))]
    #[case(139, (16, 7))]
    #[case(140, (17, 7))]
    #[case(141, (0, 8))]
    #[case(142, (1, 8))]
    #[case(143, (2, 8))]
    #[case(144, (3, 8))]
    #[case(145, (4, 8))]
    #[case(146, (5, 8))]
    #[case(147, (6, 8))]
    #[case(148, (7, 8))]
    #[case(149, (8, 8))]
    #[case(150, (9, 8))]
    #[case(151, (10, 8))]
    #[case(152, (11, 8))]
    #[case(153, (12, 8))]
    #[case(154, (13, 8))]
    #[case(155, (14, 8))]
    #[case(156, (15, 8))]
    #[case(157, (16, 8))]
    #[case(158, (17, 8))]
    #[case(159, (0, 9))]
    #[case(160, (1, 9))]
    #[case(161, (2, 9))]
    #[case(162, (3, 9))]
    #[case(163, (4, 9))]
    #[case(164, (5, 9))]
    #[case(165, (6, 9))]
    #[case(166, (7, 9))]
    #[case(167, (8, 9))]
    #[case(168, (9, 9))]
    #[case(169, (10, 9))]
    #[case(170, (11, 9))]
    #[case(171, (12, 9))]
    #[case(172, (13, 9))]
    #[case(173, (14, 9))]
    #[case(174, (15, 9))]
    #[case(175, (16, 9))]
    #[case(176, (17, 9))]
    #[case(177, (0, 10))]
    #[case(178, (1, 10))]
    #[case(179, (2, 10))]
    #[case(180, (3, 10))]
    #[case(181, (4, 10))]
    #[case(182, (5, 10))]
    #[case(183, (6, 10))]
    #[case(184, (7, 10))]
    #[case(185, (8, 10))]
    #[case(186, (9, 10))]
    #[case(187, (10, 10))]
    #[case(188, (11, 10))]
    #[case(189, (12, 10))]
    #[case(190, (13, 10))]
    #[case(191, (14, 10))]
    #[case(192, (15, 10))]
    #[case(193, (16, 10))]
    #[case(194, (17, 10))]
    #[case(195, (0, 11))]
    #[case(196, (1, 11))]
    #[case(197, (2, 11))]
    #[case(198, (3, 11))]
    #[case(199, (4, 11))]
    #[case(200, (5, 11))]
    #[case(201, (6, 11))]
    #[case(202, (7, 11))]
    #[case(203, (8, 11))]
    #[case(204, (9, 11))]
    #[case(205, (10, 11))]
    #[case(206, (11, 11))]
    #[case(207, (12, 11))]
    #[case(208, (13, 11))]
    #[case(209, (14, 11))]
    #[case(210, (15, 11))]
    #[case(211, (16, 11))]
    #[case(212, (17, 11))]
    #[case(213, (0, 12))]
    #[case(214, (1, 12))]
    #[case(215, (2, 12))]
    #[case(216, (3, 12))]
    #[case(217, (4, 12))]
    #[case(218, (5, 12))]
    #[case(219, (6, 12))]
    #[case(220, (7, 12))]
    #[case(221, (8, 12))]
    #[case(222, (9, 12))]
    #[case(223, (10, 12))]
    #[case(224, (11, 12))]
    #[case(225, (12, 12))]
    #[case(226, (13, 12))]
    #[case(227, (14, 12))]
    #[case(228, (15, 12))]
    #[case(229, (16, 12))]
    #[case(230, (17, 12))]
    #[case(231, (0, 13))]
    #[case(232, (1, 13))]
    #[case(233, (2, 13))]
    #[case(234, (3, 13))]
    #[case(235, (4, 13))]
    #[case(236, (5, 13))]
    #[case(237, (6, 13))]
    #[case(238, (7, 13))]
    #[case(239, (8, 13))]
    #[case(240, (9, 13))]
    #[case(241, (10, 13))]
    #[case(242, (11, 13))]
    #[case(243, (12, 13))]
    #[case(244, (13, 13))]
    #[case(245, (14, 13))]
    #[case(246, (15, 13))]
    #[case(247, (16, 13))]
    #[case(248, (17, 13))]
    fn test_grid_id(#[case] idx: usize, #[case] expected: (usize, usize)) {
        assert_eq!(expected, AUTD3::grid_id(idx));
    }
}
