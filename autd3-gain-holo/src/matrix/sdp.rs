/*
 * File: sdp.rs
 * Project: matrix
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 09/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Shun Suzuki. All rights reserved.
 *
 */

use std::{collections::HashMap, rc::Rc};

use rand::Rng;

use crate::{
    constraint::EmissionConstraint, helper::generate_result, impl_holo, Amplitude, Complex,
    LinAlgBackend, Trans,
};
use autd3_derive::Gain;

use autd3_driver::{
    derive::prelude::*,
    geometry::{Geometry, Vector3},
};

/// Gain to produce multiple foci by solving Semi-Denfinite Programming
///
/// Reference
/// * Inoue, Seki, Yasutoshi Makino, and Hiroyuki Shinoda. "Active touch perception produced by airborne ultrasonic haptic hologram." 2015 IEEE World Haptics Conference (WHC). IEEE, 2015.
#[derive(Gain)]
pub struct SDP<B: LinAlgBackend + 'static> {
    foci: Vec<Vector3>,
    amps: Vec<Amplitude>,
    alpha: float,
    lambda: float,
    repeat: usize,
    constraint: EmissionConstraint,
    backend: Rc<B>,
}

impl_holo!(B, SDP<B>);

impl<B: LinAlgBackend + 'static> SDP<B> {
    pub fn new(backend: Rc<B>) -> Self {
        Self {
            foci: vec![],
            amps: vec![],
            alpha: 1e-3,
            lambda: 0.9,
            repeat: 100,
            backend,
            constraint: EmissionConstraint::DontCare,
        }
    }

    pub fn with_alpha(self, alpha: float) -> Self {
        Self { alpha, ..self }
    }

    pub fn with_lambda(self, lambda: float) -> Self {
        Self { lambda, ..self }
    }

    pub fn with_repeat(self, repeat: usize) -> Self {
        Self { repeat, ..self }
    }

    pub fn alpha(&self) -> float {
        self.alpha
    }

    pub fn lambda(&self) -> float {
        self.lambda
    }

    pub fn repeat(&self) -> usize {
        self.repeat
    }
}

impl<B: LinAlgBackend> Gain for SDP<B> {
    #[allow(non_snake_case)]
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        let G = self
            .backend
            .generate_propagation_matrix(geometry, &self.foci, &filter)?;

        let m = self.foci.len();
        let n = self.backend.cols_c(&G)?;

        let zeros = self.backend.alloc_zeros_cv(m)?;
        let ones = self.backend.from_slice_cv(&vec![1.; m])?;

        let P = {
            let mut P = self.backend.alloc_zeros_cm(m, m)?;
            let amps = self.backend.from_slice_cv(self.amps_as_slice())?;
            self.backend.create_diagonal_c(&amps, &mut P)?;
            P
        };

        let G_inv = {
            let mut G_inv = self.backend.alloc_zeros_cm(n, m)?;
            let mut u_ = self.backend.alloc_cm(m, m)?;
            let mut s = self.backend.alloc_cm(n, m)?;
            let mut vt = self.backend.alloc_cm(n, n)?;
            let mut buf = self.backend.alloc_zeros_cm(n, m)?;
            let G_ = self.backend.clone_cm(&G)?;
            self.backend.pseudo_inverse_svd(
                G_, self.alpha, &mut u_, &mut s, &mut vt, &mut buf, &mut G_inv,
            )?;
            G_inv
        };

        let M = {
            let mut M = self.backend.alloc_cm(m, m)?;

            // M = I
            self.backend.create_diagonal_c(&ones, &mut M)?;

            // M = I - GG^{-1}
            self.backend.gemm_c(
                Trans::NoTrans,
                Trans::NoTrans,
                Complex::new(-1., 0.),
                &G,
                &G_inv,
                Complex::new(1., 0.),
                &mut M,
            )?;

            // tmp = P (I - GG^{-1})
            let mut tmp = self.backend.alloc_zeros_cm(m, m)?;
            self.backend.gemm_c(
                Trans::NoTrans,
                Trans::NoTrans,
                Complex::new(1., 0.),
                &P,
                &M,
                Complex::new(0., 0.),
                &mut tmp,
            )?;

            // M = P (I - GG^{-1}) P
            self.backend.gemm_c(
                Trans::NoTrans,
                Trans::NoTrans,
                Complex::new(1., 0.),
                &tmp,
                &P,
                Complex::new(0., 0.),
                &mut M,
            )?;
            M
        };

        // Block coordinate descent
        let u = {
            let mut U = self.backend.alloc_cm(m, m)?;
            self.backend.create_diagonal_c(&ones, &mut U)?;

            let mut rng = rand::thread_rng();

            let mut x = self.backend.alloc_zeros_cv(m)?;
            let mut Mc = self.backend.alloc_cv(m)?;
            for _ in 0..self.repeat {
                let i = rng.gen_range(0..m);

                self.backend.get_col_c(&M, i, &mut Mc)?;
                self.backend.set_cv(i, Complex::new(0., 0.), &mut Mc)?;

                self.backend.gemv_c(
                    Trans::NoTrans,
                    Complex::new(1., 0.),
                    &U,
                    &Mc,
                    Complex::new(0., 0.),
                    &mut x,
                )?;

                let gamma = self.backend.dot_c(&x, &Mc)?;
                if gamma.re > 0. {
                    self.backend.scale_assign_cv(
                        Complex::new(-(self.lambda / gamma.re).sqrt(), 0.),
                        &mut x,
                    )?;

                    self.backend.set_col_c(&x, i, 0, i, &mut U)?;
                    self.backend.set_col_c(&x, i, i + 1, m, &mut U)?;
                    self.backend.conj_assign_v(&mut x)?;
                    self.backend.set_row_c(&x, i, 0, i, &mut U)?;
                    self.backend.set_row_c(&x, i, i + 1, m, &mut U)?;
                } else {
                    self.backend.set_col_c(&zeros, i, 0, i, &mut U)?;
                    self.backend.set_col_c(&zeros, i, i + 1, m, &mut U)?;
                    self.backend.set_row_c(&zeros, i, 0, i, &mut U)?;
                    self.backend.set_row_c(&zeros, i, i + 1, m, &mut U)?;
                }
            }

            self.backend.max_eigen_vector_c(U)?
        };

        let mut ut = self.backend.alloc_zeros_cv(m)?;
        self.backend.gemv_c(
            Trans::NoTrans,
            Complex::new(1., 0.),
            &P,
            &u,
            Complex::new(0., 0.),
            &mut ut,
        )?;

        let mut q = self.backend.alloc_zeros_cv(n)?;
        self.backend.gemv_c(
            Trans::NoTrans,
            Complex::new(1., 0.),
            &G_inv,
            &ut,
            Complex::new(0., 0.),
            &mut q,
        )?;

        generate_result(
            geometry,
            self.backend.to_host_cv(q)?,
            &self.constraint,
            filter,
        )
    }
}
