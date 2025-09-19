use std::mem::{ManuallyDrop, MaybeUninit};

use autd3_core::{
    acoustics::{directivity::Directivity, propagate},
    environment::Environment,
    gain::TransducerMask,
    geometry::{Complex, Geometry, Point3},
};
use nalgebra::{ComplexField, Dyn, Normed, U1, VecStorage};

use crate::{LinAlgBackend, MatrixX, MatrixXc, VectorX, VectorXc, error::HoloError};

/// [`LinAlgBackend`] using [`nalgebra`].
///
/// [`nalgebra`]: https://docs.rs/nalgebra/latest/nalgebra/
#[derive(Debug, Clone, Copy, Default)]
pub struct NalgebraBackend;

impl NalgebraBackend {
    /// Create a new [`NalgebraBackend`].
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

macro_rules! par_map {
    ($iter:expr, $f:expr) => {
        if cfg!(miri) {
            $iter.iter().map($f).collect::<Vec<_>>()
        } else {
            $iter.par_iter().map($f).collect::<Vec<_>>()
        }
    };
}

macro_rules! par_for_each {
    ($iter:expr, $f:expr) => {
        if cfg!(miri) {
            $iter.for_each($f)
        } else {
            $iter.par_bridge().for_each($f)
        }
    };
}

struct Ptr(*mut Complex);
impl Ptr {
    #[inline]
    fn write(&mut self, value: Complex) {
        unsafe {
            *self.0 = value;
            self.0 = self.0.add(1);
        }
    }

    #[inline]
    fn add(&self, i: usize) -> Self {
        Self(unsafe { self.0.add(i) })
    }
}
unsafe impl Send for Ptr {}
unsafe impl Sync for Ptr {}

fn uninit_mat(row: usize, col: usize) -> MatrixXc {
    MatrixXc::from_data(unsafe {
        let mut data = Vec::<MaybeUninit<Complex>>::new();
        let length = row * col;
        data.reserve_exact(length);
        data.resize_with(length, MaybeUninit::uninit);
        let uninit = VecStorage::new(Dyn(row), Dyn(col), data);
        let vec: Vec<_> = uninit.into();
        let mut md = ManuallyDrop::new(vec);
        let new_data = Vec::from_raw_parts(md.as_mut_ptr() as *mut _, md.len(), md.capacity());
        VecStorage::new(Dyn(row), Dyn(col), new_data)
    })
}

impl LinAlgBackend for NalgebraBackend {
    type MatrixXc = MatrixXc;
    type MatrixX = MatrixX;
    type VectorXc = VectorXc;
    type VectorX = VectorX;

    fn alloc_v(&self, size: usize) -> Result<Self::VectorX, HoloError> {
        Ok(Self::VectorX::zeros(size))
    }

    fn alloc_zeros_cv(&self, size: usize) -> Result<Self::VectorXc, HoloError> {
        Ok(Self::VectorXc::zeros(size))
    }

    fn alloc_zeros_cm(&self, rows: usize, cols: usize) -> Result<Self::MatrixXc, HoloError> {
        Ok(Self::MatrixXc::zeros(rows, cols))
    }

    fn cv_from_slice(&self, real: &[f32]) -> Result<Self::VectorXc, HoloError> {
        Ok(Self::VectorXc::from_iterator(
            real.len(),
            real.iter().map(|&r| Complex::new(r, 0.)),
        ))
    }

    fn clone_cv(&self, v: &Self::VectorXc) -> Result<Self::VectorXc, HoloError> {
        Ok(v.clone())
    }

    fn to_host_cv(&self, v: Self::VectorXc) -> Result<VectorXc, HoloError> {
        Ok(v)
    }

    fn cols_c(&self, m: &Self::MatrixXc) -> Result<usize, HoloError> {
        Ok(m.ncols())
    }

    fn norm_squared_cv(&self, a: &Self::VectorXc, b: &mut Self::VectorX) -> Result<(), HoloError> {
        *b = a.map(|v| v.norm_squared());
        Ok(())
    }

    fn max_v(&self, m: &Self::VectorX) -> Result<f32, HoloError> {
        Ok(m.max())
    }

