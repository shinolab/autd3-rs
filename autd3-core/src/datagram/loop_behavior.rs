use std::num::NonZeroU16;

/// The behavior of the loop.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum LoopBehavior {
    /// Infinite loop.
    Infinite,
    /// Finite loop.
    Finite(NonZeroU16),
}

impl std::fmt::Debug for LoopBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoopBehavior::Infinite => write!(f, "Infinite"),
            LoopBehavior::Finite(rep) => write!(f, "Finite({})", rep.get()),
        }
    }
}

impl LoopBehavior {
    /// Creates a new [`LoopBehavior`] with a single loop.
    pub const ONCE: Self = Self::Finite(NonZeroU16::MIN);
}

impl LoopBehavior {
    #[doc(hidden)]
    pub fn rep(&self) -> u16 {
        match self {
            LoopBehavior::Infinite => 0xFFFF,
            LoopBehavior::Finite(rep) => rep.get() - 1,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::infinite(0xFFFF, LoopBehavior::Infinite)]
    #[case::finite(0x1233, LoopBehavior::Finite(NonZeroU16::new(0x1234).unwrap()))]
    #[case::once(0x0000, LoopBehavior::ONCE)]
    fn rep(#[case] expect: u16, #[case] target: LoopBehavior) {
        assert_eq!(expect, target.rep());
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", LoopBehavior::Infinite), "Infinite");
        assert_eq!(format!("{:?}", LoopBehavior::ONCE), "Finite(1)");
    }
}
