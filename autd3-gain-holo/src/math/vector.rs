use autd3_core::geometry::Complex;

use crate::math::matrix::MatrixXc;

#[derive(Debug, Clone)]
pub struct VectorX<T> {
    pub rows: usize,
    pub data: Vec<T>,
}

impl<T> std::ops::Deref for VectorX<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> VectorX<T> {
    #[must_use]
    pub fn from_iterator<I: IntoIterator<Item = T>>(n: usize, iter: I) -> Self {
        Self {
            rows: n,
            data: iter.into_iter().take(n).collect(),
        }
    }

    #[must_use]
    pub fn map<U, F: Fn(T) -> U>(&self, f: F) -> VectorX<U>
    where
        T: Clone,
        Vec<U>: FromIterator<U>,
    {
        VectorX {
            rows: self.rows,
            data: self.data.iter().cloned().map(f).collect(),
        }
    }

    #[must_use]
    pub fn zip_map<U, V, F: Fn(T, U) -> V>(&self, other: &VectorX<U>, f: F) -> VectorX<V>
    where
        T: Clone,
        U: Clone,
        Vec<V>: FromIterator<V>,
    {
        VectorX {
            rows: self.rows,
            data: self
                .data
                .iter()
                .cloned()
                .zip(other.data.iter().cloned())
                .map(|(a, b)| f(a, b))
                .collect(),
        }
    }

    pub fn zip_apply<U, F: Fn(&mut T, U)>(&mut self, other: &VectorX<U>, f: F)
    where
        T: Clone,
        U: Clone,
    {
        self.data
            .iter_mut()
            .zip(other.data.iter().cloned())
            .for_each(|(a, b)| f(a, b));
    }
}

impl VectorX<f32> {
    #[must_use]
    pub fn max(&self) -> f32 {
        self.data
            .iter()
            .copied()
            .fold(f32::MIN, |a, b| if a > b { a } else { b })
    }
}

impl VectorX<Complex> {
    #[must_use]
    pub fn zeros(n: usize) -> Self {
        Self {
            rows: n,
            data: vec![Complex::new(0., 0.); n],
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn gemv(&mut self, alpha: Complex, a: &MatrixXc, x: &VectorX<Complex>, beta: Complex) {
        let mut res = vec![Complex::new(0., 0.); self.rows];
        for i in 0..a.nrows() {
            for j in 0..a.ncols() {
                res[i] += a[(i, j)] * x.data[j];
            }
        }
        for i in 0..self.rows {
            self.data[i] = alpha * res[i] + beta * self.data[i];
        }
    }
}

pub type VectorXc = VectorX<autd3_core::geometry::Complex>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iterator() {
        let v = VectorX::from_iterator(5, 0..10);
        assert_eq!(v.rows, 5);
        assert_eq!(v.data, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn deref() {
        let v = VectorX::from_iterator(3, vec![1, 2, 3]);
        assert_eq!(&v[..], &[1, 2, 3]);
    }

    #[test]
    fn map() {
        let v = VectorX::from_iterator(3, vec![1, 2, 3]);
        let v2 = v.map(|x| x * 2);
        assert_eq!(v2.rows, 3);
        assert_eq!(v2.data, vec![2, 4, 6]);
    }

    #[test]
    fn zip_map() {
        let v1 = VectorX::from_iterator(3, vec![1, 2, 3]);
        let v2 = VectorX::from_iterator(3, vec![4, 5, 6]);
        let v3 = v1.zip_map(&v2, |a, b| a + b);
        assert_eq!(v3.rows, 3);
        assert_eq!(v3.data, vec![5, 7, 9]);
    }

    #[test]
    fn zip_apply() {
        let mut v1 = VectorX::from_iterator(3, vec![1, 2, 3]);
        let v2 = VectorX::from_iterator(3, vec![4, 5, 6]);
        v1.zip_apply(&v2, |a, b| *a += b);
        assert_eq!(v1.data, vec![5, 7, 9]);
    }

    #[test]
    fn f32_max() {
        let v = VectorX::from_iterator(5, vec![1.0, 5.0, 3.0, 2.0, 4.0]);
        assert_eq!(v.max(), 5.0);
    }

    #[test]
    fn complex_zeros() {
        let v = VectorXc::zeros(3);
        assert_eq!(v.rows, 3);
        assert_eq!(v.data.len(), 3);
        for c in v.data.iter() {
            assert_eq!(c.re, 0.0);
            assert_eq!(c.im, 0.0);
        }
    }

    #[test]
    fn complex_gemv() {
        let mut a = MatrixXc::zeros(3, 2);
        a[(0, 0)] = Complex::new(1.0, 1.0);
        a[(0, 1)] = Complex::new(2.0, 2.0);
        a[(1, 0)] = Complex::new(3.0, 3.0);
        a[(1, 1)] = Complex::new(4.0, 4.0);
        a[(2, 0)] = Complex::new(5.0, 5.0);
        a[(2, 1)] = Complex::new(6.0, 6.0);

        let x = VectorX::from_iterator(2, vec![Complex::new(1.0, 0.0), Complex::new(1.0, 0.0)]);

        let mut y = VectorXc::zeros(3);
        y.gemv(Complex::new(1.0, 0.0), &a, &x, Complex::new(0.0, 0.0));

        assert_eq!(y.data[0].re, 3.0);
        assert_eq!(y.data[0].im, 3.0);
        assert_eq!(y.data[1].re, 7.0);
        assert_eq!(y.data[1].im, 7.0);
        assert_eq!(y.data[2].re, 11.0);
        assert_eq!(y.data[2].im, 11.0);
    }

    #[test]
    fn complex_gemv_with_alpha_beta() {
        let mut a = MatrixXc::zeros(2, 2);
        a[(0, 0)] = Complex::new(1.0, 0.0);
        a[(0, 1)] = Complex::new(0.0, 0.0);
        a[(1, 0)] = Complex::new(0.0, 0.0);
        a[(1, 1)] = Complex::new(1.0, 0.0);

        let x = VectorX::from_iterator(2, vec![Complex::new(2.0, 0.0), Complex::new(3.0, 0.0)]);

        let mut y = VectorX::from_iterator(2, vec![Complex::new(1.0, 0.0), Complex::new(1.0, 0.0)]);

        y.gemv(Complex::new(2.0, 0.0), &a, &x, Complex::new(3.0, 0.0));

        assert_eq!(y.data[0].re, 7.0);
        assert_eq!(y.data[1].re, 9.0);
    }
}
