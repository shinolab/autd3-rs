#[allow(non_camel_case_types)]
pub struct deg;
#[allow(non_camel_case_types)]
pub struct rad;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Angle {
    Deg(f32),
    Rad(f32),
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
