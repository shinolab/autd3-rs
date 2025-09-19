use crate::{
    Amplitude, Complex, VectorXc,
    constraint::EmissionConstraint,
    helper::{
        HoloCalculatorGenerator, gen_back_prop, generate_propagation_matrix, generate_result,
    },
};

use autd3_core::{
    acoustics::directivity::{Directivity, Sphere},
    derive::*,
    geometry::Point3,
};
use nalgebra::Normed;

/// The option of [`Naive`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NaiveOption {
    /// The transducers' emission constraint.
    pub constraint: EmissionConstraint,
}

impl Default for NaiveOption {
    fn default() -> Self {
        Self {
            constraint: EmissionConstraint::Clamp(Intensity::MIN, Intensity::MAX),
        }
    }
}

/// Naive linear synthesis of simple focal solutions
#[derive(Gain, Debug)]
pub struct Naive<D: Directivity> {
    /// The focal positions and amplitudes.
    pub foci: Vec<(Point3, Amplitude)>,
    /// The option of the Gain.
    pub option: NaiveOption,
    /// The directivity of the transducers.
    pub directivity: std::marker::PhantomData<D>,
}

impl Naive<Sphere> {
    /// Create a new [`Naive`].
    #[must_use]
    pub fn new(foci: impl IntoIterator<Item = (Point3, Amplitude)>, option: NaiveOption) -> Self {
        Self::with_directivity(foci, option)
    }
}

impl<D: Directivity> Naive<D> {
    /// Create a new [`Naive`] with directivity.
    #[must_use]
    pub fn with_directivity(
        foci: impl IntoIterator<Item = (Point3, Amplitude)>,
        option: NaiveOption,
    ) -> Self {
        Self {
            foci: foci.into_iter().collect(),
            option,
            directivity: std::marker::PhantomData,
        }
    }
}

impl<D: Directivity> Gain<'_> for Naive<D> {
    type G = HoloCalculatorGenerator;

    fn init(
        self,
        geometry: &Geometry,
        env: &Environment,
        filter: &TransducerMask,
    ) -> Result<Self::G, GainError> {
        let (foci, amps): (Vec<_>, Vec<_>) = self.foci.into_iter().unzip();

        let g = generate_propagation_matrix::<D>(geometry, env, &foci, filter);

        let m = foci.len();
        let n = g.ncols();

        let b = gen_back_prop(n, m, &g);

        let p = VectorXc::from_iterator(
            amps.len(),
            amps.into_iter().map(|a| Complex::new(a.pascal(), 0.)),
        );
        let mut q = VectorXc::zeros(n);
        q.gemv(Complex::new(1., 0.), &b, &p, Complex::new(0., 0.));

        let max_coefficient = q.map(|v| v.norm_squared()).max().sqrt();
        generate_result(geometry, q, max_coefficient, self.option.constraint, filter)
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::{
        firmware::Drive,
        gain::{GainCalculator, GainCalculatorGenerator},
    };

    use crate::tests::create_geometry;

    use super::{super::super::Pa, *};

    #[test]
    fn naive_option_default() {
        let option = NaiveOption::default();
        assert_eq!(
            option.constraint,
            EmissionConstraint::Clamp(Intensity::MIN, Intensity::MAX)
        );
    }

    #[test]
    fn test_naive_all() {
        let geometry = create_geometry(1, 1);

        let g = Naive::new(
            vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            NaiveOption {
                constraint: EmissionConstraint::Uniform(Intensity::MAX),
            },
        );

        assert_eq!(
            g.init(&geometry, &Environment::new(), &TransducerMask::AllEnabled,)
                .map(|mut res| {
                    let f = res.generate(&geometry[0]);
                    geometry[0]
                        .iter()
                        .filter(|tr| f.calc(tr) != Drive::NULL)
                        .count()
                }),
            Ok(geometry.num_transducers()),
        );
    }

    #[test]
    fn test_naive_all_disabled() -> Result<(), Box<dyn std::error::Error>> {
        let geometry = create_geometry(2, 1);

        let g = Naive {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: NaiveOption {
                constraint: EmissionConstraint::Uniform(Intensity::MAX),
            },
            directivity: std::marker::PhantomData::<Sphere>,
        };

        let mut g = g.init(
            &geometry,
            &Environment::new(),
            &TransducerMask::new([
                DeviceTransducerMask::AllDisabled,
                DeviceTransducerMask::AllEnabled,
            ]),
        )?;
        let f = g.generate(&geometry[1]);
        assert_eq!(
            geometry[1]
                .iter()
                .filter(|tr| f.calc(tr) != Drive::NULL)
                .count(),
            geometry[1].num_transducers()
        );

        Ok(())
    }

    #[test]
    fn test_naive_filtered() {
        let geometry = create_geometry(2, 1);

        let g = Naive {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: NaiveOption {
                constraint: EmissionConstraint::Uniform(Intensity::MAX),
            },
            directivity: std::marker::PhantomData::<Sphere>,
        };

        let filter = TransducerMask::from_fn(&geometry, |dev| {
            if dev.idx() == 0 {
                DeviceTransducerMask::from_fn(dev, |tr: &Transducer| tr.idx() < 100)
            } else {
                DeviceTransducerMask::AllDisabled
            }
        });
        let mut g = g.init(&geometry, &Environment::new(), &filter).unwrap();
        assert_eq!(
            {
                let f = g.generate(&geometry[0]);
                geometry[0]
                    .iter()
                    .filter(|tr| f.calc(tr) != Drive::NULL)
                    .count()
            },
            100,
        );
        assert_eq!(
            {
                let f = g.generate(&geometry[1]);
                geometry[1]
                    .iter()
                    .filter(|tr| f.calc(tr) != Drive::NULL)
                    .count()
            },
            0,
        );
    }

    #[test]
    fn test_naive_filtered_disabled() -> Result<(), Box<dyn std::error::Error>> {
        let geometry = create_geometry(2, 1);

        let g = Naive {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: NaiveOption {
                constraint: EmissionConstraint::Uniform(Intensity::MAX),
            },
            directivity: std::marker::PhantomData::<Sphere>,
        };

        let filter = TransducerMask::from_fn(&geometry, |dev| {
            if dev.idx() == 0 {
                DeviceTransducerMask::AllDisabled
            } else {
                DeviceTransducerMask::from_fn(dev, |tr: &Transducer| tr.idx() < 100)
            }
        });
        let mut g = g.init(&geometry, &Environment::new(), &filter)?;
        let f = g.generate(&geometry[1]);
        assert_eq!(
            geometry[1]
                .iter()
                .filter(|tr| f.calc(tr) != Drive::NULL)
                .count(),
            100
        );

        Ok(())
    }
}
