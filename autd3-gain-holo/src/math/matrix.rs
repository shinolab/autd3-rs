use autd3_core::geometry::Complex;

use super::RowVectorXc;

#[derive(Debug)]
pub struct MatrixXc {
    pub nrows: usize,
    pub ncols: usize,
    pub data: Vec<Complex>,
}

pub struct RowMatrixView<'a> {
    nrows: usize,
    ncols: usize,
    first_row: usize,
    view_nrows: usize,
    data: &'a [Complex],
}

pub struct RowMatrixViewIter<'a> {
    cur_row: usize,
    cur_col: usize,
    nrows: usize,
    ncols: usize,
    first_row: usize,
    view_nrows: usize,
    data: &'a [Complex],
}

impl<'a> RowMatrixView<'a> {
    pub fn iter(self) -> RowMatrixViewIter<'a> {
        RowMatrixViewIter {
            cur_row: self.first_row,
            cur_col: 0,
            nrows: self.nrows,
            ncols: self.ncols,
            first_row: self.first_row,
            view_nrows: self.view_nrows,
            data: self.data,
        }
    }
}

impl<'a> Iterator for RowMatrixViewIter<'a> {
    type Item = Complex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_row >= self.first_row + self.view_nrows {
            self.cur_row = self.first_row;
            self.cur_col += 1;
            if self.cur_col >= self.ncols {
                return None;
            }
        }
        let value = self.data[self.cur_col * self.nrows + self.cur_row];
        self.cur_row += 1;
        Some(value)
    }
}

impl MatrixXc {
    #[must_use]
    pub fn zeros(nrows: usize, ncols: usize) -> Self {
        Self {
            nrows,
            ncols,
            data: vec![Complex::ZERO; nrows * ncols],
        }
    }

    #[must_use]
    pub fn from_vec(nrows: usize, ncols: usize, data: Vec<Complex>) -> Self {
        Self { nrows, ncols, data }
    }

    #[must_use]
    pub fn from_rows(rows: &[RowVectorXc]) -> Self {
        let nrows = rows.len();
        let ncols = rows[0].rows;
        let mut data = vec![Complex::ZERO; nrows * ncols];
        for (i, row) in rows.iter().enumerate() {
            for j in 0..ncols {
                data[j * nrows + i] = row.data[j];
            }
        }
        Self { nrows, ncols, data }
    }

    #[must_use]
    pub fn ncols(&self) -> usize {
        self.ncols
    }

    #[must_use]
    pub fn nrows(&self) -> usize {
        self.nrows
    }

    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut Complex {
        self.data.as_mut_ptr()
    }

    #[must_use]
    pub fn rows(&self, first_row: usize, nrows: usize) -> RowMatrixView<'_> {
        RowMatrixView {
            nrows: self.nrows,
            ncols: self.ncols,
            first_row,
            view_nrows: nrows,
            data: &self.data,
        }
    }

    pub fn gemm(&mut self, alpha: Complex, a: &MatrixXc, b: &MatrixXc, beta: Complex) {
        let alpha_is_zero = alpha.re == 0.0 && alpha.im == 0.0;
        let beta_is_zero = beta.re == 0.0 && beta.im == 0.0;
        let beta_is_one = beta.re == 1.0 && beta.im == 0.0;

        if alpha_is_zero {
            if beta_is_zero {
                self.data.fill(Complex::new(0., 0.));
            } else if !beta_is_one {
                self.data.iter_mut().for_each(|c| *c *= beta);
            }
            return;
        }

        if beta_is_zero {
            self.data.fill(Complex::new(0., 0.));
        } else if !beta_is_one {
            self.data.iter_mut().for_each(|c| *c *= beta);
        }

        for j in 0..b.ncols() {
            for k in 0..a.ncols() {
                let alpha_b = alpha * b[(k, j)];
                for i in 0..a.nrows() {
                    self.data[j * self.nrows + i] += a[(i, k)] * alpha_b;
                }
            }
        }
    }
}

impl std::ops::Index<(usize, usize)> for MatrixXc {
    type Output = Complex;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (i, j) = index;
        &self.data[j * self.nrows + i]
    }
}

