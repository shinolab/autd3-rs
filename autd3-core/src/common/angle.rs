/// \[°\]
#[allow(non_camel_case_types)]
pub struct deg;

/// \[rad\]
#[allow(non_camel_case_types)]
pub struct rad;

use derive_more::Debug;

/// Angle
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
#[debug("{}rad", radian)]
pub struct Angle {
    radian: f32,
}

impl Angle {
    /// An angle of zero
    pub const ZERO: Self = Self { radian: 0.0 };

    /// An angle of π
    pub const PI: Self = Self {
        radian: std::f32::consts::PI,
    };

    /// Returns the angle in radian
    #[must_use]
    pub const fn radian(self) -> f32 {
        self.radian
    }

    /// Returns the angle in degree
    #[must_use]
    pub const fn degree(self) -> f32 {
        self.radian.to_degrees()
    }
}

impl std::ops::Mul<deg> for f32 {
    type Output = Angle;

    fn mul(self, _rhs: deg) -> Self::Output {
        Self::Output {
            radian: self.to_radians(),
        }
    }
}

impl std::ops::Mul<rad> for f32 {
    type Output = Angle;

    fn mul(self, _rhs: rad) -> Self::Output {
        Self::Output { radian: self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dbg() {
        assert_eq!(format!("{:?}", 1.0 * rad), "1rad");
    }
}
