#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
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
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn zeros() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[must_use]
    pub const fn x() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[must_use]
    pub const fn y() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    #[must_use]
    pub const fn z() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }

    #[must_use]
    pub const fn xy(&self) -> Vector2 {
        Vector2 {
            x: self.x,
            y: self.y,
        }
    }

    #[must_use]
    pub const fn x_axis() -> UnitVector3 {
        UnitVector3 { vec: Self::x() }
    }

    #[must_use]
    pub const fn y_axis() -> UnitVector3 {
        UnitVector3 { vec: Self::y() }
    }

    #[must_use]
    pub const fn z_axis() -> UnitVector3 {
        UnitVector3 { vec: Self::z() }
    }

    #[must_use]
    pub fn dot(&self, rhs: &Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[must_use]
    pub fn cross(&self, rhs: &Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    #[must_use]
    pub fn norm(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

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

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl core::iter::Sum for Vector3 {
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
    #[must_use]
    pub const fn new_unchecked(v: Vector3) -> Self {
        Self { vec: v }
    }

    #[must_use]
    pub const fn into_inner(self) -> Vector3 {
        self.vec
    }

    #[must_use]
    pub fn new_normalize(v: Vector3) -> Self {
        Self { vec: v.normalize() }
    }
}

impl core::ops::Deref for UnitVector3 {
    type Target = Vector3;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}
