use super::{UnitQuaternion, Vector3};

/// 3-dimensional point.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Point3 {
    pub coords: Vector3,
}

impl Point3 {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            coords: Vector3::new(x, y, z),
        }
    }

    #[must_use]
    pub const fn origin() -> Self {
        Self {
            coords: Vector3::zeros(),
        }
    }
}

impl From<Vector3> for Point3 {
    fn from(coords: Vector3) -> Self {
        Self { coords }
    }
}

impl core::ops::Deref for Point3 {
    type Target = Vector3;

    fn deref(&self) -> &Self::Target {
        &self.coords
    }
}

impl core::ops::Add<Vector3> for Point3 {
    type Output = Self;

    fn add(self, rhs: Vector3) -> Self::Output {
        Self {
            coords: Vector3 {
                x: self.coords.x + rhs.x,
                y: self.coords.y + rhs.y,
                z: self.coords.z + rhs.z,
            },
        }
    }
}

impl core::ops::Sub for Point3 {
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3 {
            x: self.coords.x - rhs.coords.x,
            y: self.coords.y - rhs.coords.y,
            z: self.coords.z - rhs.coords.z,
        }
    }
}

impl core::ops::Sub<Vector3> for Point3 {
    type Output = Self;

    fn sub(self, rhs: Vector3) -> Self::Output {
        Self {
            coords: Vector3 {
                x: self.coords.x - rhs.x,
                y: self.coords.y - rhs.y,
                z: self.coords.z - rhs.z,
            },
        }
    }
}

impl core::ops::Mul<f32> for Point3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            coords: self.coords * rhs,
        }
    }
}

impl core::ops::Mul<Point3> for f32 {
    type Output = Point3;

    fn mul(self, rhs: Point3) -> Self::Output {
        rhs * self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Translation3 {
    pub vector: Vector3,
}

impl From<Point3> for Translation3 {
    fn from(p: Point3) -> Self {
        Self { vector: p.coords }
    }
}

impl core::ops::Mul<UnitQuaternion> for Translation3 {
    type Output = Isometry3;

    fn mul(self, rhs: UnitQuaternion) -> Self::Output {
        Isometry3 {
            translation: self,
            rotation: rhs,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Isometry3 {
    pub translation: Translation3,
    pub rotation: UnitQuaternion,
}

impl Isometry3 {
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            translation: Translation3 {
                vector: Vector3::zeros(),
            },
            rotation: UnitQuaternion::identity(),
        }
    }

    #[must_use]
    pub fn inverse(&self) -> Self {
        let inv_rot = self.rotation.conjugate();
        let inv_tr = -self.translation.vector;
        let inv_tr = inv_rot.transform_vector(&inv_tr);
        Self {
            translation: Translation3 { vector: inv_tr },
            rotation: inv_rot,
        }
    }
}

impl core::ops::Mul<&Point3> for &Isometry3 {
    type Output = Point3;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: &Point3) -> Self::Output {
        self.rotation.transform_point(rhs) + self.translation.vector
    }
}

impl core::ops::Mul<Point3> for &Isometry3 {
    type Output = Point3;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Point3) -> Self::Output {
        self.rotation.transform_point(&rhs) + self.translation.vector
    }
}

impl core::ops::Mul<Point3> for Isometry3 {
    type Output = Point3;

    fn mul(self, rhs: Point3) -> Self::Output {
        &self * &rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::UnitVector3;

    #[test]
    fn point3_new() {
        let p = Point3::new(1.0, 2.0, 3.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
        assert_eq!(p.z, 3.0);
    }

    #[test]
    fn point3_origin() {
        let p = Point3::origin();
        assert_eq!(p.x, 0.0);
        assert_eq!(p.y, 0.0);
        assert_eq!(p.z, 0.0);
    }

    #[test]
    fn point3_from_vector() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let p = Point3::from(v);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
        assert_eq!(p.z, 3.0);
    }

    #[test]
    fn point3_add_vector() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let v = Vector3::new(4.0, 5.0, 6.0);
        let result = p + v;
        assert_eq!(result.x, 1.0 + 4.0);
        assert_eq!(result.y, 2.0 + 5.0);
        assert_eq!(result.z, 3.0 + 6.0);
    }

