use getset::Getters;

use super::{Isometry, Point3};

use derive_new::new;

/// A ultrasound transducer.
#[derive(Clone, Debug, PartialEq, Getters, new)]
pub struct Transducer {
    idx: u8,
    dev_idx: u16,
    #[getset(get = "pub")]
    /// The position of the transducer.
    position: Point3,
}

impl Transducer {
    /// Gets the local index of the transducer.
    pub const fn idx(&self) -> usize {
        self.idx as _
    }

    /// Gets the index of the device to which this transducer belongs.
    pub const fn dev_idx(&self) -> usize {
        self.dev_idx as _
    }

    pub(super) fn affine(&mut self, isometry: &Isometry) {
        self.position = isometry * self.position;
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use crate::geometry::{Translation, UnitQuaternion, Vector3};

    use super::*;

    macro_rules! assert_vec3_approx_eq {
        ($a:expr, $b:expr) => {
            approx::assert_abs_diff_eq!($a.x, $b.x, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.y, $b.y, epsilon = 1e-3);
            approx::assert_abs_diff_eq!($a.z, $b.z, epsilon = 1e-3);
        };
    }

    #[test]
    fn idx() {
        let tr = Transducer::new(1, 2, Point3::origin());
        assert_eq!(1, tr.idx());
        assert_eq!(2, tr.dev_idx());
    }

    #[rstest::rstest]
    #[test]
    fn affine() {
        let mut tr = Transducer::new(0, 0, Point3::origin());

        let vector = Vector3::new(40., 50., 60.);
        let rotation = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.)
            * UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.);
        tr.affine(&Isometry {
            translation: Translation { vector },
            rotation,
        });

        let expect_pos = vector;
        assert_vec3_approx_eq!(expect_pos, tr.position());
    }
}
