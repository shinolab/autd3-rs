use crate::{
    derive::*,
    firmware::fpga::{EmitIntensity, Phase},
    geometry::Vector3,
};

use derive_more::{Deref, DerefMut};
use derive_new::new;

#[derive(Clone, Copy, Builder, PartialEq, Debug, new)]
#[repr(C)]
pub struct ControlPoint {
    #[get(ref)]
    #[set]
    point: Vector3,
    #[new(value = "Phase::ZERO")]
    #[get]
    #[set(into)]
    phase_offset: Phase,
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

#[derive(Clone, Builder, PartialEq, Debug, Deref, DerefMut, new)]
#[repr(C)]
pub struct ControlPoints<const N: usize> {
    #[deref]
    #[deref_mut]
    #[get]
    points: [ControlPoint; N],
    #[new(value = "EmitIntensity::MAX")]
    #[get]
    #[set(into)]
    intensity: EmitIntensity,
}

impl<C> From<C> for ControlPoints<1>
where
    ControlPoint: From<C>,
{
    fn from(point: C) -> Self {
        Self::new([point.into()])
    }
}

impl<C, I: Into<EmitIntensity>> From<(C, I)> for ControlPoints<1>
where
    ControlPoint: From<C>,
{
    fn from(point: (C, I)) -> Self {
        Self::new([point.0.into()]).with_intensity(point.1.into())
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

impl<C, I: Into<EmitIntensity>, const N: usize> From<([C; N], I)> for ControlPoints<N>
where
    ControlPoint: From<C>,
{
    fn from(points: ([C; N], I)) -> Self {
        Self::new(points.0.map(ControlPoint::from)).with_intensity(points.1.into())
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
    }

    #[test]
    fn from_vector3_ref() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let cp = ControlPoint::from(&v);
        assert_eq!(&v, cp.point());
    }

    #[test]
    fn from_control_point() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        let cp = ControlPoints::from([v1, v2]);
        assert_eq!(EmitIntensity::MAX, cp.intensity());
        assert_eq!(&v1, cp[0].point());
        assert_eq!(&v2, cp[1].point());
    }

    #[test]
    fn from_control_point_and_intensity() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        let cp = ControlPoints::from(([v1, v2], EmitIntensity::MIN));
        assert_eq!(EmitIntensity::MIN, cp.intensity());
        assert_eq!(&v1, cp[0].point());
        assert_eq!(&v2, cp[1].point());
    }
}
