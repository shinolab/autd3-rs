use autd3_core::{
    acoustics::directivity::Directivity,
    environment::Environment,
    gain::TransducerFilter,
    geometry::{Geometry, Point3},
};
use nalgebra::{Dyn, U1, VecStorage};

use crate::{Complex, error::HoloError};

/// Complex matrix
pub type MatrixXc = nalgebra::Matrix<Complex, Dyn, Dyn, VecStorage<Complex, Dyn, Dyn>>;
/// Real matrix
pub type MatrixX = nalgebra::Matrix<f32, Dyn, Dyn, VecStorage<f32, Dyn, Dyn>>;
/// Complex vector
pub type VectorXc = nalgebra::Matrix<Complex, Dyn, U1, VecStorage<Complex, Dyn, U1>>;
/// Real vector
pub type VectorX = nalgebra::Matrix<f32, Dyn, U1, VecStorage<f32, Dyn, U1>>;

/// Transpose
pub enum Trans {
    /// No transpose
    NoTrans,
    /// Transpose
    Trans,
    /// Conjugate transpose
    ConjTrans,
}

/// Linear algebra backend
#[allow(missing_docs)]
pub trait LinAlgBackend {
    type MatrixXc;
    type MatrixX;
    type VectorXc;
    type VectorX;

    fn generate_propagation_matrix<D: Directivity>(
        &self,
        geometry: &Geometry,
        env: &Environment,
        foci: &[Point3],
        filter: &TransducerFilter,
    ) -> Result<Self::MatrixXc, HoloError>;

    fn alloc_v(&self, size: usize) -> Result<Self::VectorX, HoloError>;
    fn alloc_m(&self, rows: usize, cols: usize) -> Result<Self::MatrixX, HoloError>;
    fn alloc_cv(&self, size: usize) -> Result<Self::VectorXc, HoloError>;
    fn alloc_cm(&self, rows: usize, cols: usize) -> Result<Self::MatrixXc, HoloError>;
    fn alloc_zeros_v(&self, size: usize) -> Result<Self::VectorX, HoloError>;
    fn alloc_zeros_cv(&self, size: usize) -> Result<Self::VectorXc, HoloError>;
    fn alloc_zeros_cm(&self, rows: usize, cols: usize) -> Result<Self::MatrixXc, HoloError>;

    fn to_host_v(&self, v: Self::VectorX) -> Result<VectorX, HoloError>;
    fn to_host_m(&self, v: Self::MatrixX) -> Result<MatrixX, HoloError>;
    fn to_host_cv(&self, v: Self::VectorXc) -> Result<VectorXc, HoloError>;
    fn to_host_cm(&self, v: Self::MatrixXc) -> Result<MatrixXc, HoloError>;

    fn cols_c(&self, m: &Self::MatrixXc) -> Result<usize, HoloError>;

    #[allow(clippy::wrong_self_convention)]
    fn from_slice_v(&self, v: &[f32]) -> Result<Self::VectorX, HoloError>;
    #[allow(clippy::wrong_self_convention)]
    fn from_slice_m(&self, rows: usize, cols: usize, v: &[f32])
    -> Result<Self::MatrixX, HoloError>;
    #[allow(clippy::wrong_self_convention)]
    fn from_slice_cv(&self, v: &[f32]) -> Result<Self::VectorXc, HoloError>;
    #[allow(clippy::wrong_self_convention)]
    fn from_slice2_cv(&self, r: &[f32], i: &[f32]) -> Result<Self::VectorXc, HoloError>;
    #[allow(clippy::wrong_self_convention)]
    fn from_slice2_cm(
        &self,
        rows: usize,
        cols: usize,
        r: &[f32],
        i: &[f32],
    ) -> Result<Self::MatrixXc, HoloError>;

    fn copy_from_slice_v(&self, v: &[f32], dst: &mut Self::VectorX) -> Result<(), HoloError>;

    fn copy_to_v(&self, src: &Self::VectorX, dst: &mut Self::VectorX) -> Result<(), HoloError>;
    fn copy_to_m(&self, src: &Self::MatrixX, dst: &mut Self::MatrixX) -> Result<(), HoloError>;

    fn clone_v(&self, v: &Self::VectorX) -> Result<Self::VectorX, HoloError>;
    fn clone_m(&self, v: &Self::MatrixX) -> Result<Self::MatrixX, HoloError>;
    fn clone_cv(&self, v: &Self::VectorXc) -> Result<Self::VectorXc, HoloError>;
    fn clone_cm(&self, v: &Self::MatrixXc) -> Result<Self::MatrixXc, HoloError>;

