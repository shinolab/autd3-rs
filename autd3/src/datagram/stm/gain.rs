use std::f32::consts::PI;

use autd3_driver::{
    datagram::{GainSTM, STMConfigNearest},
    geometry::Vector3,
};

use crate::{error::AUTDError, gain::Focus};

use super::STMUtilsExt;

impl STMUtilsExt for GainSTM<Focus> {
    type STM = Self;

    fn line(
        config: impl Into<autd3_driver::datagram::STMConfig>,
        dir: Vector3,
        num_points: usize,
        center: Vector3,
    ) -> Result<Self::STM, crate::prelude::AUTDError> {
        let start = center - dir / 2.;
        Ok(GainSTM::new(
            config,
            (0..num_points).map(|i| Focus::new(start + dir * (i as f32 / (num_points - 1) as f32))),
        )?)
    }

    fn line_nearest(
        config: impl Into<autd3_driver::datagram::STMConfigNearest>,
        dir: Vector3,
        num_points: usize,
        center: Vector3,
    ) -> Result<Self::STM, crate::prelude::AUTDError> {
        let start = center - dir / 2.;
        Ok(GainSTM::new_nearest(
            config,
            (0..num_points).map(|i| Focus::new(start + dir * (i as f32 / (num_points - 1) as f32))),
        )?)
    }

    fn circle(
        config: impl Into<autd3_driver::datagram::STMConfig>,
        radius: f32,
        num_points: usize,
        center: Vector3,
    ) -> Result<Self::STM, AUTDError> {
        Ok(GainSTM::new(
            config,
            (0..num_points).map(|i| {
                let theta = 2.0 * PI * i as f32 / num_points as f32;
                let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
                Focus::new(center + p)
            }),
        )?)
    }

    fn circle_nearest(
        config: impl Into<STMConfigNearest>,
        radius: f32,
        num_points: usize,
        center: Vector3,
    ) -> Result<Self::STM, AUTDError> {
        Ok(GainSTM::new_nearest(
            config,
            (0..num_points).map(|i| {
                let theta = 2.0 * PI * i as f32 / num_points as f32;
                let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
                Focus::new(center + p)
            }),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::{
        defined::{mm, ULTRASOUND_PERIOD},
        derive::SamplingConfig,
    };

    use crate::assert_near_vector3;

    use super::*;

    #[test]
    fn line() -> anyhow::Result<()> {
        let point_num = 3;
        let length = 30.0 * mm;
        let dir = Vector3::new(0., length, 0.);
        let center = Vector3::zeros();
        let stm = GainSTM::line(SamplingConfig::FREQ_40K, dir, point_num, center)?;

        assert_eq!(SamplingConfig::FREQ_40K, stm.sampling_config());
        assert_near_vector3!(&Vector3::new(0., -length / 2., 0.), stm[0].pos());
        assert_near_vector3!(&Vector3::zeros(), stm[1].pos());
        assert_near_vector3!(&Vector3::new(0., length / 2., 0.), stm[2].pos());

        Ok(())
    }

    #[test]
    fn line_nearest() -> anyhow::Result<()> {
        let point_num = 3;
        let length = 30.0 * mm;
        let dir = Vector3::new(0., length, 0.);
        let center = Vector3::zeros();
        let stm = GainSTM::line_nearest(ULTRASOUND_PERIOD, dir, point_num, center)?;

        assert_eq!(SamplingConfig::FREQ_40K, stm.sampling_config());
        assert_near_vector3!(&Vector3::new(0., -length / 2., 0.), stm[0].pos());
        assert_near_vector3!(&Vector3::zeros(), stm[1].pos());
        assert_near_vector3!(&Vector3::new(0., length / 2., 0.), stm[2].pos());

        Ok(())
    }

    #[test]
    fn circle() -> anyhow::Result<()> {
        let point_num = 4;
        let radius = 30.0 * mm;
        let center = Vector3::zeros();
        let stm = GainSTM::circle(SamplingConfig::FREQ_40K, radius, point_num, center)?;

        assert_eq!(SamplingConfig::FREQ_40K, stm.sampling_config());
        assert_near_vector3!(&Vector3::new(radius, 0., 0.), stm[0].pos());
        assert_near_vector3!(&Vector3::new(0., radius, 0.), stm[1].pos());
        assert_near_vector3!(&Vector3::new(-radius, 0., 0.), stm[2].pos());
        assert_near_vector3!(&Vector3::new(0., -radius, 0.), stm[3].pos());

        Ok(())
    }

    #[test]
    fn circle_nearest() -> anyhow::Result<()> {
        let point_num = 4;
        let radius = 30.0 * mm;
        let center = Vector3::zeros();
        let stm = GainSTM::circle_nearest(ULTRASOUND_PERIOD, radius, point_num, center)?;

        assert_eq!(SamplingConfig::FREQ_40K, stm.sampling_config());
        assert_near_vector3!(&Vector3::new(radius, 0., 0.), stm[0].pos());
        assert_near_vector3!(&Vector3::new(0., radius, 0.), stm[1].pos());
        assert_near_vector3!(&Vector3::new(-radius, 0., 0.), stm[2].pos());
        assert_near_vector3!(&Vector3::new(0., -radius, 0.), stm[3].pos());

        Ok(())
    }
}
