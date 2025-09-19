use std::num::NonZeroUsize;

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

use nalgebra::{ComplexField, Normed};

/// The option of [`GS`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GSOption {
    /// The number of iterations.
    pub repeat: NonZeroUsize,
    /// The transducers' emission constraint.
    pub constraint: EmissionConstraint,
}

impl Default for GSOption {
    fn default() -> Self {
        Self {
            repeat: NonZeroUsize::new(100).unwrap(),
            constraint: EmissionConstraint::Clamp(Intensity::MIN, Intensity::MAX),
        }
    }
}

/// Gerchberg-Saxton algorithm
///
/// See [Marzo, et al., 2019](https://www.pnas.org/doi/full/10.1073/pnas.1813047115) for more details.
#[derive(Gain, Debug)]
pub struct GS<D: Directivity> {
    /// The focal positions and amplitudes.
    pub foci: Vec<(Point3, Amplitude)>,
    /// The option of the Gain.
    pub option: GSOption,
    /// The directivity of the transducers.
    pub directivity: std::marker::PhantomData<D>,
}

impl GS<Sphere> {
    /// Create a new [`GS`].
    #[must_use]
    pub fn new(foci: impl IntoIterator<Item = (Point3, Amplitude)>, option: GSOption) -> Self {
        Self::with_directivity(foci, option)
    }
}

impl<D: Directivity> GS<D> {
    /// Create a new [`GS`] with directivity.
    #[must_use]
    pub fn with_directivity(
        foci: impl IntoIterator<Item = (Point3, Amplitude)>,
        option: GSOption,
    ) -> Self {
        Self {
            foci: foci.into_iter().collect(),
            option,
            directivity: std::marker::PhantomData,
        }
    }
}

impl<D: Directivity> Gain<'_> for GS<D> {
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
        let ones = vec![1.; n];

        let b = gen_back_prop(n, m, &g);

        let mut q = VectorXc::from_iterator(ones.len(), ones.iter().map(|&r| Complex::new(r, 0.)));
        let q0 = q.clone();

        let amps = VectorXc::from_iterator(
            amps.len(),
            amps.into_iter().map(|a| Complex::new(a.pascal(), 0.)),
        );
        let mut p = VectorXc::zeros(m);
        (0..self.option.repeat.get()).for_each(|_| {
            q.zip_apply(&q0, |b, a| *b = *b / b.abs() * a);
            p.gemv(Complex::new(1., 0.), &g, &q, Complex::new(0., 0.));
            p.zip_apply(&amps, |b, a| *b = *b / b.abs() * a);
            q.gemv(Complex::new(1., 0.), &b, &p, Complex::new(0., 0.));
        });

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
    fn gs_option_default() {
        let option = GSOption::default();
        assert_eq!(option.repeat, NonZeroUsize::new(100).unwrap());
        assert_eq!(
            option.constraint,
            EmissionConstraint::Clamp(Intensity::MIN, Intensity::MAX)
        );
    }

    #[test]
    fn test_gs_all() {
        let geometry = create_geometry(1, 1);

        let g = GS::new(
            vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            GSOption {
                repeat: NonZeroUsize::new(5).unwrap(),
                constraint: EmissionConstraint::Uniform(Intensity::MAX),
            },
        );

        assert_eq!(
            g.init(&geometry, &Environment::new(), &TransducerMask::AllEnabled)
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
    fn test_gs_filtered() {
        let geometry = create_geometry(2, 1);

        let g = GS {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: GSOption {
                repeat: NonZeroUsize::new(5).unwrap(),
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
}
