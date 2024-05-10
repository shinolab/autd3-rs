#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopBehavior {
    pub(crate) rep: u32,
}

pub trait IntoLoopBehaviorFinite {
    type Output;
    fn into_loop_behavior(self) -> Self::Output;
}

impl IntoLoopBehaviorFinite for u32 {
    type Output = Option<LoopBehavior>;
    fn into_loop_behavior(self) -> Self::Output {
        if self == 0 {
            None
        } else {
            Some(LoopBehavior { rep: self - 1 })
        }
    }
}

impl IntoLoopBehaviorFinite for std::num::NonZeroU32 {
    type Output = LoopBehavior;
    fn into_loop_behavior(self) -> Self::Output {
        LoopBehavior {
            rep: self.get() - 1,
        }
    }
}

impl LoopBehavior {
    pub const fn infinite() -> Self {
        LoopBehavior { rep: 0xFFFFFFFF }
    }

    pub fn finite<T: IntoLoopBehaviorFinite>(repeat: T) -> T::Output {
        repeat.into_loop_behavior()
    }

    pub const fn once() -> Self {
        Self { rep: 0 }
    }

    pub const fn rep(&self) -> u32 {
        self.rep
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::infinite(0xFFFFFFFF, LoopBehavior::infinite())]
    #[case::finite(0x12345677, LoopBehavior::finite(0x12345678).unwrap())]
    #[case::once(0x00000000, LoopBehavior::once())]
    fn loop_behavior(#[case] expect: u32, #[case] target: LoopBehavior) {
        assert_eq!(expect, target.rep());
    }

    #[rstest::rstest]
    #[test]
    #[case(Some(LoopBehavior{ rep: 0 }), 1)]
    #[case(Some(LoopBehavior{ rep: 0xFFFFFFFE }), 0xFFFFFFFF)]
    #[case(None, 0)]
    fn into_loop_behavior_u32(#[case] expect: Option<LoopBehavior>, #[case] rep: u32) {
        assert_eq!(expect, LoopBehavior::finite(rep));
    }

    #[rstest::rstest]
    #[test]
    #[case(LoopBehavior{ rep: 0 }, std::num::NonZeroU32::new(1).unwrap())]
    #[case(LoopBehavior{ rep: 0xFFFFFFFE }, std::num::NonZeroU32::new(0xFFFFFFFF).unwrap())]
    fn into_loop_behavior_non_zero_u32(
        #[case] expect: LoopBehavior,
        #[case] rep: std::num::NonZeroU32,
    ) {
        assert_eq!(expect, LoopBehavior::finite(rep));
    }
}
