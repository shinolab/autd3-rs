use super::{EmitIntensity, Phase};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drive {
    /// Phase of ultrasound
    phase: Phase,
    /// emission intensity
    intensity: EmitIntensity,
}

impl Drive {
    pub const fn new(phase: Phase, intensity: EmitIntensity) -> Self {
        Self { phase, intensity }
    }

    pub const fn phase(&self) -> Phase {
        self.phase
    }

    pub const fn intensity(&self) -> EmitIntensity {
        self.intensity
    }

    pub const fn null() -> Self {
        Self {
            phase: Phase::new(0),
            intensity: EmitIntensity::MIN,
        }
    }
}

#[cfg(test)]
impl Drive {
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self {
            phase: Phase::new(rng.gen()),
            intensity: EmitIntensity::new(rng.gen()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive() {
        let d = Drive::new(Phase::new(1), EmitIntensity::new(1));

        let dc = Clone::clone(&d);
        assert_eq!(d.phase, dc.phase);
        assert_eq!(d.intensity, dc.intensity);

        assert_eq!(
            format!("{:?}", d),
            "Drive { phase: Phase { value: 1 }, intensity: EmitIntensity { value: 1 } }"
        );
    }
}
