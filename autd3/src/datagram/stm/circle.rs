use std::{collections::HashMap, f32::consts::PI};

use autd3_core::{
    derive::{Device, Geometry},
    gain::{BitVec, EmitIntensity, GainError, Phase},
};
use autd3_driver::{
    datagram::{
        ControlPoint, ControlPoints, FociSTMGenerator, FociSTMIterator, FociSTMIteratorGenerator,
        GainSTMGenerator, GainSTMIterator, GainSTMIteratorGenerator,
    },
    error::AUTDDriverError,
    geometry::{Point3, UnitVector3, Vector3},
};

use crate::gain::Focus;

/// Utility for generating a circular trajectory STM.
///
/// # Examples
///
/// ```
/// use autd3::prelude::*;
///
/// FociSTM {
///     config: 1.0 * Hz,
///     foci: Circle {
///         center: Point3::origin(),
///         radius: 30.0 * mm,
///         num_points: 50,
///         n: Vector3::z_axis(),
///         intensity: EmitIntensity::MAX,
///     },
/// };
/// ```
#[derive(Clone, Debug)]
pub struct Circle {
    /// The center of the circle.
    pub center: Point3,
    /// The radius of the circle.
    pub radius: f32,
    /// The number of points on the circle.
    pub num_points: usize,
    /// The normal vector of the circle.
    pub n: UnitVector3,
    /// The intensity of the emitted ultrasound.
    pub intensity: EmitIntensity,
}

pub struct CircleSTMIterator {
    center: Point3,
    radius: f32,
    num_points: usize,
    u: Vector3,
    v: Vector3,
    wavenumber: f32,
    intensity: EmitIntensity,
    i: usize,
}

impl CircleSTMIterator {
    fn next(&mut self) -> Option<Point3> {
        if self.i >= self.num_points {
            return None;
        }
        let theta = 2.0 * PI * self.i as f32 / self.num_points as f32;
        self.i += 1;
        Some(self.center + self.radius * (theta.cos() * self.u + theta.sin() * self.v))
    }
}

impl FociSTMIterator<1> for CircleSTMIterator {
    fn next(&mut self) -> ControlPoints<1> {
        ControlPoints {
            points: [ControlPoint::from(self.next().unwrap())],
            intensity: self.intensity,
        }
    }
}

impl GainSTMIterator for CircleSTMIterator {
    type Calculator = crate::gain::focus::Impl;

    fn next(&mut self) -> Option<Self::Calculator> {
        Some(Self::Calculator {
            pos: self.next()?,
            intensity: self.intensity,
            phase_offset: Phase::ZERO,
            wavenumber: self.wavenumber,
        })
    }
}

impl FociSTMIteratorGenerator<1> for Circle {
    type Iterator = CircleSTMIterator;

    fn generate(&mut self, device: &Device) -> Self::Iterator {
        let v = if self.n.dot(&Vector3::z()).abs() < 0.9 {
            Vector3::z()
        } else {
            Vector3::y()
        };
        let u = self.n.cross(&v).normalize();
        let v = self.n.cross(&u).normalize();
        Self::Iterator {
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
}

impl GainSTMIteratorGenerator for Circle {
    type Gain = Focus;
    type Iterator = CircleSTMIterator;

    fn generate(&mut self, device: &Device) -> Self::Iterator {
        FociSTMIteratorGenerator::<1>::generate(self, device)
    }
}

impl FociSTMGenerator<1> for Circle {
    type T = Self;

    // GRCOV_EXCL_START
    fn init(self) -> Result<Self::T, AUTDDriverError> {
        Ok(self)
    }
    // GRCOV_EXCL_STOP

    fn len(&self) -> usize {
        self.num_points
    }
}

impl GainSTMGenerator for Circle {
    type T = Self;

    // GRCOV_EXCL_START
    fn init(
        self,
        _: &Geometry,
        _filter: Option<&HashMap<usize, BitVec>>,
    ) -> Result<Self::T, GainError> {
        Ok(self)
    }
    // GRCOV_EXCL_STOP

    fn len(&self) -> usize {
        self.num_points
    }
}

#[cfg(test)]
mod tests {
    use std::ops::DerefMut;

    use autd3_core::sampling_config::SamplingConfig;
    use autd3_driver::{
        common::mm,
        datagram::{FociSTM, GainSTM},
    };

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
        use autd3_driver::datagram::GainSTMOption;

        let circle = Circle {
            center: Point3::origin(),
            radius: 30.0 * mm,
            num_points: 4,
            n,
            intensity: EmitIntensity::MAX,
        };
        assert_eq!(4, FociSTMGenerator::len(&circle));
        assert_eq!(4, GainSTMGenerator::len(&circle));

        let device = autd3_driver::autd3_device::AUTD3::default().into();
        {
            let mut stm = FociSTM {
                foci: circle.clone(),
                config: SamplingConfig::FREQ_4K,
            };
            let mut iterator = FociSTMIteratorGenerator::generate(stm.deref_mut(), &device);
            expect.iter().for_each(|e| {
                let f = FociSTMIterator::<1>::next(&mut iterator).points[0];
                assert_near_vector3!(e, f.point);
            });
            assert!(iterator.next().is_none());
        }
        {
            let mut stm = GainSTM {
                gains: circle.clone(),
                config: SamplingConfig::FREQ_4K,
                option: GainSTMOption::default(),
            };
            let mut iterator = GainSTMIteratorGenerator::generate(stm.deref_mut(), &device);
            expect.iter().for_each(|e| {
                let f = GainSTMIterator::next(&mut iterator).unwrap();
                assert_near_vector3!(e, &f.pos);
            });
            assert!(iterator.next().is_none());
        }
    }
}
