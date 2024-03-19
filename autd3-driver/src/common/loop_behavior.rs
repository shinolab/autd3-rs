#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopBehavior {
    Infinite,
    Finite(std::num::NonZeroU32),
}

impl LoopBehavior {
    pub(crate) fn to_rep(self) -> u32 {
        match self {
            LoopBehavior::Infinite => 0xFFFFFFFF,
            LoopBehavior::Finite(n) => n.get() - 1,
        }
    }

    pub fn once() -> Self {
        LoopBehavior::Finite(std::num::NonZeroU32::new(1).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::infinite(0xFFFFFFFF, LoopBehavior::Infinite)]
    #[case::finite(0x12345677, LoopBehavior::Finite(NonZeroU32::new(0x12345678).unwrap()))]
    #[case::once(0x00000000, LoopBehavior::once())]
    fn test(#[case] expected: u32, #[case] target: LoopBehavior) {
        assert_eq!(expected, target.to_rep());
    }
}