    fn gemv_c(
        &self,
        trans: crate::Trans,
        alpha: Complex,
        a: &Self::MatrixXc,
        x: &Self::VectorXc,
        beta: Complex,
        y: &mut Self::VectorXc,
    ) -> Result<(), HoloError> {
        match trans {
            crate::Trans::NoTrans => y.gemv(alpha, a, x, beta),
            crate::Trans::Trans => y.gemv_tr(alpha, a, x, beta),
            crate::Trans::ConjTrans => y.gemv_ad(alpha, a, x, beta),
        }
        Ok(())
    }

    fn gemm_c(
        &self,
        trans_a: crate::Trans,
        trans_b: crate::Trans,
        alpha: Complex,
        a: &Self::MatrixXc,
        b: &Self::MatrixXc,
        beta: Complex,
        y: &mut Self::MatrixXc,
    ) -> Result<(), HoloError> {
        match trans_a {
            crate::Trans::NoTrans => match trans_b {
                crate::Trans::NoTrans => y.gemm(alpha, a, b, beta),
                crate::Trans::Trans => y.gemm(alpha, a, &b.transpose(), beta),
                crate::Trans::ConjTrans => y.gemm(alpha, a, &b.adjoint(), beta),
            },
            crate::Trans::Trans => match trans_b {
                crate::Trans::NoTrans => y.gemm_tr(alpha, a, b, beta),
                crate::Trans::Trans => y.gemm_tr(alpha, a, &b.transpose(), beta),
                crate::Trans::ConjTrans => y.gemm_tr(alpha, a, &b.adjoint(), beta),
            },
            crate::Trans::ConjTrans => match trans_b {
                crate::Trans::NoTrans => y.gemm_ad(alpha, a, b, beta),
                crate::Trans::Trans => y.gemm_ad(alpha, a, &b.transpose(), beta),
                crate::Trans::ConjTrans => y.gemm_ad(alpha, a, &b.adjoint(), beta),
            },
        }
        Ok(())
    }

    fn scaled_to_cv(
        &self,
        a: &Self::VectorXc,
        b: &Self::VectorXc,
        c: &mut Self::VectorXc,
    ) -> Result<(), HoloError> {
        *c = a.zip_map(b, |a, b| a / a.abs() * b);
        Ok(())
    }

    fn scaled_to_assign_cv(
        &self,
        a: &Self::VectorXc,
        b: &mut Self::VectorXc,
    ) -> Result<(), HoloError> {
        b.zip_apply(a, |b, a| *b = *b / b.abs() * a);
        Ok(())
    }