    fn make_complex2_v(
        &self,
        real: &Self::VectorX,
        imag: &Self::VectorX,
        v: &mut Self::VectorXc,
    ) -> Result<(), HoloError>;

    fn create_diagonal(&self, v: &Self::VectorX, a: &mut Self::MatrixX) -> Result<(), HoloError>;
    fn create_diagonal_c(
        &self,
        v: &Self::VectorXc,
        a: &mut Self::MatrixXc,
    ) -> Result<(), HoloError>;
    fn get_diagonal(&self, a: &Self::MatrixX, v: &mut Self::VectorX) -> Result<(), HoloError>;

    fn norm_squared_cv(&self, a: &Self::VectorXc, b: &mut Self::VectorX) -> Result<(), HoloError>;
    fn real_cm(&self, a: &Self::MatrixXc, b: &mut Self::MatrixX) -> Result<(), HoloError>;
    fn imag_cm(&self, a: &Self::MatrixXc, b: &mut Self::MatrixX) -> Result<(), HoloError>;
    fn scale_assign_cv(&self, a: Complex, b: &mut Self::VectorXc) -> Result<(), HoloError>;
    fn conj_assign_v(&self, b: &mut Self::VectorXc) -> Result<(), HoloError>;
    fn exp_assign_cv(&self, v: &mut Self::VectorXc) -> Result<(), HoloError>;

    fn concat_col_cm(
        &self,
        a: &Self::MatrixXc,
        b: &Self::MatrixXc,
        c: &mut Self::MatrixXc,
    ) -> Result<(), HoloError>;

    fn max_v(&self, m: &Self::VectorX) -> Result<f32, HoloError>;
    fn max_abs_v(&self, m: &Self::VectorX) -> Result<f32, HoloError>;

    fn hadamard_product_cm(
        &self,
        x: &Self::MatrixXc,
        y: &Self::MatrixXc,
        z: &mut Self::MatrixXc,
    ) -> Result<(), HoloError>;

    fn dot(&self, x: &Self::VectorX, y: &Self::VectorX) -> Result<f32, HoloError>;
    fn dot_c(&self, x: &Self::VectorXc, y: &Self::VectorXc) -> Result<Complex, HoloError>;

    fn add_v(&self, alpha: f32, a: &Self::VectorX, b: &mut Self::VectorX) -> Result<(), HoloError>;
    fn add_m(&self, alpha: f32, a: &Self::MatrixX, b: &mut Self::MatrixX) -> Result<(), HoloError>;

    #[allow(clippy::too_many_arguments)]
    fn gevv_c(
        &self,
        trans_a: Trans,
        trans_b: Trans,
        alpha: Complex,
        a: &Self::VectorXc,
        x: &Self::VectorXc,
        beta: Complex,
        y: &mut Self::MatrixXc,
    ) -> Result<(), HoloError>;

    fn gemv_c(
        &self,
        trans: Trans,
        alpha: Complex,
        a: &Self::MatrixXc,
        x: &Self::VectorXc,
        beta: Complex,
        y: &mut Self::VectorXc,
    ) -> Result<(), HoloError>;

    #[allow(clippy::too_many_arguments)]
    fn gemm_c(
        &self,
        trans_a: Trans,
        trans_b: Trans,
        alpha: Complex,
        a: &Self::MatrixXc,
        b: &Self::MatrixXc,
        beta: Complex,
        y: &mut Self::MatrixXc,
    ) -> Result<(), HoloError>;

    fn solve_inplace(&self, a: &Self::MatrixX, x: &mut Self::VectorX) -> Result<(), HoloError>;

    fn reduce_col(&self, a: &Self::MatrixX, b: &mut Self::VectorX) -> Result<(), HoloError>;

    fn scaled_to_cv(
        &self,
        a: &Self::VectorXc,
        b: &Self::VectorXc,
        c: &mut Self::VectorXc,
    ) -> Result<(), HoloError>;
    fn scaled_to_assign_cv(
        &self,
        a: &Self::VectorXc,
        b: &mut Self::VectorXc,
    ) -> Result<(), HoloError>;

    fn gen_back_prop(
        &self,
        _m: usize,
        n: usize,
        transfer: &Self::MatrixXc,
    ) -> Result<Self::MatrixXc, HoloError>;
}
