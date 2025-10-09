use super::{Point3, UnitVector3, Vector3};

/// A quaternion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
    pub w: f32,
    pub i: f32,
    pub j: f32,
    pub k: f32,
}

impl Quaternion {
    #[must_use]
    pub const fn new(w: f32, i: f32, j: f32, k: f32) -> Self {
        Self { w, i, j, k }
    }

    #[must_use]
    pub const fn from_imag(v: Vector3) -> Self {
        Self {
            w: 0.0,
            i: v.x,
            j: v.y,
            k: v.z,
        }
    }
}

/// A unit quaternion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitQuaternion {
    pub w: f32,
    pub i: f32,
    pub j: f32,
    pub k: f32,
}

impl UnitQuaternion {
    #[must_use]
    pub fn new(asisangle: Vector3) -> Self {
        let angle = asisangle.norm();
        if angle == 0.0 {
            Self::identity()
        } else {
            let axis = UnitVector3::new_unchecked(asisangle / angle);
            Self::from_axis_angle(&axis, angle)
        }
    }

    #[must_use]
    pub const fn identity() -> Self {
        Self {
            w: 1.0,
            i: 0.0,
            j: 0.0,
            k: 0.0,
        }
    }

    #[must_use]
    pub const fn quaternion(&self) -> Quaternion {
        Quaternion {
            w: self.w,
            i: self.i,
            j: self.j,
            k: self.k,
        }
    }

    #[must_use]
    pub fn from_quaternion(quat: Quaternion) -> Self {
        Self::new_normalize(quat)
    }

    #[must_use]
    pub fn from_axis_angle(axis: &UnitVector3, angle: f32) -> Self {
        let (s, c) = (angle / 2.0).sin_cos();
        let axis = axis.into_inner();
        Self {
            w: c,
            i: axis.x * s,
            j: axis.y * s,
            k: axis.z * s,
        }
    }

    #[must_use]
    pub fn new_normalize(quat: Quaternion) -> Self {
        let norm = (quat.w * quat.w + quat.i * quat.i + quat.j * quat.j + quat.k * quat.k).sqrt();
        Self {
            w: quat.w / norm,
            i: quat.i / norm,
            j: quat.j / norm,
            k: quat.k / norm,
        }
    }

    #[must_use]
    pub const fn conjugate(self) -> Self {
        Self {
            w: self.w,
            i: -self.i,
            j: -self.j,
            k: -self.k,
        }
    }

    #[must_use]
    pub const fn imag(self) -> Vector3 {
        Vector3 {
            x: self.i,
            y: self.j,
            z: self.k,
        }
    }

    #[must_use]
    pub fn transform_point(&self, p: &Point3) -> Point3 {
        Point3 {
            coords: self.transform_vector(&p.coords),
        }
    }

    #[must_use]
    pub fn transform_vector(&self, v: &Vector3) -> Vector3 {
        self * v
    }
}

impl core::ops::Mul for UnitQuaternion {
    type Output = Self;

    #[allow(clippy::op_ref)]
    fn mul(self, rhs: Self) -> Self::Output {
        &self * rhs
    }
}

impl core::ops::Mul<UnitQuaternion> for &UnitQuaternion {
    type Output = UnitQuaternion;

    fn mul(self, rhs: UnitQuaternion) -> Self::Output {
        let w = self.w * rhs.w - self.i * rhs.i - self.j * rhs.j - self.k * rhs.k;
        let i = self.w * rhs.i + self.i * rhs.w + self.j * rhs.k - self.k * rhs.j;
        let j = self.w * rhs.j - self.i * rhs.k + self.j * rhs.w + self.k * rhs.i;
        let k = self.w * rhs.k + self.i * rhs.j - self.j * rhs.i + self.k * rhs.w;
        UnitQuaternion { w, i, j, k }
    }
}

impl core::ops::Mul<&Vector3> for &UnitQuaternion {
    type Output = Vector3;

    fn mul(self, rhs: &Vector3) -> Self::Output {
        let t = self.imag().cross(rhs) * 2.0;
        let cross = self.imag().cross(&t);
        t * self.w + cross + rhs
    }
}

impl core::ops::Mul<Vector3> for UnitQuaternion {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Self::Output {
        &self * &rhs
    }
}
