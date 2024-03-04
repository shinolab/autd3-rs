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
    use super::*;

    #[test]
    fn test_loop_behavior() {
        let d = LoopBehavior::Infinite;

        let dc = Clone::clone(&d);
        assert_eq!(d, dc);
        assert_eq!(
            LoopBehavior::once(),
            LoopBehavior::Finite(std::num::NonZeroU32::new(1).unwrap())
        );

        assert_eq!(format!("{:?}", d), "Infinite");
    }
}
