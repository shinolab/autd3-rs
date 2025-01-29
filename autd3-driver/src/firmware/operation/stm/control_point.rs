use crate::{
    firmware::fpga::{EmitIntensity, Phase},
    geometry::{Isometry, Point3},
};

use derive_more::{Deref, DerefMut};
use derive_new::new;

/// A pair of a focal point and a phase offset.
#[derive(Clone, Copy, PartialEq, Debug, Default, new)]
#[repr(C)]
pub struct ControlPoint {
    /// The focal point.
    pub point: Point3,
    /// The phase offset of the control point.
    pub phase_offset: Phase,
}

impl ControlPoint {
    pub(crate) fn transform(&self, iso: &Isometry) -> Self {
        Self {
            point: iso.transform_point(&self.point),
            phase_offset: self.phase_offset,
        }
    }
}

impl From<Point3> for ControlPoint {
    fn from(point: Point3) -> Self {
        Self {
            point,
            ..Default::default()
        }
    }
}

impl From<&Point3> for ControlPoint {
    fn from(point: &Point3) -> Self {
        Self {
            point: *point,
            ..Default::default()
        }
    }
}

/// A collection of control points and the intensity of all control points.
#[derive(Clone, PartialEq, Debug, Deref, DerefMut, new)]
#[repr(C)]
pub struct ControlPoints<const N: usize> {
    #[deref]
    #[deref_mut]
    /// The control points.
    pub points: [ControlPoint; N],
    /// The intensity of all control points.
    pub intensity: EmitIntensity,
}

impl<const N: usize> Default for ControlPoints<N> {
    fn default() -> Self {
        Self {
            points: [Default::default(); N],
            intensity: EmitIntensity::MAX,
        }
    }
}

impl<const N: usize> ControlPoints<N> {
    pub(crate) fn transform(&self, iso: &nalgebra::Isometry3<f32>) -> Self {
        Self {
            points: self.points.map(|p| p.transform(iso)),
            intensity: self.intensity,
        }
    }
}

impl<C> From<C> for ControlPoints<1>
where
    ControlPoint: From<C>,
{
    fn from(point: C) -> Self {
        Self {
            points: [point.into()],
            ..Default::default()
        }
    }
}

impl<C, const N: usize> From<[C; N]> for ControlPoints<N>
where
    ControlPoint: From<C>,
{
    fn from(points: [C; N]) -> Self {
        Self {
            points: points.map(ControlPoint::from),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_vector3() {
        let v = Point3::new(1.0, 2.0, 3.0);
        let cp = ControlPoint::from(v);
        assert_eq!(v, cp.point);
    }

    #[test]
    fn from_vector3_ref() {
        let v = Point3::new(1.0, 2.0, 3.0);
        let cp = ControlPoint::from(&v);
        assert_eq!(v, cp.point);
    }

    #[test]
    fn from_control_point() {
        let v1 = Point3::new(1.0, 2.0, 3.0);
        let v2 = Point3::new(4.0, 5.0, 6.0);
        let cp = ControlPoints::from([v1, v2]);
        assert_eq!(EmitIntensity::MAX, cp.intensity);
        assert_eq!(v1, cp[0].point);
        assert_eq!(v2, cp[1].point);
    }
}