impl std::ops::IndexMut<(usize, usize)> for MatrixXc {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (i, j) = index;
        &mut self.data[j * self.nrows + i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeros() {
        let m = MatrixXc::zeros(3, 4);
        assert_eq!(m.nrows(), 3);
        assert_eq!(m.ncols(), 4);
        assert_eq!(m.data.len(), 12);
        for c in m.data.iter() {
            assert_eq!(c.re, 0.0);
            assert_eq!(c.im, 0.0);
        }
    }

    #[test]
    fn from_vec() {
        let data = vec![
            Complex::new(1.0, 0.0),
            Complex::new(2.0, 0.0),
            Complex::new(3.0, 0.0),
            Complex::new(4.0, 0.0),
        ];
        let m = MatrixXc::from_vec(2, 2, data);
        assert_eq!(m.nrows(), 2);
        assert_eq!(m.ncols(), 2);
        assert_eq!(m.data.len(), 4);
    }

    #[test]
    fn from_raws() {
        let row1 = RowVectorXc::from_iterator(
            3,
            vec![
                Complex::new(1.0, 0.0),
                Complex::new(2.0, 0.0),
                Complex::new(3.0, 0.0),
            ],
        );
        let row2 = RowVectorXc::from_iterator(
            3,
            vec![
                Complex::new(4.0, 0.0),
                Complex::new(5.0, 0.0),
                Complex::new(6.0, 0.0),
            ],
        );
        let m = MatrixXc::from_rows(&[row1, row2]);
        assert_eq!(m.nrows(), 2);
        assert_eq!(m.ncols(), 3);
        assert_eq!(m.data.len(), 6);
        assert_eq!(m[(0, 0)].re, 1.0);
        assert_eq!(m[(0, 1)].re, 2.0);
        assert_eq!(m[(0, 2)].re, 3.0);
        assert_eq!(m[(1, 0)].re, 4.0);
        assert_eq!(m[(1, 1)].re, 5.0);
        assert_eq!(m[(1, 2)].re, 6.0);
    }

    #[test]
    fn index() {
        let mut m = MatrixXc::zeros(2, 2);
        m[(0, 0)] = Complex::new(1.0, 1.0);
        m[(0, 1)] = Complex::new(2.0, 2.0);
        m[(1, 0)] = Complex::new(3.0, 3.0);
        m[(1, 1)] = Complex::new(4.0, 4.0);

        assert_eq!(m[(0, 0)].re, 1.0);
        assert_eq!(m[(0, 0)].im, 1.0);
        assert_eq!(m[(0, 1)].re, 2.0);
        assert_eq!(m[(0, 1)].im, 2.0);
        assert_eq!(m[(1, 0)].re, 3.0);
        assert_eq!(m[(1, 0)].im, 3.0);
        assert_eq!(m[(1, 1)].re, 4.0);
        assert_eq!(m[(1, 1)].im, 4.0);
    }

    #[test]
    fn rows() {
        let mut m = MatrixXc::zeros(4, 3);
        for i in 0..4 {
            for j in 0..3 {
                m[(i, j)] = Complex::new((i * 3 + j) as f32, 0.0);
            }
        }

        let view = m.rows(1, 2);
        let collected: Vec<Complex> = view.iter().collect();

        assert_eq!(collected.len(), 6);

        assert_eq!(collected[0].re, 3.0);
        assert_eq!(collected[1].re, 6.0);
        assert_eq!(collected[2].re, 4.0);
        assert_eq!(collected[3].re, 7.0);
        assert_eq!(collected[4].re, 5.0);
        assert_eq!(collected[5].re, 8.0);
    }
    #[test]
    fn gemm_simple() {
        let mut a = MatrixXc::zeros(2, 2);
        a[(0, 0)] = Complex::new(1.0, 0.0);
        a[(0, 1)] = Complex::new(2.0, 0.0);
        a[(1, 0)] = Complex::new(3.0, 0.0);
        a[(1, 1)] = Complex::new(4.0, 0.0);

        let mut b = MatrixXc::zeros(2, 2);
        b[(0, 0)] = Complex::new(1.0, 0.0);
        b[(1, 1)] = Complex::new(1.0, 0.0);

        let mut c = MatrixXc::zeros(2, 2);
        c.gemm(Complex::new(1.0, 0.0), &a, &b, Complex::new(0.0, 0.0));

        assert_eq!(c[(0, 0)].re, 1.0);
        assert_eq!(c[(0, 1)].re, 2.0);
        assert_eq!(c[(1, 0)].re, 3.0);
        assert_eq!(c[(1, 1)].re, 4.0);
    }

    #[test]
    fn gemm_complex() {
        let mut a = MatrixXc::zeros(2, 2);
        a[(0, 0)] = Complex::new(1.0, 1.0);
        a[(1, 1)] = Complex::new(1.0, 1.0);

        let mut b = MatrixXc::zeros(2, 2);
        b[(0, 0)] = Complex::new(1.0, 0.0);
        b[(0, 1)] = Complex::new(1.0, 0.0);
        b[(1, 0)] = Complex::new(1.0, 0.0);
        b[(1, 1)] = Complex::new(1.0, 0.0);

        let mut c = MatrixXc::zeros(2, 2);
        c.gemm(Complex::new(1.0, 0.0), &a, &b, Complex::new(0.0, 0.0));

        assert_eq!(c[(0, 0)].re, 1.0);
        assert_eq!(c[(0, 0)].im, 1.0);
        assert_eq!(c[(0, 1)].re, 1.0);
        assert_eq!(c[(0, 1)].im, 1.0);
        assert_eq!(c[(1, 0)].re, 1.0);
        assert_eq!(c[(1, 0)].im, 1.0);
        assert_eq!(c[(1, 1)].re, 1.0);
        assert_eq!(c[(1, 1)].im, 1.0);
    }

    #[test]
    fn gemm_with_alpha_beta() {
        let mut a = MatrixXc::zeros(2, 2);
        a[(0, 0)] = Complex::new(1.0, 0.0);
        a[(0, 1)] = Complex::new(0.0, 0.0);
        a[(1, 0)] = Complex::new(0.0, 0.0);
        a[(1, 1)] = Complex::new(1.0, 0.0);

        let mut b = MatrixXc::zeros(2, 2);
        b[(0, 0)] = Complex::new(2.0, 0.0);
        b[(0, 1)] = Complex::new(0.0, 0.0);
        b[(1, 0)] = Complex::new(0.0, 0.0);
        b[(1, 1)] = Complex::new(2.0, 0.0);

        let mut c = MatrixXc::zeros(2, 2);
        c[(0, 0)] = Complex::new(1.0, 0.0);
        c[(1, 1)] = Complex::new(1.0, 0.0);

        c.gemm(Complex::new(2.0, 0.0), &a, &b, Complex::new(3.0, 0.0));

        assert_eq!(c[(0, 0)].re, 7.0);
        assert_eq!(c[(1, 1)].re, 7.0);
        assert_eq!(c[(0, 1)].re, 0.0);
        assert_eq!(c[(1, 0)].re, 0.0);
    }

    #[test]
    fn gemm_non_square() {
        let mut a = MatrixXc::zeros(2, 3);
        a[(0, 0)] = Complex::new(1.0, 0.0);
        a[(0, 1)] = Complex::new(2.0, 0.0);
        a[(0, 2)] = Complex::new(3.0, 0.0);
        a[(1, 0)] = Complex::new(4.0, 0.0);
        a[(1, 1)] = Complex::new(5.0, 0.0);
        a[(1, 2)] = Complex::new(6.0, 0.0);

        let mut b = MatrixXc::zeros(3, 2);
        b[(0, 0)] = Complex::new(1.0, 0.0);
        b[(0, 1)] = Complex::new(0.0, 0.0);
        b[(1, 0)] = Complex::new(0.0, 0.0);
        b[(1, 1)] = Complex::new(1.0, 0.0);
        b[(2, 0)] = Complex::new(0.0, 0.0);
        b[(2, 1)] = Complex::new(0.0, 0.0);

        let mut c = MatrixXc::zeros(2, 2);
        c.gemm(Complex::new(1.0, 0.0), &a, &b, Complex::new(0.0, 0.0));

        assert_eq!(c[(0, 0)].re, 1.0);
        assert_eq!(c[(0, 1)].re, 2.0);
        assert_eq!(c[(1, 0)].re, 4.0);
        assert_eq!(c[(1, 1)].re, 5.0);
    }
}
