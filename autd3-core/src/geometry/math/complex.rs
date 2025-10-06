/// a complex number
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    #[must_use]
    pub const fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    #[must_use]
    pub fn arg(&self) -> f32 {
        self.im.atan2(self.re)
    }

    #[must_use]
    pub fn norm_sqr(&self) -> f32 {
        self.re * self.re + self.im * self.im
    }
}
