use autd3_core::{
    environment::Environment,
    firmware::{Intensity, Phase},
    gain::{GainError, TransducerMask},
    geometry::{Device, Geometry, Isometry3},
};
use autd3_driver::{
    datagram::{
        ControlPoint, ControlPoints, FociSTMGenerator, FociSTMIterator, FociSTMIteratorGenerator,
        GainSTMGenerator, GainSTMIterator, GainSTMIteratorGenerator,
    },
    error::AUTDDriverError,
    geometry::{Point3, Vector3},
};

/// Utility for generating a line STM.
///
/// # Examples
///
/// ```
/// use autd3::prelude::*;
///
/// FociSTM {
///     config: 1.0 * Hz,
///     foci: Line {
///         start: Point3::new(-15.0 * mm, 0., 0.),
///         end: Point3::new(15.0 * mm, 0., 0.),
///         num_points: 50,
///         intensity: Intensity::MAX,
///     },
/// };
/// ```
#[derive(Clone, Debug)]
pub struct Line {
    /// The start point of the line.
    pub start: Point3,
    /// The end point of the line.
    pub end: Point3,
    /// The number of points on the line.
    pub num_points: usize,
    /// The intensity of the emitted ultrasound.
    pub intensity: Intensity,
}

#[derive(Clone, Debug)]
pub struct LineSTMIterator {
    start: Point3,
    dir: Vector3,
    num_points: usize,
    wavenumber: f32,
    intensity: Intensity,
    i: usize,
}

impl LineSTMIterator {
    fn next(&mut self) -> Option<Point3> {
        if self.i >= self.num_points {
            return None;
        }
        let f = self.start + self.dir * (self.i as f32 / (self.num_points - 1) as f32);
        self.i += 1;
        Some(f)
    }
}

impl FociSTMIterator<1> for LineSTMIterator {
    fn next(&mut self, iso: &Isometry3) -> ControlPoints<1> {
        ControlPoints {
            points: [ControlPoint::from(iso * self.next().unwrap())],
            intensity: self.intensity,
        }
    }
}

impl GainSTMIterator<'_> for LineSTMIterator {
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

impl FociSTMIteratorGenerator<1> for LineSTMIterator {
    type Iterator = Self;

    fn generate(&mut self, _: &Device) -> Self::Iterator {
        self.clone()
    }
}

impl GainSTMIteratorGenerator<'_> for LineSTMIterator {
    type Gain = crate::gain::focus::Impl;
    type Iterator = Self;

    fn generate(&mut self, _: &Device) -> Self::Iterator {
        self.clone()
    }
}

impl GainSTMGenerator<'_> for Line {
    type T = LineSTMIterator;

    fn init(
        self,
        _: &Geometry,
        env: &Environment,
        _filter: &TransducerMask,
    ) -> Result<Self::T, GainError> {
        Ok(LineSTMIterator {
            start: self.start,
            dir: self.end - self.start,
            num_points: self.num_points,
            wavenumber: env.wavenumber(),
            intensity: self.intensity,
            i: 0,
        })
    }

    fn len(&self) -> usize {
        self.num_points
    }
}

impl FociSTMGenerator<1> for Line {
    type T = LineSTMIterator;

    fn init(self) -> Result<Self::T, AUTDDriverError> {
        Ok(LineSTMIterator {
            start: self.start,
            dir: self.end - self.start,
            num_points: self.num_points,
            wavenumber: 0.,
            intensity: self.intensity,
            i: 0,
        })
    }

    fn len(&self) -> usize {
        self.num_points
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::common::mm;

    use crate::assert_near_vector3;

    use super::*;

    #[test]
    fn line() -> Result<(), Box<dyn std::error::Error>> {
        let env = Environment::default();

        let length = 30.0 * mm;
        let line = Line {
            start: Point3::new(0., -length / 2., 0.),
            end: Point3::new(0., length / 2., 0.),
            num_points: 3,
            intensity: Intensity::MAX,
        };
        assert_eq!(3, FociSTMGenerator::len(&line));
        assert_eq!(3, GainSTMGenerator::len(&line));

        let expect = [
            Point3::new(0., -length / 2., 0.),
            Point3::origin(),
            Point3::new(0., length / 2., 0.),
        ];

        let device = autd3_core::devices::AUTD3::default().into();
        {
            let mut g = FociSTMGenerator::init(line.clone())?;
            let mut iterator = FociSTMIteratorGenerator::generate(&mut g, &device);
            expect.iter().for_each(|e| {
                let f = FociSTMIterator::<1>::next(&mut iterator, device.inv()).points[0];
                assert_near_vector3!(e, f.point);
            });
            assert!(iterator.next().is_none());
        }
        {
            let mut g = GainSTMGenerator::init(
                line,
                &Geometry::new(vec![]),
                &env,
                &TransducerMask::AllEnabled,
            )?;
            let mut iterator = GainSTMIteratorGenerator::generate(&mut g, &device);
            expect.iter().for_each(|e| {
                let f = GainSTMIterator::next(&mut iterator).unwrap();
                assert_near_vector3!(e, &f.pos);
            });
            assert!(iterator.next().is_none());
        }

        Ok(())
    }
}
