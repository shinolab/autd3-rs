use autd3_driver::{
    datagram::{
        FociSTMContext, FociSTMContextGenerator, FociSTMGenerator, GainSTMContext,
        GainSTMContextGenerator, GainSTMGenerator, IntoFociSTMGenerator, IntoGainSTMGenerator,
    },
    defined::{ControlPoint, ControlPoints},
    derive::{EmitIntensity, Phase},
    geometry::Vector3,
};

use crate::gain::Focus;

#[derive(Clone, Debug)]
pub struct Line {
    pub start: Vector3,
    pub end: Vector3,
    pub num_points: usize,
    pub intensity: EmitIntensity,
}

pub struct LineSTMContext {
    start: Vector3,
    dir: Vector3,
    num_points: usize,
    wavenumber: f32,
    intensity: EmitIntensity,
    i: usize,
}

impl LineSTMContext {
    fn next(&mut self) -> Option<Vector3> {
        if self.i >= self.num_points {
            return None;
        }
        let f = self.start + self.dir * (self.i as f32 / (self.num_points - 1) as f32);
        self.i += 1;
        Some(f)
    }
}

impl FociSTMContext<1> for LineSTMContext {
    fn next(&mut self) -> ControlPoints<1> {
        ControlPoints::new([ControlPoint::from(self.next().unwrap())])
            .with_intensity(self.intensity)
    }
}

impl GainSTMContext for LineSTMContext {
    type Context = crate::gain::focus::Context;

    fn next(&mut self) -> Option<Self::Context> {
        Some(Self::Context {
            pos: self.next()?,
            intensity: self.intensity,
            phase_offset: Phase::ZERO,
            wavenumber: self.wavenumber,
        })
    }
}

impl FociSTMContextGenerator<1> for Line {
    type Context = LineSTMContext;

    fn generate(&mut self, device: &autd3_driver::derive::Device) -> Self::Context {
        Self::Context {
            start: self.start,
            dir: self.end - self.start,
            num_points: self.num_points,
            wavenumber: device.wavenumber(),
            intensity: self.intensity,
            i: 0,
        }
    }
}

impl GainSTMContextGenerator for Line {
    type Gain = Focus;
    type Context = LineSTMContext;

    fn generate(&mut self, device: &autd3_driver::derive::Device) -> Self::Context {
        FociSTMContextGenerator::<1>::generate(self, device)
    }
}

impl FociSTMGenerator<1> for Line {
    type T = Self;

    // GRCOV_EXCL_START
    fn init(self) -> Result<Self::T, autd3_driver::error::AUTDInternalError> {
        Ok(self)
    }
    // GRCOV_EXCL_STOP

    fn len(&self) -> usize {
        self.num_points
    }
}

impl GainSTMGenerator for Line {
    type T = Self;

    // GRCOV_EXCL_START
    fn init(
        self,
        _geometry: &autd3_driver::derive::Geometry,
        _filter: Option<&std::collections::HashMap<usize, bit_vec::BitVec<u32>>>,
    ) -> Result<Self::T, autd3_driver::error::AUTDInternalError> {
        Ok(self)
    }
    // GRCOV_EXCL_STOP

    fn len(&self) -> usize {
        self.num_points
    }
}

impl IntoFociSTMGenerator<1> for Line {
    type G = Self;

    fn into(self) -> Self::G {
        self
    }
}

impl IntoGainSTMGenerator for Line {
    type I = Self;

    fn into(self) -> Self::I {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::ops::DerefMut;

    use autd3_driver::{
        datagram::{FociSTM, GainSTM},
        defined::mm,
        derive::SamplingConfig,
    };

    use crate::assert_near_vector3;

    use super::*;

    #[test]
    fn line() {
        use autd3_driver::geometry::IntoDevice;

        let length = 30.0 * mm;
        let line = Line {
            start: Vector3::new(0., -length / 2., 0.),
            end: Vector3::new(0., length / 2., 0.),
            num_points: 3,
            intensity: EmitIntensity::MAX,
        };

        let expect = [
            Vector3::new(0., -length / 2., 0.),
            Vector3::zeros(),
            Vector3::new(0., length / 2., 0.),
        ];

        let device = autd3_driver::autd3_device::AUTD3::new(Vector3::zeros()).into_device(0);
        {
            let mut stm = FociSTM::new(SamplingConfig::FREQ_40K, line.clone()).unwrap();
            let mut context = FociSTMContextGenerator::generate(stm.deref_mut(), &device);
            expect.iter().for_each(|e| {
                let f = FociSTMContext::<1>::next(&mut context).points()[0];
                assert_near_vector3!(e, f.point());
            });
            assert!(context.next().is_none());
        }
        {
            let mut stm = GainSTM::new(SamplingConfig::FREQ_40K, line.clone()).unwrap();
            let mut context = GainSTMContextGenerator::generate(stm.deref_mut(), &device);
            expect.iter().for_each(|e| {
                let f = GainSTMContext::next(&mut context).unwrap();
                assert_near_vector3!(e, &f.pos);
            });
            assert!(context.next().is_none());
        }
    }
}
