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
