use std::mem::{ManuallyDrop, MaybeUninit};

use autd3_core::{
    acoustics::{
        directivity::{Directivity, Sphere},
        propagate,
    },
    gain::TransducerFilter,
    geometry::{Complex, Geometry, Point3},
};
use nalgebra::{ComplexField, Dyn, Normed, U1, VecStorage};

use crate::{LinAlgBackend, MatrixX, MatrixXc, VectorX, VectorXc, error::HoloError};

/// [`LinAlgBackend`] using [`nalgebra`].
///
/// [`nalgebra`]: https://docs.rs/nalgebra/latest/nalgebra/
pub struct NalgebraBackend<D: Directivity> {
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity> NalgebraBackend<D> {
    /// Create a new [`NalgebraBackend`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Default for NalgebraBackend<Sphere> {
    fn default() -> Self {
        Self::new()
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

impl<D: Directivity> LinAlgBackend<D> for NalgebraBackend<D> {
    type MatrixXc = MatrixXc;
    type MatrixX = MatrixX;
    type VectorXc = VectorXc;
    type VectorX = VectorX;

    fn generate_propagation_matrix(
        &self,
        geometry: &Geometry,
        foci: &[Point3],
        filter: &TransducerFilter,
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
                            propagate::<D>(tr, dev.wavenumber(), dev.axial_direction(), f)
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
                                dev.wavenumber(),
                                dev.axial_direction(),
                                f,
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
                        geometry.iter().filter(|dev| filter.is_enabled_device(dev)).flat_map(|dev| {
                            dev.iter().filter(|tr| filter.is_enabled(tr)).map(move |tr| {
                                propagate::<D>(tr, dev.wavenumber(), dev.axial_direction(), f)
                            })
                        }),
                    )
                });
                Ok(MatrixXc::from_rows(&columns))
            } else {
                let mut r = uninit_mat(foci.len(), n);
                let ptr = Ptr(r.as_mut_ptr());
                par_for_each!(
                    geometry.iter().filter(|dev| filter.is_enabled_device(dev)),
                    move |dev| {
                        let mut ptr = ptr.add(foci.len() * num_transducers[dev.idx()]);
                        dev.iter().for_each(move |tr| {
                            if filter.is_enabled(tr) {
                                foci.iter().for_each(|f| {
                                    ptr.write(propagate::<D>(
                                        tr,
                                        dev.wavenumber(),
                                        dev.axial_direction(),
                                        f,
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

    fn to_host_cv(&self, v: Self::VectorXc) -> Result<VectorXc, HoloError> {
        Ok(v)
    }

    fn to_host_v(&self, v: Self::VectorX) -> Result<VectorX, HoloError> {
        Ok(v)
    }

    fn to_host_cm(&self, v: Self::MatrixXc) -> Result<MatrixXc, HoloError> {
        Ok(v)
    }

    fn alloc_v(&self, size: usize) -> Result<Self::VectorX, HoloError> {
        Ok(Self::VectorX::zeros(size))
    }

    fn alloc_zeros_v(&self, size: usize) -> Result<Self::VectorX, HoloError> {
        Ok(Self::VectorX::zeros(size))
    }

    fn alloc_zeros_cv(&self, size: usize) -> Result<Self::VectorXc, HoloError> {
        Ok(Self::VectorXc::zeros(size))
    }

    fn from_slice_v(&self, v: &[f32]) -> Result<Self::VectorX, HoloError> {
        Ok(Self::VectorX::from_row_slice(v))
    }

    fn from_slice_m(
        &self,
        rows: usize,
        cols: usize,
        v: &[f32],
    ) -> Result<Self::MatrixX, HoloError> {
        Ok(Self::MatrixX::from_iterator(rows, cols, v.iter().copied()))
    }

    fn make_complex2_v(
        &self,
        real: &Self::VectorX,
        imag: &Self::VectorX,
        v: &mut Self::VectorXc,
    ) -> Result<(), HoloError> {
        *v = Self::VectorXc::from_iterator(
            real.len(),
            real.iter()
                .zip(imag.iter())
                .map(|(&r, &i)| Complex::new(r, i)),
        );
        Ok(())
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

    fn alloc_cv(&self, size: usize) -> Result<Self::VectorXc, HoloError> {
        Ok(Self::VectorXc::zeros(size))
    }

    fn alloc_zeros_cm(&self, rows: usize, cols: usize) -> Result<Self::MatrixXc, HoloError> {
        Ok(Self::MatrixXc::zeros(rows, cols))
    }

    fn create_diagonal_c(
        &self,
        v: &Self::VectorXc,
        a: &mut Self::MatrixXc,
    ) -> Result<(), HoloError> {
        a.fill(Complex::new(0., 0.));
        a.set_diagonal(v);
        Ok(())
    }

    fn norm_squared_cv(&self, a: &Self::VectorXc, b: &mut Self::VectorX) -> Result<(), HoloError> {
        *b = a.map(|v| v.norm_squared());
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

    fn alloc_cm(&self, rows: usize, cols: usize) -> Result<Self::MatrixXc, HoloError> {
        Ok(Self::MatrixXc::zeros(rows, cols))
    }

    fn clone_v(&self, v: &Self::VectorX) -> Result<Self::VectorX, HoloError> {
        Ok(v.clone())
    }

    fn clone_m(&self, v: &Self::MatrixX) -> Result<Self::MatrixX, HoloError> {
        Ok(v.clone())
    }

    fn clone_cv(&self, v: &Self::VectorXc) -> Result<Self::VectorXc, HoloError> {
        Ok(v.clone())
    }

    fn clone_cm(&self, v: &Self::MatrixXc) -> Result<Self::MatrixXc, HoloError> {
        Ok(v.clone())
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

    fn from_slice_cv(&self, real: &[f32]) -> Result<Self::VectorXc, HoloError> {
        Ok(Self::VectorXc::from_iterator(
            real.len(),
            real.iter().map(|&r| Complex::new(r, 0.)),
        ))
    }

    fn from_slice2_cv(&self, r: &[f32], i: &[f32]) -> Result<Self::VectorXc, HoloError> {
        Ok(Self::VectorXc::from_iterator(
            r.len(),
            r.iter().zip(i.iter()).map(|(&r, &i)| Complex::new(r, i)),
        ))
    }

    fn from_slice2_cm(
        &self,
        rows: usize,
        cols: usize,
        r: &[f32],
        i: &[f32],
    ) -> Result<Self::MatrixXc, HoloError> {
        Ok(Self::MatrixXc::from_iterator(
            rows,
            cols,
            r.iter().zip(i.iter()).map(|(&r, &i)| Complex::new(r, i)),
        ))
    }

    fn scale_assign_cv(&self, a: Complex, b: &mut Self::VectorXc) -> Result<(), HoloError> {
        b.apply(|x| *x *= a);
        Ok(())
    }

    fn conj_assign_v(&self, b: &mut Self::VectorXc) -> Result<(), HoloError> {
        b.apply(|x| *x = x.conj());
        Ok(())
    }

    fn dot_c(&self, x: &Self::VectorXc, y: &Self::VectorXc) -> Result<Complex, HoloError> {
        Ok(x.dotc(y))
    }

    fn alloc_m(&self, rows: usize, cols: usize) -> Result<Self::MatrixX, HoloError> {
        Ok(Self::MatrixX::zeros(rows, cols))
    }

    fn to_host_m(&self, v: Self::MatrixX) -> Result<MatrixX, HoloError> {
        Ok(v)
    }

    fn copy_from_slice_v(&self, v: &[f32], dst: &mut Self::VectorX) -> Result<(), HoloError> {
        dst.view_mut((0, 0), (v.len(), 1)).copy_from_slice(v);
        Ok(())
    }

    fn copy_to_v(&self, src: &Self::VectorX, dst: &mut Self::VectorX) -> Result<(), HoloError> {
        dst.copy_from(src);
        Ok(())
    }

    fn copy_to_m(&self, src: &Self::MatrixX, dst: &mut Self::MatrixX) -> Result<(), HoloError> {
        dst.copy_from(src);
        Ok(())
    }

    fn create_diagonal(&self, v: &Self::VectorX, a: &mut Self::MatrixX) -> Result<(), HoloError> {
        a.fill(0.);
        a.set_diagonal(v);
        Ok(())
    }

    fn get_diagonal(&self, a: &Self::MatrixX, v: &mut Self::VectorX) -> Result<(), HoloError> {
        *v = a.diagonal();
        Ok(())
    }

    fn real_cm(&self, a: &Self::MatrixXc, b: &mut Self::MatrixX) -> Result<(), HoloError> {
        *b = a.map(|v| v.re);
        Ok(())
    }

    fn imag_cm(&self, a: &Self::MatrixXc, b: &mut Self::MatrixX) -> Result<(), HoloError> {
        *b = a.map(|v| v.im);
        Ok(())
    }

    fn exp_assign_cv(&self, v: &mut Self::VectorXc) -> Result<(), HoloError> {
        v.apply(|v| *v = v.exp());
        Ok(())
    }

    fn concat_col_cm(
        &self,
        a: &Self::MatrixXc,
        b: &Self::MatrixXc,
        c: &mut Self::MatrixXc,
    ) -> Result<(), HoloError> {
        c.view_mut((0, 0), (a.nrows(), a.ncols())).copy_from(a);
        c.view_mut((0, a.ncols()), (b.nrows(), b.ncols()))
            .copy_from(b);
        Ok(())
    }

    fn max_v(&self, m: &Self::VectorX) -> Result<f32, HoloError> {
        Ok(m.max())
    }

    fn hadamard_product_cm(
        &self,
        x: &Self::MatrixXc,
        y: &Self::MatrixXc,
        z: &mut Self::MatrixXc,
    ) -> Result<(), HoloError> {
        *z = x.component_mul(y);
        Ok(())
    }

    fn dot(&self, x: &Self::VectorX, y: &Self::VectorX) -> Result<f32, HoloError> {
        Ok(x.dot(y))
    }

    fn add_v(&self, alpha: f32, a: &Self::VectorX, b: &mut Self::VectorX) -> Result<(), HoloError> {
        *b += alpha * a;
        Ok(())
    }

    fn add_m(&self, alpha: f32, a: &Self::MatrixX, b: &mut Self::MatrixX) -> Result<(), HoloError> {
        *b += alpha * a;
        Ok(())
    }

    fn gevv_c(
        &self,
        trans_a: crate::Trans,
        trans_b: crate::Trans,
        alpha: Complex,
        a: &Self::VectorXc,
        b: &Self::VectorXc,
        beta: Complex,
        y: &mut Self::MatrixXc,
    ) -> Result<(), HoloError> {
        match trans_a {
            crate::Trans::NoTrans => match trans_b {
                crate::Trans::NoTrans => return Err(HoloError::InvalidOperation),
                crate::Trans::Trans => y.gemm(alpha, a, &b.transpose(), beta),
                crate::Trans::ConjTrans => y.gemm(alpha, a, &b.adjoint(), beta),
            },
            crate::Trans::Trans => match trans_b {
                crate::Trans::NoTrans => y.gemm_tr(alpha, a, b, beta),
                crate::Trans::Trans => return Err(HoloError::InvalidOperation),
                crate::Trans::ConjTrans => return Err(HoloError::InvalidOperation),
            },
            crate::Trans::ConjTrans => match trans_b {
                crate::Trans::NoTrans => y.gemm_ad(alpha, a, b, beta),
                crate::Trans::Trans => return Err(HoloError::InvalidOperation),
                crate::Trans::ConjTrans => return Err(HoloError::InvalidOperation),
            },
        }
        Ok(())
    }

    fn solve_inplace(&self, a: &Self::MatrixX, x: &mut Self::VectorX) -> Result<(), HoloError> {
        if !a.clone().lu().solve_mut(x) {
            return Err(HoloError::SolveFailed); // GRCOV_EXCL_LINE
        }
        Ok(())
    }

    fn reduce_col(&self, a: &Self::MatrixX, b: &mut Self::VectorX) -> Result<(), HoloError> {
        *b = a.column_sum();
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

    fn cols_c(&self, m: &Self::MatrixXc) -> Result<usize, HoloError> {
        Ok(m.ncols())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, f32::consts::PI};

    use crate::{Amplitude, Pa, Trans, tests::create_geometry};

    use super::*;

    use autd3_core::derive::Transducer;
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

    fn make_random_v(backend: &NalgebraBackend<Sphere>, size: usize) -> Result<VectorX, HoloError> {
        let mut rng = rand::rng();
        let v: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(size)
            .collect();
        backend.from_slice_v(&v)
    }

    fn make_random_m(
        backend: &NalgebraBackend<Sphere>,
        rows: usize,
        cols: usize,
    ) -> Result<MatrixX, HoloError> {
        let mut rng = rand::rng();
        let v: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(rows * cols)
            .collect();
        backend.from_slice_m(rows, cols, &v)
    }

    fn make_random_cv(
        backend: &NalgebraBackend<Sphere>,
        size: usize,
    ) -> Result<VectorXc, HoloError> {
        let mut rng = rand::rng();
        let real: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(size)
            .collect();
        let imag: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(size)
            .collect();
        backend.from_slice2_cv(&real, &imag)
    }

    fn make_random_cm(
        backend: &NalgebraBackend<Sphere>,
        rows: usize,
        cols: usize,
    ) -> Result<MatrixXc, HoloError> {
        let mut rng = rand::rng();
        let real: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(rows * cols)
            .collect();
        let imag: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(rows * cols)
            .collect();
        backend.from_slice2_cm(rows, cols, &real, &imag)
    }

    #[rstest::fixture]
    fn backend() -> NalgebraBackend<Sphere> {
        NalgebraBackend {
            _phantom: std::marker::PhantomData,
        }
    }

    #[rstest::rstest]
    #[test]
    fn test_alloc_v(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let v = backend.alloc_v(N)?;
        let v = backend.to_host_v(v)?;

        assert_eq!(N, v.len());
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_alloc_m(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let m = backend.alloc_m(N, 2 * N)?;
        let m = backend.to_host_m(m)?;

        assert_eq!(N, m.nrows());
        assert_eq!(2 * N, m.ncols());
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_alloc_cv(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let v = backend.alloc_cv(N)?;
        let v = backend.to_host_cv(v)?;

        assert_eq!(N, v.len());
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_alloc_cm(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let m = backend.alloc_cm(N, 2 * N)?;
        let m = backend.to_host_cm(m)?;

        assert_eq!(N, m.nrows());
        assert_eq!(2 * N, m.ncols());
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_alloc_zeros_v(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let v = backend.alloc_v(N)?;
        let v = backend.to_host_v(v)?;

        assert_eq!(N, v.len());
        assert!(v.iter().all(|&v| v == 0.));
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_alloc_zeros_cv(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let v = backend.alloc_cv(N)?;
        let v = backend.to_host_cv(v)?;

        assert_eq!(N, v.len());
        assert!(v.iter().all(|&v| v == Complex::new(0., 0.)));
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_alloc_zeros_cm(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let m = backend.alloc_cm(N, 2 * N)?;
        let m = backend.to_host_cm(m)?;

        assert_eq!(N, m.nrows());
        assert_eq!(2 * N, m.ncols());
        assert!(m.iter().all(|&v| v == Complex::new(0., 0.)));
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_cols_c(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let m = backend.alloc_cm(N, 2 * N)?;

        assert_eq!(2 * N, backend.cols_c(&m)?);

        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_from_slice_v(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let rng = rand::rng();

        let v: Vec<f32> = rng
            .sample_iter(rand::distr::StandardUniform)
            .take(N)
            .collect();

        let c = backend.from_slice_v(&v)?;
        let c = backend.to_host_v(c)?;

        assert_eq!(N, c.len());
        v.iter().zip(c.iter()).for_each(|(&r, &c)| {
            assert_eq!(r, c);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_from_slice_m(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let rng = rand::rng();

        let v: Vec<f32> = rng
            .sample_iter(rand::distr::StandardUniform)
            .take(N * 2 * N)
            .collect();

        let c = backend.from_slice_m(N, 2 * N, &v)?;
        let c = backend.to_host_m(c)?;

        assert_eq!(N, c.nrows());
        assert_eq!(2 * N, c.ncols());
        (0..2 * N).for_each(|col| {
            (0..N).for_each(|row| {
                assert_eq!(v[col * N + row], c[(row, col)]);
            })
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_from_slice_cv(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let rng = rand::rng();

        let real: Vec<f32> = rng
            .sample_iter(rand::distr::StandardUniform)
            .take(N)
            .collect();

        let c = backend.from_slice_cv(&real)?;
        let c = backend.to_host_cv(c)?;

        assert_eq!(N, c.len());
        real.iter().zip(c.iter()).for_each(|(r, c)| {
            assert_eq!(r, &c.re);
            assert_eq!(0.0, c.im);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_from_slice2_cv(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let mut rng = rand::rng();

        let real: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(N)
            .collect();
        let imag: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(N)
            .collect();

        let c = backend.from_slice2_cv(&real, &imag)?;
        let c = backend.to_host_cv(c)?;

        assert_eq!(N, c.len());
        real.iter()
            .zip(imag.iter())
            .zip(c.iter())
            .for_each(|((r, i), c)| {
                assert_eq!(r, &c.re);
                assert_eq!(i, &c.im);
            });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_from_slice2_cm(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let mut rng = rand::rng();

        let real: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(N * 2 * N)
            .collect();
        let imag: Vec<f32> = (&mut rng)
            .sample_iter(rand::distr::StandardUniform)
            .take(N * 2 * N)
            .collect();

        let c = backend.from_slice2_cm(N, 2 * N, &real, &imag)?;
        let c = backend.to_host_cm(c)?;

        assert_eq!(N, c.nrows());
        assert_eq!(2 * N, c.ncols());
        (0..2 * N).for_each(|col| {
            (0..N).for_each(|row| {
                assert_eq!(real[col * N + row], c[(row, col)].re);
                assert_eq!(imag[col * N + row], c[(row, col)].im);
            })
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_copy_from_slice_v(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        {
            let mut a = backend.alloc_zeros_v(N)?;
            let mut rng = rand::rng();
            let v = (&mut rng)
                .sample_iter(rand::distr::StandardUniform)
                .take(N / 2)
                .collect::<Vec<f32>>();

            backend.copy_from_slice_v(&v, &mut a)?;

            let a = backend.to_host_v(a)?;
            (0..N / 2).for_each(|i| {
                assert_eq!(v[i], a[i]);
            });
            (N / 2..N).for_each(|i| {
                assert_eq!(0., a[i]);
            });
        }

        {
            let mut a = backend.alloc_zeros_v(N)?;
            let v = [];

            backend.copy_from_slice_v(&v, &mut a)?;

            let a = backend.to_host_v(a)?;
            a.iter().for_each(|&a| {
                assert_eq!(0., a);
            });
        }

        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_copy_to_v(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let a = make_random_v(&backend, N)?;
        let mut b = backend.alloc_v(N)?;

        backend.copy_to_v(&a, &mut b)?;

        let a = backend.to_host_v(a)?;
        let b = backend.to_host_v(b)?;
        a.iter().zip(b.iter()).for_each(|(a, b)| {
            assert_eq!(a, b);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_copy_to_m(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let a = make_random_m(&backend, N, N)?;
        let mut b = backend.alloc_m(N, N)?;

        backend.copy_to_m(&a, &mut b)?;

        let a = backend.to_host_m(a)?;
        let b = backend.to_host_m(b)?;
        a.iter().zip(b.iter()).for_each(|(a, b)| {
            assert_eq!(a, b);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_clone_v(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let c = make_random_v(&backend, N)?;
        let c2 = backend.clone_v(&c)?;

        let c = backend.to_host_v(c)?;
        let c2 = backend.to_host_v(c2)?;

        c.iter().zip(c2.iter()).for_each(|(c, c2)| {
            assert_eq!(c, c2);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_clone_m(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let c = make_random_m(&backend, N, N)?;
        let c2 = backend.clone_m(&c)?;

        let c = backend.to_host_m(c)?;
        let c2 = backend.to_host_m(c2)?;

        c.iter().zip(c2.iter()).for_each(|(c, c2)| {
            assert_eq!(c, c2);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_clone_cv(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let c = make_random_cv(&backend, N)?;
        let c2 = backend.clone_cv(&c)?;

        let c = backend.to_host_cv(c)?;
        let c2 = backend.to_host_cv(c2)?;

        c.iter().zip(c2.iter()).for_each(|(c, c2)| {
            assert_eq!(c.re, c2.re);
            assert_eq!(c.im, c2.im);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_clone_cm(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let c = make_random_cm(&backend, N, N)?;
        let c2 = backend.clone_cm(&c)?;

        let c = backend.to_host_cm(c)?;
        let c2 = backend.to_host_cm(c2)?;

        c.iter().zip(c2.iter()).for_each(|(c, c2)| {
            assert_eq!(c.re, c2.re);
            assert_eq!(c.im, c2.im);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_make_complex2_v(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let real = make_random_v(&backend, N)?;
        let imag = make_random_v(&backend, N)?;

        let mut c = backend.alloc_cv(N)?;
        backend.make_complex2_v(&real, &imag, &mut c)?;

        let real = backend.to_host_v(real)?;
        let imag = backend.to_host_v(imag)?;
        let c = backend.to_host_cv(c)?;
        real.iter()
            .zip(imag.iter())
            .zip(c.iter())
            .for_each(|((r, i), c)| {
                assert_eq!(r, &c.re);
                assert_eq!(i, &c.im);
            });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_create_diagonal(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let diagonal = make_random_v(&backend, N)?;

        let mut c = backend.alloc_m(N, N)?;

        backend.create_diagonal(&diagonal, &mut c)?;

        let diagonal = backend.to_host_v(diagonal)?;
        let c = backend.to_host_m(c)?;
        (0..N).for_each(|i| {
            (0..N).for_each(|j| {
                if i == j {
                    assert_eq!(diagonal[i], c[(i, j)]);
                } else {
                    assert_eq!(0.0, c[(i, j)]);
                }
            })
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_create_diagonal_c(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let diagonal = make_random_cv(&backend, N)?;

        let mut c = backend.alloc_cm(N, N)?;

        backend.create_diagonal_c(&diagonal, &mut c)?;

        let diagonal = backend.to_host_cv(diagonal)?;
        let c = backend.to_host_cm(c)?;
        (0..N).for_each(|i| {
            (0..N).for_each(|j| {
                if i == j {
                    assert_eq!(diagonal[i].re, c[(i, j)].re);
                    assert_eq!(diagonal[i].im, c[(i, j)].im);
                } else {
                    assert_eq!(0.0, c[(i, j)].re);
                    assert_eq!(0.0, c[(i, j)].im);
                }
            })
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_get_diagonal(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let m = make_random_m(&backend, N, N)?;
        let mut diagonal = backend.alloc_v(N)?;

        backend.get_diagonal(&m, &mut diagonal)?;

        let m = backend.to_host_m(m)?;
        let diagonal = backend.to_host_v(diagonal)?;
        (0..N).for_each(|i| {
            assert_eq!(m[(i, i)], diagonal[i]);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_norm_squared_cv(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let v = make_random_cv(&backend, N)?;

        let mut abs = backend.alloc_v(N)?;
        backend.norm_squared_cv(&v, &mut abs)?;

        let v = backend.to_host_cv(v)?;
        let abs = backend.to_host_v(abs)?;
        v.iter().zip(abs.iter()).for_each(|(v, abs)| {
            approx::assert_abs_diff_eq!(v.norm_squared(), abs, epsilon = EPS);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_real_cm(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let v = make_random_cm(&backend, N, N)?;
        let mut r = backend.alloc_m(N, N)?;

        backend.real_cm(&v, &mut r)?;

        let v = backend.to_host_cm(v)?;
        let r = backend.to_host_m(r)?;
        (0..N).for_each(|i| {
            (0..N).for_each(|j| {
                approx::assert_abs_diff_eq!(v[(i, j)].re, r[(i, j)], epsilon = EPS);
            })
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_imag_cm(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let v = make_random_cm(&backend, N, N)?;
        let mut r = backend.alloc_m(N, N)?;

        backend.imag_cm(&v, &mut r)?;

        let v = backend.to_host_cm(v)?;
        let r = backend.to_host_m(r)?;
        (0..N).for_each(|i| {
            (0..N).for_each(|j| {
                approx::assert_abs_diff_eq!(v[(i, j)].im, r[(i, j)], epsilon = EPS);
            })
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_scale_assign_cv(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let mut v = make_random_cv(&backend, N)?;
        let vc = backend.clone_cv(&v)?;
        let mut rng = rand::rng();
        let scale = Complex::new(rng.random(), rng.random());

        backend.scale_assign_cv(scale, &mut v)?;

        let v = backend.to_host_cv(v)?;
        let vc = backend.to_host_cv(vc)?;
        v.iter().zip(vc.iter()).for_each(|(&v, &vc)| {
            let e = scale * vc;
            approx::assert_abs_diff_eq!(e.re, v.re, epsilon = EPS);
            approx::assert_abs_diff_eq!(e.im, v.im, epsilon = EPS);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_conj_assign_v(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let mut v = make_random_cv(&backend, N)?;
        let vc = backend.clone_cv(&v)?;

        backend.conj_assign_v(&mut v)?;

        let v = backend.to_host_cv(v)?;
        let vc = backend.to_host_cv(vc)?;
        v.iter().zip(vc.iter()).for_each(|(&v, &vc)| {
            assert_eq!(vc.re, v.re);
            assert_eq!(vc.im, -v.im);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_exp_assign_cv(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let mut v = make_random_cv(&backend, N)?;
        let vc = backend.clone_cv(&v)?;

        backend.exp_assign_cv(&mut v)?;

        let v = backend.to_host_cv(v)?;
        let vc = backend.to_host_cv(vc)?;
        v.iter().zip(vc.iter()).for_each(|(v, vc)| {
            approx::assert_abs_diff_eq!(vc.exp().re, v.re, epsilon = EPS);
            approx::assert_abs_diff_eq!(vc.exp().im, v.im, epsilon = EPS);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_concat_col_cm(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let a = make_random_cm(&backend, N, N)?;
        let b = make_random_cm(&backend, N, 2 * N)?;
        let mut c = backend.alloc_cm(N, N + 2 * N)?;

        backend.concat_col_cm(&a, &b, &mut c)?;

        let a = backend.to_host_cm(a)?;
        let b = backend.to_host_cm(b)?;
        let c = backend.to_host_cm(c)?;
        (0..N).for_each(|col| (0..N).for_each(|row| assert_eq!(a[(row, col)], c[(row, col)])));
        (0..2 * N)
            .for_each(|col| (0..N).for_each(|row| assert_eq!(b[(row, col)], c[(row, N + col)])));
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_max_v(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let v = make_random_v(&backend, N)?;

        let max = backend.max_v(&v)?;

        let v = backend.to_host_v(v)?;
        assert_eq!(
            *v.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max
        );
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_hadamard_product_cm(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let a = make_random_cm(&backend, N, N)?;
        let b = make_random_cm(&backend, N, N)?;
        let mut c = backend.alloc_cm(N, N)?;

        backend.hadamard_product_cm(&a, &b, &mut c)?;

        let a = backend.to_host_cm(a)?;
        let b = backend.to_host_cm(b)?;
        let c = backend.to_host_cm(c)?;
        c.iter()
            .zip(a.iter())
            .zip(b.iter())
            .for_each(|((c, a), b)| {
                approx::assert_abs_diff_eq!(a.re * b.re - a.im * b.im, c.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(a.re * b.im + a.im * b.re, c.im, epsilon = EPS);
            });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_dot(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let a = make_random_v(&backend, N)?;
        let b = make_random_v(&backend, N)?;

        let dot = backend.dot(&a, &b)?;

        let a = backend.to_host_v(a)?;
        let b = backend.to_host_v(b)?;
        let expect = a.iter().zip(b.iter()).map(|(a, b)| a * b).sum::<f32>();
        approx::assert_abs_diff_eq!(dot, expect, epsilon = EPS);
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_dot_c(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let a = make_random_cv(&backend, N)?;
        let b = make_random_cv(&backend, N)?;

        let dot = backend.dot_c(&a, &b)?;

        let a = backend.to_host_cv(a)?;
        let b = backend.to_host_cv(b)?;
        let expect = a
            .iter()
            .zip(b.iter())
            .map(|(a, b)| a.conj() * b)
            .sum::<Complex>();
        approx::assert_abs_diff_eq!(dot.re, expect.re, epsilon = EPS);
        approx::assert_abs_diff_eq!(dot.im, expect.im, epsilon = EPS);
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_add_v(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let a = make_random_v(&backend, N)?;
        let mut b = make_random_v(&backend, N)?;
        let bc = backend.clone_v(&b)?;

        let mut rng = rand::rng();
        let alpha = rng.random();

        backend.add_v(alpha, &a, &mut b)?;

        let a = backend.to_host_v(a)?;
        let b = backend.to_host_v(b)?;
        let bc = backend.to_host_v(bc)?;
        b.iter()
            .zip(a.iter())
            .zip(bc.iter())
            .for_each(|((b, a), bc)| {
                approx::assert_abs_diff_eq!(alpha * a + bc, b, epsilon = EPS);
            });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_add_m(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let a = make_random_m(&backend, N, N)?;
        let mut b = make_random_m(&backend, N, N)?;
        let bc = backend.clone_m(&b)?;

        let mut rng = rand::rng();
        let alpha = rng.random();

        backend.add_m(alpha, &a, &mut b)?;

        let a = backend.to_host_m(a)?;
        let b = backend.to_host_m(b)?;
        let bc = backend.to_host_m(bc)?;
        b.iter()
            .zip(a.iter())
            .zip(bc.iter())
            .for_each(|((b, a), bc)| {
                approx::assert_abs_diff_eq!(alpha * a + bc, b, epsilon = EPS);
            });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_gevv_c(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let mut rng = rand::rng();

        {
            let a = make_random_cv(&backend, N)?;
            let b = make_random_cv(&backend, N)?;
            let mut c = make_random_cm(&backend, N, N)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            assert!(
                backend
                    .gevv_c(Trans::NoTrans, Trans::NoTrans, alpha, &a, &b, beta, &mut c)
                    .is_err()
            );
        }

        {
            let a = make_random_cv(&backend, N)?;
            let b = make_random_cv(&backend, N)?;
            let mut c = make_random_cm(&backend, N, N)?;
            let cc = backend.clone_cm(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gevv_c(Trans::NoTrans, Trans::Trans, alpha, &a, &b, beta, &mut c)?;

            let a = backend.to_host_cv(a)?;
            let b = backend.to_host_cv(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a * b.transpose() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cv(&backend, N)?;
            let b = make_random_cv(&backend, N)?;
            let mut c = make_random_cm(&backend, N, N)?;
            let cc = backend.clone_cm(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gevv_c(
                Trans::NoTrans,
                Trans::ConjTrans,
                alpha,
                &a,
                &b,
                beta,
                &mut c,
            )?;

            let a = backend.to_host_cv(a)?;
            let b = backend.to_host_cv(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a * b.adjoint() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cv(&backend, N)?;
            let b = make_random_cv(&backend, N)?;
            let mut c = make_random_cm(&backend, 1, 1)?;
            let cc = backend.clone_cm(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gevv_c(Trans::Trans, Trans::NoTrans, alpha, &a, &b, beta, &mut c)?;

            let a = backend.to_host_cv(a)?;
            let b = backend.to_host_cv(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a.transpose() * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cv(&backend, N)?;
            let b = make_random_cv(&backend, N)?;
            let mut c = make_random_cm(&backend, N, N)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            assert!(
                backend
                    .gevv_c(Trans::Trans, Trans::Trans, alpha, &a, &b, beta, &mut c)
                    .is_err()
            );
        }

        {
            let a = make_random_cv(&backend, N)?;
            let b = make_random_cv(&backend, N)?;
            let mut c = make_random_cm(&backend, N, N)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            assert!(
                backend
                    .gevv_c(Trans::Trans, Trans::ConjTrans, alpha, &a, &b, beta, &mut c)
                    .is_err()
            );
        }

        {
            let a = make_random_cv(&backend, N)?;
            let b = make_random_cv(&backend, N)?;
            let mut c = make_random_cm(&backend, 1, 1)?;
            let cc = backend.clone_cm(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gevv_c(
                Trans::ConjTrans,
                Trans::NoTrans,
                alpha,
                &a,
                &b,
                beta,
                &mut c,
            )?;

            let a = backend.to_host_cv(a)?;
            let b = backend.to_host_cv(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a.adjoint() * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cv(&backend, N)?;
            let b = make_random_cv(&backend, N)?;
            let mut c = make_random_cm(&backend, N, N)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            assert!(
                backend
                    .gevv_c(Trans::ConjTrans, Trans::Trans, alpha, &a, &b, beta, &mut c)
                    .is_err()
            );
        }

        {
            let a = make_random_cv(&backend, N)?;
            let b = make_random_cv(&backend, N)?;
            let mut c = make_random_cm(&backend, N, N)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            assert!(
                backend
                    .gevv_c(
                        Trans::ConjTrans,
                        Trans::ConjTrans,
                        alpha,
                        &a,
                        &b,
                        beta,
                        &mut c,
                    )
                    .is_err()
            );
        }

        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_gemv_c(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let m = N;
        let n = 2 * N;

        let mut rng = rand::rng();

        {
            let a = make_random_cm(&backend, m, n)?;
            let b = make_random_cv(&backend, n)?;
            let mut c = make_random_cv(&backend, m)?;
            let cc = backend.clone_cv(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemv_c(Trans::NoTrans, alpha, &a, &b, beta, &mut c)?;

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cv(b)?;
            let c = backend.to_host_cv(c)?;
            let cc = backend.to_host_cv(cc)?;
            let expected = a * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(&backend, n, m)?;
            let b = make_random_cv(&backend, n)?;
            let mut c = make_random_cv(&backend, m)?;
            let cc = backend.clone_cv(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemv_c(Trans::Trans, alpha, &a, &b, beta, &mut c)?;

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cv(b)?;
            let c = backend.to_host_cv(c)?;
            let cc = backend.to_host_cv(cc)?;
            let expected = a.transpose() * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(&backend, n, m)?;
            let b = make_random_cv(&backend, n)?;
            let mut c = make_random_cv(&backend, m)?;
            let cc = backend.clone_cv(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemv_c(Trans::ConjTrans, alpha, &a, &b, beta, &mut c)?;

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cv(b)?;
            let c = backend.to_host_cv(c)?;
            let cc = backend.to_host_cv(cc)?;
            let expected = a.adjoint() * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_gemm_c(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let m = N;
        let n = 2 * N;
        let k = 3 * N;

        let mut rng = rand::rng();

        {
            let a = make_random_cm(&backend, m, k)?;
            let b = make_random_cm(&backend, k, n)?;
            let mut c = make_random_cm(&backend, m, n)?;
            let cc = backend.clone_cm(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::NoTrans, Trans::NoTrans, alpha, &a, &b, beta, &mut c)?;

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cm(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(&backend, m, k)?;
            let b = make_random_cm(&backend, n, k)?;
            let mut c = make_random_cm(&backend, m, n)?;
            let cc = backend.clone_cm(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::NoTrans, Trans::Trans, alpha, &a, &b, beta, &mut c)?;

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cm(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a * b.transpose() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(&backend, m, k)?;
            let b = make_random_cm(&backend, n, k)?;
            let mut c = make_random_cm(&backend, m, n)?;
            let cc = backend.clone_cm(&c)?;

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

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cm(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a * b.adjoint() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(&backend, k, m)?;
            let b = make_random_cm(&backend, k, n)?;
            let mut c = make_random_cm(&backend, m, n)?;
            let cc = backend.clone_cm(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::Trans, Trans::NoTrans, alpha, &a, &b, beta, &mut c)?;

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cm(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a.transpose() * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(&backend, k, m)?;
            let b = make_random_cm(&backend, n, k)?;
            let mut c = make_random_cm(&backend, m, n)?;
            let cc = backend.clone_cm(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::Trans, Trans::Trans, alpha, &a, &b, beta, &mut c)?;

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cm(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a.transpose() * b.transpose() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(&backend, k, m)?;
            let b = make_random_cm(&backend, n, k)?;
            let mut c = make_random_cm(&backend, m, n)?;
            let cc = backend.clone_cm(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::Trans, Trans::ConjTrans, alpha, &a, &b, beta, &mut c)?;

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cm(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a.transpose() * b.adjoint() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(&backend, k, m)?;
            let b = make_random_cm(&backend, k, n)?;
            let mut c = make_random_cm(&backend, m, n)?;
            let cc = backend.clone_cm(&c)?;

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

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cm(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a.adjoint() * b * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(&backend, k, m)?;
            let b = make_random_cm(&backend, n, k)?;
            let mut c = make_random_cm(&backend, m, n)?;
            let cc = backend.clone_cm(&c)?;

            let alpha = Complex::new(rng.random(), rng.random());
            let beta = Complex::new(rng.random(), rng.random());
            backend.gemm_c(Trans::ConjTrans, Trans::Trans, alpha, &a, &b, beta, &mut c)?;

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cm(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a.adjoint() * b.transpose() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }

        {
            let a = make_random_cm(&backend, k, m)?;
            let b = make_random_cm(&backend, n, k)?;
            let mut c = make_random_cm(&backend, m, n)?;
            let cc = backend.clone_cm(&c)?;

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

            let a = backend.to_host_cm(a)?;
            let b = backend.to_host_cm(b)?;
            let c = backend.to_host_cm(c)?;
            let cc = backend.to_host_cm(cc)?;
            let expected = a.adjoint() * b.adjoint() * alpha + cc * beta;
            c.iter().zip(expected.iter()).for_each(|(c, expected)| {
                approx::assert_abs_diff_eq!(c.re, expected.re, epsilon = EPS);
                approx::assert_abs_diff_eq!(c.im, expected.im, epsilon = EPS);
            });
        }
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_solve_inplace(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        {
            let tmp = make_random_m(&backend, N, N)?;
            let tmp = backend.to_host_m(tmp)?;

            let a = &tmp * tmp.adjoint();

            let mut rng = rand::rng();
            let x = VectorX::from_iterator(N, (0..N).map(|_| rng.random()));

            let b = &a * &x;

            let aa = backend.from_slice_m(N, N, a.as_slice())?;
            let mut bb = backend.from_slice_v(b.as_slice())?;

            backend.solve_inplace(&aa, &mut bb)?;

            let b2 = &a * backend.to_host_v(bb)?;
            assert!(approx::relative_eq!(b, b2, epsilon = 1e-3));
        }

        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_reduce_col(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let a = make_random_m(&backend, N, N)?;

        let mut b = backend.alloc_v(N)?;

        backend.reduce_col(&a, &mut b)?;

        let a = backend.to_host_m(a)?;
        let b = backend.to_host_v(b)?;

        (0..N).for_each(|row| {
            let sum = a.row(row).iter().sum::<f32>();
            approx::assert_abs_diff_eq!(sum, b[row], epsilon = EPS);
        });
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    fn test_scaled_to_cv(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let a = make_random_cv(&backend, N)?;
        let b = make_random_cv(&backend, N)?;
        let mut c = backend.alloc_cv(N)?;

        backend.scaled_to_cv(&a, &b, &mut c)?;

        let a = backend.to_host_cv(a)?;
        let b = backend.to_host_cv(b)?;
        let c = backend.to_host_cv(c)?;
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
    #[test]
    fn test_scaled_to_assign_cv(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let a = make_random_cv(&backend, N)?;
        let mut b = make_random_cv(&backend, N)?;
        let bc = backend.clone_cv(&b)?;

        backend.scaled_to_assign_cv(&a, &mut b)?;

        let a = backend.to_host_cv(a)?;
        let b = backend.to_host_cv(b)?;
        let bc = backend.to_host_cv(bc)?;
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
    #[test]
    #[case(1, 2)]
    #[case(2, 1)]
    fn test_generate_propagation_matrix_unsafe(
        #[case] dev_num: usize,
        #[case] foci_num: usize,
        backend: NalgebraBackend<Sphere>,
    ) -> Result<(), HoloError> {
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
                        geometry[transducers[j].0].wavenumber(),
                        geometry[transducers[j].0].axial_direction(),
                        &foci[i],
                    )
                })
            });
            g
        };

        let geometry = create_geometry(dev_num, dev_num);
        let foci = gen_foci(foci_num).map(|(p, _)| p).collect::<Vec<_>>();

        let g = backend.generate_propagation_matrix(
            &geometry,
            &foci,
            &TransducerFilter::all_enabled(),
        )?;
        let g = backend.to_host_cm(g)?;
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
    #[test]
    #[case(3, 1)]
    #[case(3, 3)]
    fn test_generate_propagation_matrix_with_disabled_device(
        #[case] dev_num: usize,
        #[case] foci_num: usize,
        backend: NalgebraBackend<Sphere>,
    ) -> Result<(), HoloError> {
        let reference = |geometry: Geometry, foci: Vec<Point3>, filter: TransducerFilter| {
            let mut g = MatrixXc::zeros(
                foci.len(),
                geometry
                    .iter()
                    .map(|dev| filter.num_enabled_transducers(dev))
                    .sum::<usize>(),
            );
            let transducers = geometry
                .iter()
                .filter(|dev| filter.is_enabled_device(dev))
                .flat_map(|dev| dev.iter().map(|tr| (dev.idx(), tr)))
                .collect::<Vec<_>>();
            (0..foci.len()).for_each(|i| {
                (0..transducers.len()).for_each(|j| {
                    g[(i, j)] = propagate::<Sphere>(
                        transducers[j].1,
                        geometry[transducers[j].0].wavenumber(),
                        geometry[transducers[j].0].axial_direction(),
                        &foci[i],
                    )
                })
            });
            g
        };

        let geometry = create_geometry(dev_num, dev_num);
        let filter = TransducerFilter::new(HashMap::from_iter(
            (1..geometry.num_devices()).map(|i| (i, None)),
        ));

        let foci = gen_foci(foci_num).map(|(p, _)| p).collect::<Vec<_>>();

        let g = backend.generate_propagation_matrix(&geometry, &foci, &filter)?;
        let g = backend.to_host_cm(g)?;
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
    #[test]
    #[case(1, 2)]
    #[case(2, 1)]
    fn test_generate_propagation_matrix_with_filter(
        #[case] dev_num: usize,
        #[case] foci_num: usize,
        backend: NalgebraBackend<Sphere>,
    ) -> Result<(), HoloError> {
        let filter = |geometry: &Geometry| -> TransducerFilter {
            TransducerFilter::from_fn(geometry, |dev| {
                let num_transducers = dev.num_transducers();
                Some(move |tr: &Transducer| tr.idx() > num_transducers / 2)
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
                        geometry[transducers[j].0].wavenumber(),
                        geometry[transducers[j].0].axial_direction(),
                        &foci[i],
                    )
                })
            });
            g
        };

        let geometry = create_geometry(dev_num, dev_num);
        let foci = gen_foci(foci_num).map(|(p, _)| p).collect::<Vec<_>>();
        let filter = filter(&geometry);

        let g = backend.generate_propagation_matrix(&geometry, &foci, &filter)?;
        let g = backend.to_host_cm(g)?;
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
    #[test]
    #[case(3, 1)]
    #[case(3, 3)]
    fn test_generate_propagation_matrix_with_filter_with_disabled_devices(
        #[case] dev_num: usize,
        #[case] foci_num: usize,
        backend: NalgebraBackend<Sphere>,
    ) -> Result<(), HoloError> {
        let filter = |geometry: &Geometry| {
            TransducerFilter::from_fn(geometry, |dev| {
                if dev.idx() == 0 {
                    return None;
                }
                let num_transducers = dev.num_transducers();
                Some(move |tr: &Transducer| tr.idx() > num_transducers / 2)
            })
        };

        let reference = |geometry: Geometry, foci: Vec<Point3>, filter: TransducerFilter| {
            let mut g = MatrixXc::zeros(
                foci.len(),
                geometry
                    .iter()
                    .map(|dev| filter.num_enabled_transducers(dev))
                    .sum::<usize>(),
            );
            let transducers = geometry
                .iter()
                .filter(|dev| filter.is_enabled_device(dev))
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
                        geometry[transducers[j].0].wavenumber(),
                        geometry[transducers[j].0].axial_direction(),
                        &foci[i],
                    )
                })
            });
            g
        };

        let geometry = create_geometry(dev_num, dev_num);
        let foci = gen_foci(foci_num).map(|(p, _)| p).collect::<Vec<_>>();
        let filter = filter(&geometry);

        let g = backend.generate_propagation_matrix(&geometry, &foci, &filter)?;
        let g = backend.to_host_cm(g)?;
        assert_eq!(g.nrows(), foci.len());
        assert_eq!(
            g.ncols(),
            geometry
                .iter()
                .filter(|dev| filter.is_enabled_device(dev))
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
    #[test]
    fn test_gen_back_prop(backend: NalgebraBackend<Sphere>) -> Result<(), HoloError> {
        let geometry = create_geometry(1, 1);
        let foci = gen_foci(1).map(|(p, _)| p).collect::<Vec<_>>();

        let m = geometry
            .iter()
            .map(|dev| dev.num_transducers())
            .sum::<usize>();
        let n = foci.len();

        let g = backend.generate_propagation_matrix(
            &geometry,
            &foci,
            &TransducerFilter::all_enabled(),
        )?;

        let b = backend.gen_back_prop(m, n, &g)?;
        let g = backend.to_host_cm(g)?;
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

        let b = backend.to_host_cm(b)?;
        reference.iter().zip(b.iter()).for_each(|(r, b)| {
            approx::assert_abs_diff_eq!(r.re, b.re, epsilon = EPS);
            approx::assert_abs_diff_eq!(r.im, b.im, epsilon = EPS);
        });
        Ok(())
    }
}
