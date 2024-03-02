use crate::{common::EmitIntensity, derive::*, geometry::Vector3};

/// Control point for FocusSTM
#[derive(Clone, Copy, Builder)]
pub struct ControlPoint {
    /// Focal point
    #[getset]
    point: Vector3,
    /// Emission intensity
    #[getset]
    intensity: EmitIntensity,
}

impl ControlPoint {
    /// constructor (shift is 0)
    ///
    /// # Arguments
    ///
    /// * `point` - focal point
    ///
    pub const fn new(point: Vector3) -> Self {
        Self {
            point,
            intensity: EmitIntensity::MAX,
        }
    }
}

impl From<Vector3> for ControlPoint {
    fn from(point: Vector3) -> Self {
        Self::new(point)
    }
}

impl<I: Into<EmitIntensity>> From<(Vector3, I)> for ControlPoint {
    fn from((point, intensity): (Vector3, I)) -> Self {
        Self::new(point).with_intensity(intensity)
    }
}

impl From<&Vector3> for ControlPoint {
    fn from(point: &Vector3) -> Self {
        Self::new(*point)
    }
}

impl<I: Into<EmitIntensity> + Clone> From<&(Vector3, I)> for ControlPoint {
    fn from((point, intensity): &(Vector3, I)) -> Self {
        Self::new(*point).with_intensity(intensity.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_point() {
        let c = ControlPoint::from(Vector3::new(1., 2., 3.));
        assert_eq!(c.point().x, 1.);
        assert_eq!(c.point().y, 2.);
        assert_eq!(c.point().z, 3.);
        assert_eq!(c.intensity(), EmitIntensity::MAX);

        let c = ControlPoint::from((Vector3::new(1., 2., 3.), 4));
        assert_eq!(c.point().x, 1.);
        assert_eq!(c.point().y, 2.);
        assert_eq!(c.point().z, 3.);
        assert_eq!(c.intensity(), EmitIntensity::new(4));

        let c = ControlPoint::from(&Vector3::new(1., 2., 3.));
        assert_eq!(c.point().x, 1.);
        assert_eq!(c.point().y, 2.);
        assert_eq!(c.point().z, 3.);
        assert_eq!(c.intensity(), EmitIntensity::MAX);

        let c = ControlPoint::from(&(Vector3::new(1., 2., 3.), EmitIntensity::new(4)));
        assert_eq!(c.point().x, 1.);
        assert_eq!(c.point().y, 2.);
        assert_eq!(c.point().z, 3.);
        assert_eq!(c.intensity(), EmitIntensity::new(4));

        let cc = Clone::clone(&c);
        assert_eq!(cc.point().x, 1.);
        assert_eq!(cc.point().y, 2.);
        assert_eq!(cc.point().z, 3.);
        assert_eq!(c.intensity(), EmitIntensity::new(4));
    }
}
