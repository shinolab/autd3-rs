/// a complex number
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };

    #[must_use]
    pub const fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    #[must_use]
    pub const fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    #[must_use]
    pub fn exp(&self) -> Self {
        let exp_re = self.re.exp();
        Self {
            re: exp_re * self.im.cos(),
            im: exp_re * self.im.sin(),
        }
    }

    #[must_use]
    pub fn arg(&self) -> f32 {
        self.im.atan2(self.re)
    }

    #[must_use]
    pub fn norm_sqr(&self) -> f32 {
        self.re * self.re + self.im * self.im
    }

    #[must_use]
    pub fn norm(&self) -> f32 {
        self.norm_sqr().sqrt()
    }
}

impl core::ops::Add<&Complex> for Complex {
    type Output = Complex;

    fn add(self, rhs: &Complex) -> Self::Output {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl core::ops::Add for Complex {
    type Output = Complex;

    fn add(self, rhs: Complex) -> Self::Output {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl core::ops::AddAssign for Complex {
    fn add_assign(&mut self, rhs: Complex) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl core::ops::Mul<f32> for Complex {
    type Output = Complex;

    fn mul(self, rhs: f32) -> Self::Output {
        Complex {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

impl core::ops::Mul<&Complex> for &Complex {
    type Output = Complex;

    fn mul(self, rhs: &Complex) -> Self::Output {
        Complex {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl core::ops::Mul<Complex> for &Complex {
    type Output = Complex;

    fn mul(self, rhs: Complex) -> Self::Output {
        Complex {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl core::ops::Mul for Complex {
    type Output = Complex;

    fn mul(self, rhs: Complex) -> Self::Output {
        Complex {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl core::ops::Div<f32> for Complex {
    type Output = Complex;

    fn div(self, rhs: f32) -> Self::Output {
        Complex {
            re: self.re / rhs,
            im: self.im / rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let c = Complex::new(3.0, 4.0);
        assert_eq!(c.re, 3.0);
        assert_eq!(c.im, 4.0);
    }

    #[test]
    fn zero() {
        let c = Complex::ZERO;
        assert_eq!(c.re, 0.0);
        assert_eq!(c.im, 0.0);
    }

    #[test]
    fn conj() {
        let c = Complex::new(3.0, 4.0);
        let conj = c.conj();
        assert_eq!(conj.re, 3.0);
        assert_eq!(conj.im, -4.0);
    }

    #[rstest::rstest]
    #[case(Complex::new(1.0, 0.0), Complex::new(0.0, 0.0))]
    #[case(
        Complex::new(0.0, 1.0),
        Complex::new(0.0, core::f32::consts::FRAC_PI_2)
    )]
    #[case(Complex::new(-1.0, 0.0), Complex::new(0.0, core::f32::consts::PI))]
    #[case(Complex::new(0.0, -1.0), Complex::new(0.0, -core::f32::consts::FRAC_PI_2))]
    fn exp(#[case] expect: Complex, #[case] value: Complex) {
        let exp = value.exp();
        assert!((exp.re - expect.re).abs() < 1e-6);
        assert!((exp.im - expect.im).abs() < 1e-6);
    }

    #[test]
    fn arg() {
        let c = Complex::new(1.0, 1.0);
        let arg = c.arg();
        assert!((arg - core::f32::consts::PI / 4.0).abs() < 1e-6);
    }

    #[test]
    fn norm() {
        let c = Complex::new(3.0, 4.0);
        assert!((c.norm() - 5.0) < 1e-6);
    }

    #[test]
    fn norm_sqr() {
        let c = Complex::new(3.0, 4.0);
        assert!((c.norm_sqr() - 25.0) < 1e-6);
    }

    #[test]
    fn add() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);
        let result = c1 + c2;
        assert_eq!(result.re, 1.0 + 3.0);
        assert_eq!(result.im, 2.0 + 4.0);
    }

    #[test]
    fn add_ref() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);
        let result = c1 + &c2;
        assert_eq!(result.re, 1.0 + 3.0);
        assert_eq!(result.im, 2.0 + 4.0);
    }

    #[test]
    fn add_assign() {
        let mut c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);
        c1 += c2;
        assert_eq!(c1.re, 1.0 + 3.0);
        assert_eq!(c1.im, 2.0 + 4.0);
    }

    #[test]
    fn mul_scalar() {
        let c = Complex::new(2.0, 3.0);
        let result = c * 2.0;
        assert_eq!(result.re, 2.0 * 2.0);
        assert_eq!(result.im, 3.0 * 2.0);
    }

    #[test]
    fn mul() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);
        let result = c1 * c2;
        assert_eq!(result.re, 1.0 * 3.0 - 2.0 * 4.0);
        assert_eq!(result.im, 1.0 * 4.0 + 2.0 * 3.0);
    }

    #[test]
    fn mul_ref() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);
        let result = &c1 * &c2;
        assert_eq!(result.re, 1.0 * 3.0 - 2.0 * 4.0);
        assert_eq!(result.im, 1.0 * 4.0 + 2.0 * 3.0);
    }

    #[test]
    fn div_scalar() {
        let c = Complex::new(4.0, 6.0);
        let result = c / 2.0;
        assert_eq!(result.re, 4.0 / 2.0);
        assert_eq!(result.im, 6.0 / 2.0);
    }
}
