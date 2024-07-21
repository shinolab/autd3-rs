#[allow(non_camel_case_types)]
pub struct deg;
#[allow(non_camel_case_types)]
pub struct rad;

#[derive(Clone, Copy, PartialEq)]
pub enum Angle {
    Deg(f32),
    Rad(f32),
}

impl std::fmt::Debug for Angle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deg(arg0) => write!(f, "{} deg", arg0),
            Self::Rad(arg0) => write!(f, "{} rad", arg0),
        }
    }
}

impl Angle {
    pub fn radian(self) -> f32 {
        match self {
            Self::Deg(a) => a.to_radians(),
            Self::Rad(a) => a,
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
    #[cfg_attr(miri, ignore)]
    fn dbg() {
        assert_eq!(format!("{:?}", 90.0 * deg), "90 deg");
        assert_eq!(format!("{:?}", 1.0 * rad), "1 rad");
    }
}
