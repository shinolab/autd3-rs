mod foci;
mod gain;

use autd3_driver::{
    datagram::{STMConfig, STMConfigNearest},
    geometry::Vector3,
};

use crate::error::AUTDError;

pub trait STMUtilsExt {
    type STM;

    fn line(
        config: impl Into<STMConfig>,
        dir: Vector3,
        num_points: usize,
        center: Vector3,
    ) -> Result<Self::STM, AUTDError>;
    fn line_nearest(
        config: impl Into<STMConfigNearest>,
        dir: Vector3,
        num_points: usize,
        center: Vector3,
    ) -> Result<Self::STM, AUTDError>;
    fn circle(
        config: impl Into<STMConfig>,
        radius: f32,
        num_points: usize,
        center: Vector3,
    ) -> Result<Self::STM, AUTDError>;
    fn circle_nearest(
        config: impl Into<STMConfigNearest>,
        radius: f32,
        num_points: usize,
        center: Vector3,
    ) -> Result<Self::STM, AUTDError>;
}