    #[test]
    fn point3_sub_point() {
        let p1 = Point3::new(5.0, 7.0, 9.0);
        let p2 = Point3::new(1.0, 2.0, 3.0);
        let result = p1 - p2;
        assert_eq!(result.x, 5.0 - 1.0);
        assert_eq!(result.y, 7.0 - 2.0);
        assert_eq!(result.z, 9.0 - 3.0);
    }

    #[test]
    fn point3_sub_vector() {
        let p = Point3::new(5.0, 7.0, 9.0);
        let v = Vector3::new(1.0, 2.0, 3.0);
        let result = p - v;
        assert_eq!(result.x, 5.0 - 1.0);
        assert_eq!(result.y, 7.0 - 2.0);
        assert_eq!(result.z, 9.0 - 3.0);
    }

    #[test]
    fn point3_mul_scalar() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let result = p * 2.0;
        assert_eq!(result.x, 1.0 * 2.0);
        assert_eq!(result.y, 2.0 * 2.0);
        assert_eq!(result.z, 3.0 * 2.0);
    }

    #[test]
    fn point3_scalar_mul() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let result = 2.0 * p;
        assert_eq!(result.x, 1.0 * 2.0);
        assert_eq!(result.y, 2.0 * 2.0);
        assert_eq!(result.z, 3.0 * 2.0);
    }

    #[test]
    fn translation3_from_point() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let t = Translation3::from(p);
        assert_eq!(t.vector.x, 1.0);
        assert_eq!(t.vector.y, 2.0);
        assert_eq!(t.vector.z, 3.0);
    }

    #[test]
    fn translation3_mul_quaternion() {
        let t = Translation3 {
            vector: Vector3::new(1.0, 2.0, 3.0),
        };
        let q = UnitQuaternion::identity();
        let iso = t * q;

        assert_eq!(iso.translation.vector, t.vector);
        assert_eq!(iso.rotation, q);
    }

    #[test]
    fn isometry3_identity() {
        let iso = Isometry3::identity();
        assert_eq!(iso.translation.vector, Vector3::zeros());
        assert_eq!(iso.rotation, UnitQuaternion::identity());
    }

    #[test]
    fn isometry3_inverse() {
        let axis = UnitVector3::new_normalize(Vector3::new(0.0, 0.0, 1.0));
        let angle = core::f32::consts::PI / 2.0;
        let rotation = UnitQuaternion::from_axis_angle(&axis, angle);
        let translation = Translation3 {
            vector: Vector3::new(1.0, 0.0, 0.0),
        };

        let iso = Isometry3 {
            translation,
            rotation,
        };

        let inv = iso.inverse();
        let p = Point3::new(2.0, 3.0, 4.0);
        let transformed = &iso * &p;
        let back = &inv * &transformed;

        assert!((back.x - p.x).abs() < 1e-5);
        assert!((back.y - p.y).abs() < 1e-5);
        assert!((back.z - p.z).abs() < 1e-5);
    }

    #[test]
    fn isometry3_transform_point() {
        let translation = Translation3 {
            vector: Vector3::new(1.0, 2.0, 3.0),
        };
        let rotation = UnitQuaternion::identity();
        let iso = Isometry3 {
            translation,
            rotation,
        };

        let p = Point3::new(1.0, 1.0, 1.0);
        let result = &iso * &p;

        assert_eq!(result.x, 2.0);
        assert_eq!(result.y, 3.0);
        assert_eq!(result.z, 4.0);
    }

    #[test]
    fn isometry3_transform_with_rotation() {
        let axis = UnitVector3::new_normalize(Vector3::new(0.0, 0.0, 1.0));
        let angle = core::f32::consts::PI / 2.0;
        let rotation = UnitQuaternion::from_axis_angle(&axis, angle);
        let translation = Translation3 {
            vector: Vector3::new(0.0, 0.0, 0.0),
        };

        let iso = Isometry3 {
            translation,
            rotation,
        };

        let p = Point3::new(1.0, 0.0, 0.0);
        let result = iso * p;

        assert!(result.x.abs() < 1e-6);
        assert!((result.y - 1.0).abs() < 1e-6);
        assert!(result.z.abs() < 1e-6);
    }
}
