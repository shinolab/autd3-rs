use autd3_core::{
    acoustics::directivity::Directivity,
    environment::Environment,
    gain::TransducerMask,
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

    fn alloc_v(&self, size: usize) -> Result<Self::VectorX, HoloError>;
    fn alloc_zeros_cv(&self, size: usize) -> Result<Self::VectorXc, HoloError>;
    fn alloc_zeros_cm(&self, rows: usize, cols: usize) -> Result<Self::MatrixXc, HoloError>;

    fn cv_from_slice(&self, v: &[f32]) -> Result<Self::VectorXc, HoloError>;

    fn clone_cv(&self, v: &Self::VectorXc) -> Result<Self::VectorXc, HoloError>;

    fn to_host_cv(&self, v: Self::VectorXc) -> Result<VectorXc, HoloError>;

    fn cols_c(&self, m: &Self::MatrixXc) -> Result<usize, HoloError>;

    fn norm_squared_cv(&self, a: &Self::VectorXc, b: &mut Self::VectorX) -> Result<(), HoloError>;

    fn max_v(&self, m: &Self::VectorX) -> Result<f32, HoloError>;

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

    fn generate_propagation_matrix<D: Directivity>(
        &self,
        geometry: &Geometry,
        env: &Environment,
        foci: &[Point3],
        filter: &TransducerMask,
    ) -> Result<Self::MatrixXc, HoloError>;
    fn gen_back_prop(
        &self,
        _m: usize,
        n: usize,
        transfer: &Self::MatrixXc,
    ) -> Result<Self::MatrixXc, HoloError>;
}
