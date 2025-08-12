use core::f32::consts::PI;

use crate::common::{METER, ULTRASOUND_FREQ};

#[cfg(not(feature = "std"))]
use num_traits::float::Float;

#[non_exhaustive]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Environment {
    pub sound_speed: f32,
}

impl Environment {
    /// Creates a new environment with the default sound speed (340m/s).
    pub fn new() -> Self {
        Self {
            sound_speed: 340.0 * METER,
        }
    }

    /// Sets the sound speed of envs from the temperature.
    ///
    /// This is equivalent to `Self::set_sound_speed_from_temp_with(t, 1.4, 8.314_463, 28.9647e-3)`.
    pub fn set_sound_speed_from_temp(&mut self, t: f32) {
        self.set_sound_speed_from_temp_with(t, 1.4, 8.314_463, 28.9647e-3);
    }

    /// Sets the sound speed of envs from the temperature `t`, heat capacity ratio `k`, gas constant `r`, and molar mass `m` [kg/mol].
    pub fn set_sound_speed_from_temp_with(&mut self, t: f32, k: f32, r: f32, m: f32) {
        self.sound_speed = (k * r * (273.15 + t) / m).sqrt() * METER;
    }

    /// Gets the wavelength of the ultrasound.
    #[must_use]
    pub const fn wavelength(&self) -> f32 {
        self.sound_speed / ULTRASOUND_FREQ.hz() as f32
    }

    /// Gets the wavenumber of the ultrasound.
    #[must_use]
    pub const fn wavenumber(&self) -> f32 {
        2.0 * PI * ULTRASOUND_FREQ.hz() as f32 / self.sound_speed
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::common::mm;

    #[rstest::rstest]
    #[case(340.29525e3, 15.)]
    #[case(343.23497e3, 20.)]
    #[case(349.04013e3, 30.)]
    fn set_sound_speed_from_temp(#[case] expected: f32, #[case] temp: f32) {
        let mut env = Environment::new();
        env.set_sound_speed_from_temp(temp);
        approx::assert_abs_diff_eq!(expected * mm, env.sound_speed, epsilon = 1e-3);
    }

    #[rstest::rstest]
    #[case(8.5, 340e3)]
    #[case(10., 400e3)]
    fn wavelength(#[case] expect: f32, #[case] c: f32) {
        let mut env = Environment::new();
        env.sound_speed = c;
        approx::assert_abs_diff_eq!(expect, env.wavelength());
    }

    #[rstest::rstest]
    #[case(0.739_198_27, 340e3)]
    #[case(0.628_318_55, 400e3)]
    fn wavenumber(#[case] expect: f32, #[case] c: f32) {
        let mut env = Environment::new();
        env.sound_speed = c;
        approx::assert_abs_diff_eq!(expect, env.wavenumber());
    }
}
