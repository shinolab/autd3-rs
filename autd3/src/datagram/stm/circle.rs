use std::f32::consts::PI;

use autd3_driver::{
    datagram::{
        FociSTMContext, FociSTMContextGenerator, FociSTMGenerator, GainSTMContext,
        GainSTMContextGenerator, GainSTMGenerator, IntoGainSTMGenerator,
    },
    defined::{ControlPoint, ControlPoints},
    derive::{EmitIntensity, Phase},
    geometry::{UnitVector3, Vector3},
};

use crate::gain::Focus;

#[derive(Clone, Debug)]
pub struct Circle {
    pub center: Vector3,
    pub radius: f32,
    pub num_points: usize,
    pub n: UnitVector3,
    pub intensity: EmitIntensity,
}

pub struct CircleSTMContext {
    center: Vector3,
    radius: f32,
    num_points: usize,
    u: Vector3,
    v: Vector3,
    wavenumber: f32,
    intensity: EmitIntensity,
    i: usize,
}

impl CircleSTMContext {
    fn next(&mut self) -> Option<Vector3> {
        if self.i >= self.num_points {
            return None;
        }
        let theta = 2.0 * PI * self.i as f32 / self.num_points as f32;
        self.i += 1;
        Some(self.center + self.radius * (theta.cos() * self.u + theta.sin() * self.v))
    }
}

impl FociSTMContext<1> for CircleSTMContext {
    fn next(&mut self) -> ControlPoints<1> {
        ControlPoints::new([ControlPoint::from(self.next().unwrap())])
            .with_intensity(self.intensity)
    }
}

impl GainSTMContext for CircleSTMContext {
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

impl FociSTMContextGenerator<1> for Circle {
    type Context = CircleSTMContext;

    fn generate(&mut self, device: &autd3_driver::derive::Device) -> Self::Context {
        let v = if self.n.dot(&Vector3::z()).abs() < 0.9 {
            Vector3::z()
        } else {
            Vector3::y()
        };
        let u = self.n.cross(&v).normalize();
        let v = self.n.cross(&u).normalize();
        Self::Context {
            center: self.center,
            radius: self.radius,
            num_points: self.num_points,
            u,
            v,
            wavenumber: device.wavenumber(),
            intensity: self.intensity,
            i: 0,
        }
    }

    fn len(&self) -> usize {
        self.num_points
    }
}

impl GainSTMContextGenerator for Circle {
    type Gain = Focus;
    type Context = CircleSTMContext;

    fn generate(&mut self, device: &autd3_driver::derive::Device) -> Self::Context {
        FociSTMContextGenerator::<1>::generate(self, device)
    }
}

impl GainSTMGenerator for Circle {
    type T = Self;

    fn init(
        self,
        _geometry: &autd3_driver::derive::Geometry,
        _filter: Option<&std::collections::HashMap<usize, bit_vec::BitVec<u32>>>,
    ) -> Result<Self::T, autd3_driver::error::AUTDInternalError> {
        Ok(self)
    }

    fn len(&self) -> usize {
        self.num_points
    }
}

impl FociSTMGenerator<1> for Circle {
    type G = Self;

    fn into(self) -> Self::G {
        self
    }
}

impl IntoGainSTMGenerator for Circle {
    type I = Self;

    fn into(self) -> Self::I {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::ops::DerefMut;

    use autd3_driver::{datagram::FociSTM, defined::mm};

    use crate::assert_near_vector3;

    use super::*;

    #[rstest::rstest]
    #[case(
        vec![
            Vector3::new(0., -30.0 * mm, 0.),
            Vector3::new(0., 0., -30.0 * mm),
            Vector3::new(0., 30.0 * mm, 0.),
            Vector3::new(0., 0., 30.0 * mm),
        ]
        ,
        Vector3::x_axis()
    )]
    #[case(
        vec![
            Vector3::new(30.0 * mm, 0., 0.),
            Vector3::new(0., 0., -30.0 * mm),
            Vector3::new(-30.0 * mm, 0., 0.),
            Vector3::new(0., 0., 30.0 * mm),
        ]
        ,
        Vector3::y_axis()
    )]
    #[case(
        vec![
            Vector3::new(-30.0 * mm, 0., 0.),
            Vector3::new(0., -30.0 * mm, 0.),
            Vector3::new(30.0 * mm, 0., 0.),
            Vector3::new(0., 30.0 * mm, 0.),
        ]
        ,
        Vector3::z_axis()
    )]
    #[test]
    fn circle(#[case] expect: Vec<Vector3>, #[case] n: UnitVector3) {
        use autd3_driver::{datagram::GainSTM, derive::SamplingConfig, geometry::IntoDevice};

        let circle = Circle {
            center: Vector3::zeros(),
            radius: 30.0 * mm,
            num_points: 4,
            n,
            intensity: EmitIntensity::MAX,
        };

        let device = autd3_driver::autd3_device::AUTD3::new(Vector3::zeros()).into_device(0);
        {
            let mut stm = FociSTM::new(SamplingConfig::FREQ_40K, circle.clone()).unwrap();
            let mut context = FociSTMContextGenerator::generate(stm.deref_mut(), &device);
            expect.iter().for_each(|e| {
                let f = FociSTMContext::<1>::next(&mut context).points()[0];
                assert_near_vector3!(e, f.point());
            });
            assert!(context.next().is_none());
        }
        {
            let mut stm = GainSTM::new(SamplingConfig::FREQ_40K, circle.clone()).unwrap();
            let mut context = GainSTMContextGenerator::generate(stm.deref_mut(), &device);
            expect.iter().for_each(|e| {
                let f = GainSTMContext::next(&mut context).unwrap();
                assert_near_vector3!(e, &f.pos);
            });
            assert!(context.next().is_none());
        }
    }
}
