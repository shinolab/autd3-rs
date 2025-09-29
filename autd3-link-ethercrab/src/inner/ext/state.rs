#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub(crate) struct State(u16);

impl State {
    pub const NONE: Self = Self(0x00);
    pub const INIT: Self = Self(0x01);
    pub const PRE_OP: Self = Self(0x02);
    pub const SAFE_OP: Self = Self(0x04);
    pub const OPERATIONAL: Self = Self(0x08);
    pub const ACK: Self = Self(0x10);
    pub const ERROR: Self = Self(0x10);

    pub const fn state(self) -> u16 {
        self.0
    }

    pub fn is_safe_op(self) -> bool {
        (self & !Self::ERROR) == Self::SAFE_OP
    }

    pub fn is_error(self) -> bool {
        (self & Self::ERROR) != Self::NONE
    }
}

impl core::ops::BitAnd for State {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::Not for State {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl core::ops::Add for State {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl From<u16> for State {
    fn from(state: u16) -> Self {
        Self(state as _)
    }
}

impl std::fmt::Display for State {
    #[allow(non_upper_case_globals)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self & !Self::ERROR {
            Self::NONE => write!(f, "NONE")?,
            Self::INIT => write!(f, "INIT")?,
            Self::PRE_OP => write!(f, "PRE-OP")?,
            Self::SAFE_OP => write!(f, "SAFE-OP")?,
            Self::OPERATIONAL => write!(f, "OP")?,
            _ => {
                return write!(f, "UNKNOWN ({})", self.0);
            }
        };
        if self.is_error() {
            write!(f, " + ERROR")
        } else {
            Ok(())
        }
    }
}
