use std::{
    collections::HashMap,
    mem::{ManuallyDrop, MaybeUninit},
    sync::Arc,
};

use bitvec::{order::Lsb0, vec::BitVec};
use nalgebra::{ComplexField, Dyn, Normed, VecStorage, U1};

use autd3_driver::{
    acoustics::{directivity::Directivity, propagate},
    defined::Complex,
    geometry::Geometry,
};

use crate::{error::HoloError, LinAlgBackend, MatrixX, MatrixXc, VectorX, VectorXc};

pub struct NalgebraBackend<D: Directivity> {
    _phantom: std::marker::PhantomData<D>,
}

impl<D: Directivity> LinAlgBackend<D> for NalgebraBackend<D> {
    type MatrixXc = MatrixXc;
    type MatrixX = MatrixX;
    type VectorXc = VectorXc;
    type VectorX = VectorX;

    fn new() -> Result<Arc<Self>, HoloError> {
        Ok(Arc::new(Self {
            _phantom: std::marker::PhantomData,
        }))
    }

    fn generate_propagation_matrix(
        &self,
        geometry: &Geometry,
        foci: &[autd3_driver::geometry::Vector3],
        filter: &Option<HashMap<usize, BitVec<usize, Lsb0>>>,
    ) -> Result<Self::MatrixXc, HoloError> {
        use rayon::prelude::*;

        let num_transducers = [0]
            .into_iter()
            .chain(geometry.devices().scan(0, |state, dev| {
                *state += filter
                    .as_ref()
                    .map(|f| f.get(&dev.idx()).map(|f| f.count_ones()).unwrap_or(0))
                    .unwrap_or(dev.num_transducers());
                Some(*state)
            }))
            .collect::<Vec<_>>();
        let n = num_transducers[geometry.num_devices()];

        if let Some(filter) = filter {
            if geometry.num_devices() < foci.len() {
                let columns = foci
                .par_iter()
                .map(|f| {
                    nalgebra::Matrix::<Complex, U1, Dyn, VecStorage<Complex, U1, Dyn>>::from_iterator(
                        n,
                        geometry.devices().flat_map(|dev| {
                            let filter = filter.get(&dev.idx());
                            dev.iter().filter_map(move |tr| {
                                filter.and_then(|filter| {
                                    if filter[tr.idx()] {
                                        Some(
                                            propagate::<D>(
                                                tr,
                                                dev.wavenumber(),
                                                dev.axial_direction(),
                                                f,
                                            )
                                        )
                                    } else {
                                        None
                                    }
                                })
                            })
                        }),
                    )
                })
                .collect::<Vec<_>>();
                Ok(MatrixXc::from_rows(&columns))
            } else {
                let mut r = MatrixXc::from_data(unsafe {
                    let mut data = Vec::<MaybeUninit<Complex>>::new();
                    let length = foci.len() * n;
                    data.reserve_exact(length);
                    data.resize_with(length, MaybeUninit::uninit);
                    let uninit = VecStorage::new(Dyn(foci.len()), Dyn(n), data);
                    let vec: Vec<_> = uninit.into();
                    let mut md = ManuallyDrop::new(vec);
                    let new_data =
                        Vec::from_raw_parts(md.as_mut_ptr() as *mut _, md.len(), md.capacity());
                    VecStorage::new(Dyn(foci.len()), Dyn(n), new_data)
                });
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
                let ptr = Ptr(r.as_mut_ptr());
                geometry.devices().par_bridge().for_each(move |dev| {
                    let mut ptr = ptr.add(foci.len() * num_transducers[dev.idx()]);
                    let filter = filter.get(&dev.idx());
                    dev.iter().for_each(move |tr| {
                        if let Some(filter) = filter {
                            if filter[tr.idx()] {
                                foci.iter().for_each(|f| {
                                    ptr.write(propagate::<D>(
                                        tr,
                                        dev.wavenumber(),
                                        dev.axial_direction(),
                                        f,
                                    ));
                                });
                            }
                        }
                    });
                });
                Ok(r)
            }
        } else if geometry.num_devices() < foci.len() {
            let columns = foci
                .par_iter()
                .map(|f| {
                    nalgebra::Matrix::<Complex, U1, Dyn, VecStorage<Complex, U1, Dyn>>::from_iterator(
                        n,
                        geometry.devices().flat_map(|dev| {
                            dev.iter().map(move |tr| {
                                            propagate::<D>(
                                                tr,
                                                dev.wavenumber(),
                                                dev.axial_direction(),
                                                f,
                                            )
                                })
                            })
                    )}
                )
                .collect::<Vec<_>>();
            Ok(MatrixXc::from_rows(&columns))
        } else {
            let mut r = MatrixXc::from_data(unsafe {
                let mut data = Vec::<MaybeUninit<Complex>>::new();
                let length = foci.len() * n;
                data.reserve_exact(length);
                data.resize_with(length, MaybeUninit::uninit);
                let uninit = VecStorage::new(Dyn(foci.len()), Dyn(n), data);
                let vec: Vec<_> = uninit.into();
                let mut md = ManuallyDrop::new(vec);
                let new_data =
                    Vec::from_raw_parts(md.as_mut_ptr() as *mut _, md.len(), md.capacity());
                VecStorage::new(Dyn(foci.len()), Dyn(n), new_data)
            });
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
            let ptr = Ptr(r.as_mut_ptr());
            geometry.devices().par_bridge().for_each(move |dev| {
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

    fn max_eigen_vector_c(&self, m: Self::MatrixXc) -> Result<Self::VectorXc, HoloError> {
        let eig = m.symmetric_eigen();
        Ok(eig.eigenvectors.column(eig.eigenvalues.imax()).into())
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

    fn get_col_c(
        &self,
        a: &Self::MatrixXc,
        col: usize,
        v: &mut Self::VectorXc,
    ) -> Result<(), HoloError> {
        *v = a.column(col).into();
        Ok(())
    }

    fn set_cv(&self, i: usize, val: Complex, v: &mut Self::VectorXc) -> Result<(), HoloError> {
        v[i] = val;
        Ok(())
    }

    fn set_col_c(
        &self,
        a: &Self::VectorXc,
        col: usize,
        start: usize,
        end: usize,
        v: &mut Self::MatrixXc,
    ) -> Result<(), HoloError> {
        v.view_mut((start, col), (end - start, 1))
            .copy_from(&a.view((start, 0), (end - start, 1)));
        Ok(())
    }

    fn set_row_c(
        &self,
        a: &Self::VectorXc,
        row: usize,
        start: usize,
        end: usize,
        v: &mut Self::MatrixXc,
    ) -> Result<(), HoloError> {
        v.view_mut((row, start), (1, end - start))
            .copy_from(&a.view((start, 0), (end - start, 1)).transpose());
        Ok(())
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

    fn pseudo_inverse_svd(
        &self,
        a: Self::MatrixXc,
        alpha: f32,
        _u: &mut Self::MatrixXc,
        _s: &mut Self::MatrixXc,
        _vt: &mut Self::MatrixXc,
        _buf: &mut Self::MatrixXc,
        b: &mut Self::MatrixXc,
    ) -> Result<(), HoloError> {
        let svd = a.svd(true, true);
        let s_inv = MatrixXc::from_diagonal(
            &svd.singular_values
                .map(|s| Complex::new(s / (s * s + alpha * alpha), 0.)),
        );
        match (&svd.v_t, &svd.u) {
            (Some(v_t), Some(u)) => *b = v_t.adjoint() * s_inv * u.adjoint(),
            _ => unreachable!(),
        }
        Ok(())
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

impl Default for NalgebraBackend<autd3_driver::acoustics::directivity::Sphere> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[cfg(all(test, feature = "test-utilities"))]
mod tests {
    use autd3_driver::acoustics::directivity::Sphere;

    use super::*;

    use crate::test_utilities::*;

    #[test]
    fn test_nalgebra_backend() {
        LinAlgBackendTestHelper::<10, NalgebraBackend<Sphere>>::new()
            .unwrap()
            .test()
            .unwrap();
    }
}
