#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    #[inline]
    #[must_use]
    pub fn norm(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// 3-dimensional column vector.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    #[must_use]
    pub const fn zeros() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[inline]
    #[must_use]
    pub fn from_iterator(mut iter: impl Iterator<Item = f32>) -> Self {
        Self {
            x: iter.next().unwrap_or(0.0),
            y: iter.next().unwrap_or(0.0),
            z: iter.next().unwrap_or(0.0),
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        [self.x, self.y, self.z].into_iter()
    }

    #[inline]
    #[must_use]
    pub const fn x() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[inline]
    #[must_use]
    pub const fn y() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    #[inline]
    #[must_use]
    pub const fn z() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }

    #[inline]
    #[must_use]
    pub const fn xy(&self) -> Vector2 {
        Vector2 {
            x: self.x,
            y: self.y,
        }
    }

    #[inline]
    #[must_use]
    pub const fn x_axis() -> UnitVector3 {
        UnitVector3 { vec: Self::x() }
    }

    #[inline]
    #[must_use]
    pub const fn y_axis() -> UnitVector3 {
        UnitVector3 { vec: Self::y() }
    }

    #[inline]
    #[must_use]
    pub const fn z_axis() -> UnitVector3 {
        UnitVector3 { vec: Self::z() }
    }

    #[inline]
    #[must_use]
    pub const fn dot(&self, rhs: &Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline]
    #[must_use]
    pub const fn cross(&self, rhs: &Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    #[inline]
    #[must_use]
    pub fn norm(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    #[inline]
    #[must_use]
    pub fn normalize(&self) -> Self {
        let n = self.norm();
        if n == 0.0 {
            Self::zeros()
        } else {
            Self {
                x: self.x / n,
                y: self.y / n,
                z: self.z / n,
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn try_normalize(&self, eps: f32) -> Option<Self> {
        let n = self.norm();
        if n < eps {
            None
        } else {
            Some(Self {
                x: self.x / n,
                y: self.y / n,
                z: self.z / n,
            })
        }
    }
}

impl core::ops::Neg for Vector3 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl core::ops::Add<&Vector3> for Vector3 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: &Vector3) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl core::ops::Add for Vector3 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl core::ops::Mul<f32> for Vector3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl core::ops::Div<f32> for Vector3 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl core::iter::Sum for Vector3 {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(
            Self {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            |a, b| Self {
                x: a.x + b.x,
                y: a.y + b.y,
                z: a.z + b.z,
            },
        )
    }
}

/// 3-dimensional unit vector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitVector3 {
    vec: Vector3,
}

impl UnitVector3 {
    #[inline]
    #[must_use]
    pub const fn new_unchecked(v: Vector3) -> Self {
        Self { vec: v }
    }

    #[inline]
    #[must_use]
    pub const fn into_inner(self) -> Vector3 {
        self.vec
    }

    #[inline]
    #[must_use]
    pub fn new_normalize(v: Vector3) -> Self {
        Self { vec: v.normalize() }
    }
}

impl core::ops::Deref for UnitVector3 {
    type Target = Vector3;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector2_norm() {
        let v = Vector2 { x: 3.0, y: 4.0 };
        assert_eq!(v.norm(), 5.0);
    }

    #[test]
    fn vector3_new() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn vector3_zeros() {
        let v = Vector3::zeros();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
        assert_eq!(v.z, 0.0);
    }

    #[test]
    fn vector3_axes() {
        let x = Vector3::x();
        assert_eq!(x.x, 1.0);
        assert_eq!(x.y, 0.0);
        assert_eq!(x.z, 0.0);

        let y = Vector3::y();
        assert_eq!(y.x, 0.0);
        assert_eq!(y.y, 1.0);
        assert_eq!(y.z, 0.0);

        let z = Vector3::z();
        assert_eq!(z.x, 0.0);
        assert_eq!(z.y, 0.0);
        assert_eq!(z.z, 1.0);
    }

    #[test]
    fn vector3_xy() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let xy = v.xy();
        assert_eq!(xy.x, 1.0);
        assert_eq!(xy.y, 2.0);
    }

    #[test]
    fn vector3_dot() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        let result = v1.dot(&v2);
        assert_eq!(result, 32.0);
    }

    #[test]
    fn vector3_cross() {
        let v1 = Vector3::new(1.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 1.0, 0.0);
        let result = v1.cross(&v2);
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 1.0);
    }

    #[test]
    fn vector3_norm() {
        let v = Vector3::new(3.0, 4.0, 0.0);
        assert_eq!(v.norm(), 5.0);
    }

    #[test]
    fn vector3_normalize() {
        let v = Vector3::new(3.0, 4.0, 0.0);
        let normalized = v.normalize();
        assert_eq!(normalized.x, 0.6);
        assert_eq!(normalized.y, 0.8);
        assert_eq!(normalized.z, 0.0);
        assert!((normalized.norm() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn vector3_normalize_zero() {
        let v = Vector3::zeros();
        let normalized = v.normalize();
        assert_eq!(normalized, Vector3::zeros());
    }

    #[test]
    fn vector3_try_normalize() {
        let v = Vector3::new(3.0, 4.0, 0.0);
        let normalized = v.try_normalize(1e-6).unwrap();
        assert!((normalized.norm() - 1.0).abs() < 1e-6);

        let small = Vector3::new(1e-7, 0.0, 0.0);
        assert!(small.try_normalize(1e-6).is_none());
    }

    #[test]
    fn vector3_neg() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let neg = -v;
        assert_eq!(neg.x, -1.0);
        assert_eq!(neg.y, -2.0);
        assert_eq!(neg.z, -3.0);
    }

    #[test]
    fn vector3_add() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        let result = v1 + v2;
        assert_eq!(result.x, 5.0);
        assert_eq!(result.y, 7.0);
        assert_eq!(result.z, 9.0);
    }

    #[test]
    fn vector3_mul_scalar() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let result = v * 2.0;
        assert_eq!(result.x, 2.0);
        assert_eq!(result.y, 4.0);
        assert_eq!(result.z, 6.0);
    }

    #[test]
    fn vector3_div_scalar() {
        let v = Vector3::new(2.0, 4.0, 6.0);
        let result = v / 2.0;
        assert_eq!(result.x, 1.0);
        assert_eq!(result.y, 2.0);
        assert_eq!(result.z, 3.0);
    }

    #[test]
    fn vector3_sum() {
        let vectors = vec![
            Vector3::new(1.0, 2.0, 3.0),
            Vector3::new(4.0, 5.0, 6.0),
            Vector3::new(7.0, 8.0, 9.0),
        ];
        let sum: Vector3 = vectors.into_iter().sum();
        assert_eq!(sum.x, 12.0);
        assert_eq!(sum.y, 15.0);
        assert_eq!(sum.z, 18.0);
    }

    #[test]
    fn unit_vector3_new_normalize() {
        let v = Vector3::new(3.0, 4.0, 0.0);
        let unit = UnitVector3::new_normalize(v);
        assert!((unit.norm() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn unit_vector3_axes() {
        let x = Vector3::x_axis();
        assert_eq!(x.x, 1.0);
        assert_eq!(x.y, 0.0);
        assert_eq!(x.z, 0.0);

        let y = Vector3::y_axis();
        assert_eq!(y.x, 0.0);
        assert_eq!(y.y, 1.0);
        assert_eq!(y.z, 0.0);

        let z = Vector3::z_axis();
        assert_eq!(z.x, 0.0);
        assert_eq!(z.y, 0.0);
        assert_eq!(z.z, 1.0);
    }
}
