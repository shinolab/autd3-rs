use std::num::NonZeroUsize;

use crate::{
    Amplitude, Complex, MatrixXc, VectorXc,
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

/// The option of [`GSPAT`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GSPATOption {
    /// The number of iterations.
    pub repeat: NonZeroUsize,
    /// The transducers' emission constraint.
    pub constraint: EmissionConstraint,
}

impl Default for GSPATOption {
    fn default() -> Self {
        Self {
            repeat: NonZeroUsize::new(100).unwrap(),
            constraint: EmissionConstraint::Clamp(Intensity::MIN, Intensity::MAX),
        }
    }
}

/// Gerchberg-Saxon for Phased Arrays of Transducers
///
/// See [Plasencia, et al., 2020](https://dl.acm.org/doi/10.1145/3386569.3392492) for more details.
#[derive(Gain, Debug)]
pub struct GSPAT<D: Directivity> {
    /// The focal positions and amplitudes.
    pub foci: Vec<(Point3, Amplitude)>,
    /// The option of the Gain.
    pub option: GSPATOption,
    /// The directivity of the transducers.
    pub directivity: std::marker::PhantomData<D>,
}

impl GSPAT<Sphere> {
    /// Create a new [`GSPAT`].
    #[must_use]
    pub fn new(foci: impl IntoIterator<Item = (Point3, Amplitude)>, option: GSPATOption) -> Self {
        Self::with_directivity(foci, option)
    }
}

impl<D: Directivity> GSPAT<D> {
    /// Create a new [`GSPAT`] with directivity.
    #[must_use]
    pub fn with_directivity(
        foci: impl IntoIterator<Item = (Point3, Amplitude)>,
        option: GSPATOption,
    ) -> Self {
        Self {
            foci: foci.into_iter().collect(),
            option,
            directivity: std::marker::PhantomData,
        }
    }
}

impl<D: Directivity> Gain<'_> for GSPAT<D> {
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

        let mut q = VectorXc::zeros(n);

        let amps = VectorXc::from_iterator(
            amps.len(),
            amps.into_iter().map(|a| Complex::new(a.pascal(), 0.)),
        );

        let b = gen_back_prop(n, m, &g);

        let mut r = MatrixXc::zeros(m, m);
        r.gemm(Complex::new(1., 0.), &g, &b, Complex::new(0., 0.));

        let mut p = amps.clone();
        let mut gamma = amps.clone();
        gamma.gemv(Complex::new(1., 0.), &r, &p, Complex::new(0., 0.));
        (0..self.option.repeat.get()).for_each(|_| {
            p = gamma.zip_map(&amps, |a, b| a / a.norm() * b);
            gamma.gemv(Complex::new(1., 0.), &r, &p, Complex::new(0., 0.));
        });

        q.gemv(Complex::new(1., 0.), &b, &p, Complex::new(0., 0.));

        let max_coefficient = q.map(|v| v.norm_sqr()).max().sqrt();
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
    fn gspat_option_default() {
        let option = GSPATOption::default();
        assert_eq!(option.repeat, NonZeroUsize::new(100).unwrap());
        assert_eq!(
            option.constraint,
            EmissionConstraint::Clamp(Intensity::MIN, Intensity::MAX)
        );
    }

    #[test]
    fn test_gspat_all() {
        let geometry = create_geometry(1, 1);

        let g = GSPAT::new(
            vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            GSPATOption {
                repeat: NonZeroUsize::new(5).unwrap(),
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
    fn test_gspat_filtered() {
        let geometry = create_geometry(2, 1);

        let g = GSPAT {
            foci: vec![(Point3::origin(), 1. * Pa), (Point3::origin(), 1. * Pa)],
            option: GSPATOption {
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
