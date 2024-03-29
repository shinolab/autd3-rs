use crate::{derive::*, geometry::Vector3};

/// Control point for FocusSTM
#[derive(Clone, Copy, Builder, PartialEq, Debug)]
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