    fn generate_propagation_matrix<D: Directivity>(
        &self,
        geometry: &Geometry,
        env: &Environment,
        foci: &[Point3],
        filter: &TransducerMask,
    ) -> Result<Self::MatrixXc, HoloError> {
        use rayon::prelude::*;

        let num_transducers = [0]
            .into_iter()
            .chain(geometry.iter().scan(0, |state, dev| {
                *state += filter.num_enabled_transducers(dev);
                Some(*state)
            }))
            .collect::<Vec<_>>();
        let n = num_transducers.last().copied().unwrap();

        let num_devices = filter.num_enabled_devices(geometry);
        let m = foci.len();
        let do_parallel_in_col = num_devices < m;

        if filter.is_all_enabled() {
            if do_parallel_in_col {
                let columns = par_map!(foci, |f| {
                    nalgebra::Matrix::<Complex, U1, Dyn, VecStorage<Complex, U1, Dyn>>::from_iterator(
                    n,
                    geometry.iter().flat_map(|dev| {
                        dev.iter().map(move |tr| {
                            propagate::<D>(tr, env.wavenumber(), dev.axial_direction(), *f)
                        })
                    }),
                )
                });
                Ok(MatrixXc::from_rows(&columns))
            } else {
                let mut r = uninit_mat(foci.len(), n);
                let ptr = Ptr(r.as_mut_ptr());
                par_for_each!(geometry.iter(), move |dev| {
                    let mut ptr = ptr.add(foci.len() * num_transducers[dev.idx()]);
                    dev.iter().for_each(move |tr| {
                        foci.iter().for_each(|f| {
                            ptr.write(propagate::<D>(
                                tr,
                                env.wavenumber(),
                                dev.axial_direction(),
                                *f,
                            ));
                        });
                    });
                });
                Ok(r)
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if do_parallel_in_col {
                let columns = par_map!(foci, |f| {
                    nalgebra::Matrix::<Complex, U1, Dyn, VecStorage<Complex, U1, Dyn>>::from_iterator(
                        n,
                        geometry.iter().filter(|dev| filter.has_enabled(dev)).flat_map(|dev| {
                            dev.iter().filter(|tr| filter.is_enabled(tr)).map(move |tr| {
                                propagate::<D>(tr, env.wavenumber(), dev.axial_direction(), *f)
                            })
                        }),
                    )
                });
                Ok(MatrixXc::from_rows(&columns))
            } else {
                let mut r = uninit_mat(foci.len(), n);
                let ptr = Ptr(r.as_mut_ptr());
                par_for_each!(
                    geometry.iter().filter(|dev| filter.has_enabled(dev)),
                    move |dev| {
                        let mut ptr = ptr.add(foci.len() * num_transducers[dev.idx()]);
                        dev.iter().for_each(move |tr| {
                            if filter.is_enabled(tr) {
                                foci.iter().for_each(|f| {
                                    ptr.write(propagate::<D>(
                                        tr,
                                        env.wavenumber(),
                                        dev.axial_direction(),
                                        *f,
                                    ));
                                });
                            }
                        });
                    }
                );
                Ok(r)
            }
        }
    }

    fn gen_back_prop(
        &self,
        m: usize,
        n: usize,
        transfer: &Self::MatrixXc,
    ) -> Result<Self::MatrixXc, HoloError> {
        Ok(MatrixXc::from_vec(
            m,
            n,
            (0..n)
                .flat_map(|i| {
                    let x = 1.0
                        / transfer
                            .rows(i, 1)
                            .iter()
                            .map(|x| x.norm_sqr())
                            .sum::<f32>();
                    (0..m).map(move |j| transfer[(i, j)].conj() * x)
                })
                .collect::<Vec<_>>(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use crate::{Amplitude, Pa, Trans, tests::create_geometry};

    use super::*;

    use autd3_core::{
        acoustics::directivity::Sphere, gain::DeviceTransducerMask, geometry::Transducer,
    };
    use rand::Rng;

    const N: usize = 10;
    const EPS: f32 = 1e-3;

    fn gen_foci(n: usize) -> impl Iterator<Item = (Point3, Amplitude)> {
        (0..n).map(move |i| {
            (
                Point3::new(
                    90. + 10. * (2.0 * PI * i as f32 / n as f32).cos(),
                    70. + 10. * (2.0 * PI * i as f32 / n as f32).sin(),
                    150.,
                ),
                10e3 * Pa,
            )
        })
    }

    fn make_random_v(size: usize) -> VectorX {
        let mut rng = rand::rng();
        let v: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(size)
            .collect();
        VectorX::from_iterator(size, v)
    }

    fn make_random_cv(size: usize) -> VectorXc {
        let mut rng = rand::rng();
        let real: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(size)
            .collect();
        let imag: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(size)
            .collect();
        VectorXc::from_iterator(
            real.len(),
            real.into_iter().zip(imag).map(|(r, i)| Complex::new(r, i)),
        )
    }

    fn make_random_cm(rows: usize, cols: usize) -> MatrixXc {
        let mut rng = rand::rng();
        let real: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(rows * cols)
            .collect();
        let imag: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(rows * cols)
            .collect();
        MatrixXc::from_iterator(
            rows,
            cols,
            real.into_iter().zip(imag).map(|(r, i)| Complex::new(r, i)),
        )
    }

    #[rstest::fixture]
    fn backend() -> NalgebraBackend {
        NalgebraBackend::new()
    }

    #[rstest::rstest]
    fn test_alloc_v(backend: NalgebraBackend) -> Result<(), HoloError> {
        let v = backend.alloc_v(N)?;

        assert_eq!(N, v.len());
        Ok(())
    }

    #[rstest::rstest]
    fn test_alloc_zeros_cv(backend: NalgebraBackend) -> Result<(), HoloError> {
        let v = backend.alloc_zeros_cv(N)?;

        assert_eq!(N, v.len());
        assert!(v.iter().all(|&v| v == Complex::new(0., 0.)));
        Ok(())
    }

    #[rstest::rstest]
    fn test_alloc_zeros_cm(backend: NalgebraBackend) -> Result<(), HoloError> {
        let m = backend.alloc_zeros_cm(N, 2 * N)?;

        assert_eq!(N, m.nrows());
        assert_eq!(2 * N, m.ncols());
        assert!(m.iter().all(|&v| v == Complex::new(0., 0.)));
        Ok(())
    }

    #[rstest::rstest]
    fn test_cv_from_slice(backend: NalgebraBackend) -> Result<(), HoloError> {
        let rng = rand::rng();

        let real: Vec<f32> = rng
            .sample_iter(rand::distr::StandardUniform)
            .take(N)
            .collect();

        let c = backend.cv_from_slice(&real)?;

        assert_eq!(N, c.len());
        real.iter().zip(c.iter()).for_each(|(r, c)| {
            assert_eq!(r, &c.re);
            assert_eq!(0.0, c.im);
        });
        Ok(())
    }

    #[rstest::rstest]
    fn test_clone_cv(backend: NalgebraBackend) -> Result<(), HoloError> {
        let c = make_random_cv(N);
        let c2 = backend.clone_cv(&c)?;

        c.iter().zip(c2.iter()).for_each(|(c, c2)| {
            assert_eq!(c.re, c2.re);
            assert_eq!(c.im, c2.im);
        });
        Ok(())
    }

    #[rstest::rstest]
    fn test_cols_c(backend: NalgebraBackend) -> Result<(), HoloError> {
        let m = MatrixXc::zeros(N, 2 * N);

        assert_eq!(2 * N, backend.cols_c(&m)?);

        Ok(())
    }

    #[rstest::rstest]
    fn test_norm_squared_cv(backend: NalgebraBackend) -> Result<(), HoloError> {
        let v = make_random_cv(N);

        let mut abs = backend.alloc_v(N)?;
        backend.norm_squared_cv(&v, &mut abs)?;

        v.iter().zip(abs.iter()).for_each(|(v, abs)| {
            approx::assert_abs_diff_eq!(v.norm_squared(), abs, epsilon = EPS);
        });
        Ok(())
    }

    #[rstest::rstest]
    fn test_max_v(backend: NalgebraBackend) -> Result<(), HoloError> {
        let v = make_random_v(N);

        let max = backend.max_v(&v)?;

        assert_eq!(
            *v.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max
        );
        Ok(())
    }

    #[rstest::rstest]
    fn test_gemv_c(backend: NalgebraBackend) -> Result<(), HoloError> {
        let m = N;
        let n = 2 * N;

        let mut rng = rand::rng();

        {
            let a = make_random_cm(m, n);
            let b = make_random_cv(n);
            let mut c = make_random_cv(m);
            let cc = backend.clone_cv(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemv_c(Trans::NoTrans, alpha, &a, &b, beta, &mut c)?;

            let expected = a * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(n, m);
            let b = make_random_cv(n);
            let mut c = make_random_cv(m);
            let cc = backend.clone_cv(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemv_c(Trans::Trans, alpha, &a, &b, beta, &mut c)?;

            let expected = a.transpose() * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(n, m);
            let b = make_random_cv(n);
            let mut c = make_random_cv(m);
            let cc = backend.clone_cv(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemv_c(Trans::ConjTrans, alpha, &a, &b, beta, &mut c)?;

            let expected = a.adjoint() * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }
        Ok(())
    }

    #[rstest::rstest]
    fn test_gemm_c(backend: NalgebraBackend) -> Result<(), HoloError> {
        let m = N;
        let n = 2 * N;
        let k = 3 * N;

        let mut rng = rand::rng();

        {
            let a = make_random_cm(m, k);
            let b = make_random_cm(k, n);
            let mut c = make_random_cm(m, n);
            let cc = c.clone();

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::NoTrans, Trans::NoTrans, alpha, &a, &b, beta, &mut c)?;

            let expected = a * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(m, k);
            let b = make_random_cm(n, k);
            let mut c = make_random_cm(m, n);
            let cc = c.clone();

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::NoTrans, Trans::Trans, alpha, &a, &b, beta, &mut c)?;

            let expected = a * b.transpose() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(m, k);
            let b = make_random_cm(n, k);
            let mut c = make_random_cm(m, n);
            let cc = c.clone();

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(
                Trans::NoTrans,
                Trans::ConjTrans,
                alpha,
                &a,
                &b,
                beta,
                &mut c,
            )?;

            let expected = a * b.adjoint() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(k, m);
            let b = make_random_cm(k, n);
            let mut c = make_random_cm(m, n);
            let cc = c.clone();

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::Trans, Trans::NoTrans, alpha, &a, &b, beta, &mut c)?;

            let expected = a.transpose() * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(k, m);
            let b = make_random_cm(n, k);
            let mut c = make_random_cm(m, n);
            let cc = c.clone();

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::Trans, Trans::Trans, alpha, &a, &b, beta, &mut c)?;

            let expected = a.transpose() * b.transpose() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(k, m);
            let b = make_random_cm(n, k);
            let mut c = make_random_cm(m, n);
            let cc = c.clone();

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::Trans, Trans::ConjTrans, alpha, &a, &b, beta, &mut c)?;

            let expected = a.transpose() * b.adjoint() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(k, m);
            let b = make_random_cm(k, n);
            let mut c = make_random_cm(m, n);
            let cc = c.clone();

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(
                Trans::ConjTrans,
                Trans::NoTrans,
                alpha,
                &a,
                &b,
                beta,
                &mut c,
            )?;

            let expected = a.adjoint() * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(k, m);
            let b = make_random_cm(n, k);
            let mut c = make_random_cm(m, n);
            let cc = c.clone();

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::ConjTrans, Trans::Trans, alpha, &a, &b, beta, &mut c)?;

            let expected = a.adjoint() * b.transpose() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(k, m);
            let b = make_random_cm(n, k);
            let mut c = make_random_cm(m, n);
            let cc = c.clone();

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(
                Trans::ConjTrans,
                Trans::ConjTrans,
                alpha,
                &a,
                &b,
                beta,
                &mut c,
            )?;

            let expected = a.adjoint() * b.adjoint() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }
        Ok(())
    }

    #[rstest::rstest]
    fn test_scaled_to_cv(backend: NalgebraBackend) -> Result<(), HoloError> {
        let a = make_random_cv(N);
        let b = make_random_cv(N);
        let mut c = VectorXc::zeros(N);

        backend.scaled_to_cv(&a, &b, &mut c)?;

        c.iter()
            .zip(a.iter())
            .zip(b.iter())
            .for_each(|((&c, &a), &b)| {
                let e = a / a.abs() * b;
                approx::assert_abs_diff_eq!(e.re, c.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(e.im, c.im, epsilon = EPS);
            });

        Ok(())
    }

    #[rstest::rstest]
    fn test_scaled_to_assign_cv(backend: NalgebraBackend) -> Result<(), HoloError> {
        let a = make_random_cv(N);
        let mut b = make_random_cv(N);
        let bc = backend.clone_cv(&b)?;

        backend.scaled_to_assign_cv(&a, &mut b)?;

        b.iter()
            .zip(a.iter())
            .zip(bc.iter())
            .for_each(|((&b, &a), &bc)| {
                let e = bc / bc.abs() * a;
                approx::assert_abs_diff_eq!(e.re, b.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(e.im, b.im, epsilon = EPS);
            });

        Ok(())
    }

    #[rstest::rstest]
    #[case(1, 2)]
    #[case(2, 1)]
    fn test_generate_propagation_matrix_unsafe(
        #[case] dev_num: usize,
        #[case] foci_num: usize,
        backend: NalgebraBackend,
    ) -> Result<(), HoloError> {
        let env = Environment::new();

        let reference = |geometry: Geometry, foci: Vec<Point3>| {
            let mut g = MatrixXc::zeros(
                foci.len(),
                geometry
                    .iter()
                    .map(|dev| dev.num_transducers())
                    .sum::<usize>(),
            );
            let transducers = geometry
                .iter()
                .flat_map(|dev| dev.iter().map(|tr| (dev.idx(), tr)))
                .collect::<Vec<_>>();
            (0..foci.len()).for_each(|i| {
                (0..transducers.len()).for_each(|j| {
                    g[(i, j)] = propagate::<Sphere>(
                        transducers[j].1,
                        env.wavenumber(),
                        geometry[transducers[j].0].axial_direction(),
                        foci[i],
                    )
                })
            });
            g
        };

        let geometry = create_geometry(dev_num, dev_num);
        let foci = gen_foci(foci_num).map(|(p, _)| p).collect::<Vec<_>>();

        let g = backend.generate_propagation_matrix::<Sphere>(
            &geometry,
            &env,
            &foci,
            &TransducerMask::AllEnabled,
        )?;
        reference(geometry, foci)
            .iter()
            .zip(g.iter())
            .for_each(|(r, g)| {
                approx::assert_abs_diff_eq!(r.re, g.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(r.im, g.im, epsilon = EPS);
            });

        Ok(())
    }

    #[rstest::rstest]
    #[case(3, 1)]
    #[case(3, 3)]
    fn test_generate_propagation_matrix_with_disabled_device(
        #[case] dev_num: usize,
        #[case] foci_num: usize,
        backend: NalgebraBackend,
    ) -> Result<(), HoloError> {
        let env = Environment::new();

        let reference = |geometry: Geometry, foci: Vec<Point3>, filter: TransducerMask| {
            let mut g = MatrixXc::zeros(
                foci.len(),
                geometry
                    .iter()
                    .map(|dev| filter.num_enabled_transducers(dev))
                    .sum::<usize>(),
            );
            let transducers = geometry
                .iter()
                .filter(|dev| filter.has_enabled(dev))
                .flat_map(|dev| dev.iter().map(|tr| (dev.idx(), tr)))
                .collect::<Vec<_>>();
            (0..foci.len()).for_each(|i| {
                (0..transducers.len()).for_each(|j| {
                    g[(i, j)] = propagate::<Sphere>(
                        transducers[j].1,
                        env.wavenumber(),
                        geometry[transducers[j].0].axial_direction(),
                        foci[i],
                    )
                })
            });
            g
        };

        let geometry = create_geometry(dev_num, dev_num);
        let filter = TransducerMask::new(
            [DeviceTransducerMask::AllDisabled]
                .into_iter()
                .chain(std::iter::repeat(DeviceTransducerMask::AllEnabled))
                .take(geometry.num_devices()),
        );

        let foci = gen_foci(foci_num).map(|(p, _)| p).collect::<Vec<_>>();

        let g = backend.generate_propagation_matrix::<Sphere>(&geometry, &env, &foci, &filter)?;
        reference(geometry, foci, filter)
            .iter()
            .zip(g.iter())
            .for_each(|(r, g)| {
                approx::assert_abs_diff_eq!(r.re, g.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(r.im, g.im, epsilon = EPS);
            });

        Ok(())
    }

    #[rstest::rstest]
    #[case(1, 2)]
    #[case(2, 1)]
    fn test_generate_propagation_matrix_with_filter(
        #[case] dev_num: usize,
        #[case] foci_num: usize,
        backend: NalgebraBackend,
    ) -> Result<(), HoloError> {
        let env = Environment::new();

        let filter = |geometry: &Geometry| -> TransducerMask {
            TransducerMask::from_fn(geometry, |dev| {
                let num_transducers = dev.num_transducers();
                DeviceTransducerMask::from_fn(dev, |tr: &Transducer| tr.idx() > num_transducers / 2)
            })
        };

        let reference = |geometry, foci: Vec<Point3>| {
            let filter = filter(&geometry);
            let transducers = geometry
                .iter()
                .flat_map(|dev| {
                    dev.iter().filter_map(|tr| {
                        if filter.is_enabled(tr) {
                            Some((dev.idx(), tr))
                        } else {
                            None
                        }
                    })
                })
                .collect::<Vec<_>>();

            let mut g = MatrixXc::zeros(foci.len(), transducers.len());
            (0..foci.len()).for_each(|i| {
                (0..transducers.len()).for_each(|j| {
                    g[(i, j)] = propagate::<Sphere>(
                        transducers[j].1,
                        env.wavenumber(),
                        geometry[transducers[j].0].axial_direction(),
                        foci[i],
                    )
                })
            });
            g
        };

        let geometry = create_geometry(dev_num, dev_num);
        let foci = gen_foci(foci_num).map(|(p, _)| p).collect::<Vec<_>>();
        let filter = filter(&geometry);

        let g = backend.generate_propagation_matrix::<Sphere>(&geometry, &env, &foci, &filter)?;
        assert_eq!(g.nrows(), foci.len());
        assert_eq!(
            g.ncols(),
            geometry
                .iter()
                .map(|dev| dev.num_transducers() / 2)
                .sum::<usize>()
        );
        reference(geometry, foci)
            .iter()
            .zip(g.iter())
            .for_each(|(r, g)| {
                approx::assert_abs_diff_eq!(r.re, g.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(r.im, g.im, epsilon = EPS);
            });

        Ok(())
    }

    #[rstest::rstest]
    #[case(3, 1)]
    #[case(3, 3)]
    fn test_generate_propagation_matrix_with_filter_with_disabled_devices(
        #[case] dev_num: usize,
        #[case] foci_num: usize,
        backend: NalgebraBackend,
    ) -> Result<(), HoloError> {
        let env = Environment::new();

        let filter = |geometry: &Geometry| {
            TransducerMask::from_fn(geometry, |dev| {
                if dev.idx() == 0 {
                    return DeviceTransducerMask::AllDisabled;
                }
                let num_transducers = dev.num_transducers();
                DeviceTransducerMask::from_fn(dev, |tr: &Transducer| tr.idx() > num_transducers / 2)
            })
        };

        let reference = |geometry: Geometry, foci: Vec<Point3>, filter: TransducerMask| {
            let mut g = MatrixXc::zeros(
                foci.len(),
                geometry
                    .iter()
                    .map(|dev| filter.num_enabled_transducers(dev))
                    .sum::<usize>(),
            );
            let transducers = geometry
                .iter()
                .filter(|dev| filter.has_enabled(dev))
                .flat_map(|dev| {
                    dev.iter().filter_map(|tr| {
                        if filter.is_enabled(tr) {
                            Some((dev.idx(), tr))
                        } else {
                            None
                        }
                    })
                })
                .collect::<Vec<_>>();
            (0..foci.len()).for_each(|i| {
                (0..transducers.len()).for_each(|j| {
                    g[(i, j)] = propagate::<Sphere>(
                        transducers[j].1,
                        env.wavenumber(),
                        geometry[transducers[j].0].axial_direction(),
                        foci[i],
                    )
                })
            });
            g
        };

        let geometry = create_geometry(dev_num, dev_num);
        let foci = gen_foci(foci_num).map(|(p, _)| p).collect::<Vec<_>>();
        let filter = filter(&geometry);

        let g = backend.generate_propagation_matrix::<Sphere>(&geometry, &env, &foci, &filter)?;
        assert_eq!(g.nrows(), foci.len());
        assert_eq!(
            g.ncols(),
            geometry
                .iter()
                .filter(|dev| filter.has_enabled(dev))
                .map(|dev| dev.num_transducers() / 2)
                .sum::<usize>()
        );
        reference(geometry, foci, filter)
            .iter()
            .zip(g.iter())
            .for_each(|(r, g)| {
                approx::assert_abs_diff_eq!(r.re, g.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(r.im, g.im, epsilon = EPS);
            });

        Ok(())
    }

    #[rstest::rstest]
    fn test_gen_back_prop(backend: NalgebraBackend) -> Result<(), HoloError> {
        let env = Environment::new();

        let geometry = create_geometry(1, 1);
        let foci = gen_foci(1).map(|(p, _)| p).collect::<Vec<_>>();

        let m = geometry
            .iter()
            .map(|dev| dev.num_transducers())
            .sum::<usize>();
        let n = foci.len();

        let g = backend.generate_propagation_matrix::<Sphere>(
            &geometry,
            &env,
            &foci,
            &TransducerMask::AllEnabled,
        )?;

        let b = backend.gen_back_prop(m, n, &g)?;
        let reference = {
            let mut b = MatrixXc::zeros(m, n);
            (0..n).for_each(|i| {
                let x = 1.0 / g.rows(i, 1).iter().map(|x| x.norm_sqr()).sum::<f32>();
                (0..m).for_each(|j| {
                    b[(j, i)] = g[(i, j)].conj() * x;
                })
            });
            b
        };

        reference.iter().zip(b.iter()).for_each(|(r, b)| {
            approx::assert_abs_diff_eq!(r.re, b.re, epsilon = EPS);
            approx::assert_abs_diff_eq!(r.im, b.im, epsilon = EPS);
        });
        Ok(())
    }
}
