use crate::{derive::*, geometry::Vector3};

use derive_more::{Deref, DerefMut};

#[derive(Clone, Copy, Builder, PartialEq, Debug)]
pub struct ControlPoint {
    #[getset]
    point: Vector3,
    #[getset]
    intensity: EmitIntensity,
    #[getset]
    offset: Phase,
}

impl ControlPoint {
    pub const fn new(point: Vector3) -> Self {
        Self {
            point,
            intensity: EmitIntensity::MAX,
            offset: Phase::new(0),
        }
    }
}

impl From<Vector3> for ControlPoint {
    fn from(point: Vector3) -> Self {
        Self::new(point)
    }
}

impl From<&Vector3> for ControlPoint {
    fn from(point: &Vector3) -> Self {
        Self::new(*point)
    }
}

#[derive(Clone, Builder, PartialEq, Debug, Deref, DerefMut)]
pub struct ControlPoints<const N: usize> {
    #[deref]
    #[deref_mut]
    #[get]
    points: [ControlPoint; N],
}

impl<const N: usize> ControlPoints<N> {
    pub const fn new(points: [ControlPoint; N]) -> Self {
        Self { points }
    }
}

impl<C> From<C> for ControlPoints<1>
where
    ControlPoint: From<C>,
{
    fn from(point: C) -> Self {
        Self::new([point.into()])
    }
}

impl<C, const N: usize> From<[C; N]> for ControlPoints<N>
where
    ControlPoint: From<C>,
{
    fn from(points: [C; N]) -> Self {
        Self::new(points.map(ControlPoint::from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_vector3() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let cp = ControlPoint::from(v);
        assert_eq!(&v, cp.point());
        assert_eq!(EmitIntensity::MAX, cp.intensity());
    }

    #[test]
    fn from_vector3_ref() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let cp = ControlPoint::from(&v);
        assert_eq!(&v, cp.point());
        assert_eq!(EmitIntensity::MAX, cp.intensity());
    }
}
