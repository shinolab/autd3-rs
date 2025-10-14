use super::{Point3, UnitVector3, Vector3};

/// A quaternion.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
    pub w: f32,
    pub i: f32,
    pub j: f32,
    pub k: f32,
}

impl Quaternion {
    #[inline]
    #[must_use]
    pub const fn new(w: f32, i: f32, j: f32, k: f32) -> Self {
        Self { w, i, j, k }
    }

    #[inline]
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
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitQuaternion {
    pub w: f32,
    pub i: f32,
    pub j: f32,
    pub k: f32,
}

impl UnitQuaternion {
    #[inline]
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

    #[inline]
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            w: 1.0,
            i: 0.0,
            j: 0.0,
            k: 0.0,
        }
    }

    #[inline]
    #[must_use]
    pub const fn quaternion(&self) -> Quaternion {
        Quaternion {
            w: self.w,
            i: self.i,
            j: self.j,
            k: self.k,
        }
    }

    #[inline]
    #[must_use]
    pub fn from_quaternion(quat: Quaternion) -> Self {
        Self::new_normalize(quat)
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    #[must_use]
    pub const fn conjugate(self) -> Self {
        Self {
            w: self.w,
            i: -self.i,
            j: -self.j,
            k: -self.k,
        }
    }

    #[inline]
    #[must_use]
    pub const fn imag(self) -> Vector3 {
        Vector3 {
            x: self.i,
            y: self.j,
            z: self.k,
        }
    }

    #[inline]
    #[must_use]
    pub fn transform_point(&self, p: &Point3) -> Point3 {
        Point3 {
            coords: self.transform_vector(&p.coords),
        }
    }

    #[inline]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quaternion_new() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q.w, 1.0);
        assert_eq!(q.i, 2.0);
        assert_eq!(q.j, 3.0);
        assert_eq!(q.k, 4.0);
    }

    #[test]
    fn quaternion_from_imag() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let q = Quaternion::from_imag(v);
        assert_eq!(q.w, 0.0);
        assert_eq!(q.i, 1.0);
        assert_eq!(q.j, 2.0);
        assert_eq!(q.k, 3.0);
    }

    #[test]
    fn unit_quaternion_identity() {
        let q = UnitQuaternion::identity();
        assert_eq!(q.w, 1.0);
        assert_eq!(q.i, 0.0);
        assert_eq!(q.j, 0.0);
        assert_eq!(q.k, 0.0);
    }

    #[test]
    fn unit_quaternion_new_zero() {
        let q = UnitQuaternion::new(Vector3::zeros());
        assert_eq!(q, UnitQuaternion::identity());
    }

    #[test]
    fn unit_quaternion_from_axis_angle() {
        let axis = UnitVector3::new_normalize(Vector3::new(0.0, 0.0, 1.0));
        let angle = core::f32::consts::PI / 2.0;
        let q = UnitQuaternion::from_axis_angle(&axis, angle);

        assert!((q.w - (angle / 2.0).cos()).abs() < 1e-6);
        assert!((q.k - (angle / 2.0).sin()).abs() < 1e-6);
    }

    #[test]
    fn unit_quaternion_new_normalize() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let uq = UnitQuaternion::new_normalize(q);

        let norm = (uq.w * uq.w + uq.i * uq.i + uq.j * uq.j + uq.k * uq.k).sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn unit_quaternion_conjugate() {
        let q = UnitQuaternion::new_normalize(Quaternion::new(1.0, 2.0, 3.0, 4.0));
        let conj = q.conjugate();

        assert_eq!(conj.w, q.w);
        assert_eq!(conj.i, -q.i);
        assert_eq!(conj.j, -q.j);
        assert_eq!(conj.k, -q.k);
    }

    #[test]
    fn unit_quaternion_imag() {
        let q = UnitQuaternion::new_normalize(Quaternion::new(1.0, 2.0, 3.0, 4.0));
        let imag = q.imag();

        assert_eq!(imag.x, q.i);
        assert_eq!(imag.y, q.j);
        assert_eq!(imag.z, q.k);
    }

    #[test]
    fn unit_quaternion_mul() {
        let q1 = UnitQuaternion::identity();
        let q2 = UnitQuaternion::identity();
        let result = q1 * q2;

        assert_eq!(result, UnitQuaternion::identity());
    }

    #[test]
    fn unit_quaternion_transform_point() {
        let axis = UnitVector3::new_normalize(Vector3::new(0.0, 0.0, 1.0));
        let angle = core::f32::consts::PI / 2.0;
        let q = UnitQuaternion::from_axis_angle(&axis, angle);

        let p = Point3::new(1.0, 0.0, 0.0);
        let transformed = q.transform_point(&p);

        assert!(transformed.x.abs() < 1e-6);
        assert!((transformed.y - 1.0).abs() < 1e-6);
        assert!(transformed.z.abs() < 1e-6);
    }

    #[test]
    fn unit_quaternion_transform_vector() {
        let axis = UnitVector3::new_normalize(Vector3::new(0.0, 0.0, 1.0));
        let angle = core::f32::consts::PI / 2.0;
        let q = UnitQuaternion::from_axis_angle(&axis, angle);

        let v = Vector3::new(1.0, 0.0, 0.0);
        let transformed = q.transform_vector(&v);

        assert!(transformed.x.abs() < 1e-6);
        assert!((transformed.y - 1.0).abs() < 1e-6);
        assert!(transformed.z.abs() < 1e-6);
    }

    #[test]
    fn unit_quaternion_rotation_composition() {
        let axis = UnitVector3::new_normalize(Vector3::new(0.0, 0.0, 1.0));
        let angle = core::f32::consts::PI / 4.0;
        let q1 = UnitQuaternion::from_axis_angle(&axis, angle);
        let q2 = UnitQuaternion::from_axis_angle(&axis, angle);

        let q_combined = q1 * q2;
        let v = Vector3::new(1.0, 0.0, 0.0);

        let result1 = q_combined.transform_vector(&v);
        let result2 = q2.transform_vector(&(q1.transform_vector(&v)));

        assert!((result1.x - result2.x).abs() < 1e-6);
        assert!((result1.y - result2.y).abs() < 1e-6);
        assert!((result1.z - result2.z).abs() < 1e-6);
    }
}
