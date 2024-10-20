#[allow(non_camel_case_types)]
pub struct deg;
#[allow(non_camel_case_types)]
pub struct rad;

use derive_more::Debug;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Angle {
    #[debug("{} deg", _0)]
    Deg(f32),
    #[debug("{} rad", _0)]
    Rad(f32),
}

impl Angle {
    pub fn radian(self) -> f32 {
        match self {
            Self::Deg(a) => a.to_radians(),
            Self::Rad(a) => a,
        }
    }

    pub fn degree(self) -> f32 {
        match self {
            Self::Deg(a) => a,
            Self::Rad(a) => a.to_degrees(),
        }
    }
}

impl std::ops::Mul<deg> for f32 {
    type Output = Angle;

    fn mul(self, _rhs: deg) -> Self::Output {
        Self::Output::Deg(self)
    }
}

impl std::ops::Mul<rad> for f32 {
    type Output = Angle;

    fn mul(self, _rhs: rad) -> Self::Output {
        Self::Output::Rad(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dbg() {
        assert_eq!(format!("{:?}", 90.0 * deg), "90 deg");
        assert_eq!(format!("{:?}", 1.0 * rad), "1 rad");
    }
}
