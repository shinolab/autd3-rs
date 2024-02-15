#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopBehavior {
    Infinite,
    Finite(std::num::NonZeroU32),
}

impl LoopBehavior {
    pub(crate) fn to_rep(&self) -> u32 {
        match self {
            LoopBehavior::Infinite => 0xFFFFFFFF,
            LoopBehavior::Finite(n) => n.get() - 1,
        }
    }
}
