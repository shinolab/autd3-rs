use autd3_derive::Builder;
use derive_more::Debug;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Builder)]
#[debug("{}", match self.rep { 0xFFFF => "Infinite".to_string(), 0 => "Once".to_string(), i => format!("Finite({})", i + 1) })]
pub struct LoopBehavior {
    #[get]
    pub(crate) rep: u16,
}

pub trait IntoLoopBehaviorFinite {
    type Output;
    fn into_loop_behavior(self) -> Self::Output;
}

impl IntoLoopBehaviorFinite for u16 {
    type Output = Option<LoopBehavior>;
    fn into_loop_behavior(self) -> Self::Output {
        if self == 0 {
            None
        } else {
            Some(LoopBehavior { rep: self - 1 })
        }
    }
}

impl IntoLoopBehaviorFinite for std::num::NonZeroU16 {
    type Output = LoopBehavior;
    fn into_loop_behavior(self) -> Self::Output {
        LoopBehavior {
            rep: self.get() - 1,
        }
    }
}

impl LoopBehavior {
    pub const fn infinite() -> Self {
        LoopBehavior { rep: 0xFFFF }
    }

    pub fn finite<T: IntoLoopBehaviorFinite>(repeat: T) -> T::Output {
        repeat.into_loop_behavior()
    }

    pub const fn once() -> Self {
        Self { rep: 0 }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::infinite(0xFFFF, LoopBehavior::infinite())]
    #[case::finite(0x1233, LoopBehavior::finite(0x1234).unwrap())]
    #[case::once(0x0000, LoopBehavior::once())]
    fn loop_behavior(#[case] expect: u16, #[case] target: LoopBehavior) {
        assert_eq!(expect, target.rep());
    }

    #[rstest::rstest]
    #[test]
    #[case(Some(LoopBehavior{ rep: 0 }), 1)]
    #[case(Some(LoopBehavior{ rep: 0xFFFE }), 0xFFFF)]
    #[case(None, 0)]
    fn into_loop_behavior_u16(#[case] expect: Option<LoopBehavior>, #[case] rep: u16) {
        assert_eq!(expect, LoopBehavior::finite(rep));
    }

    #[rstest::rstest]
    #[test]
    #[case(LoopBehavior{ rep: 0 }, std::num::NonZeroU16::new(1).unwrap())]
    #[case(LoopBehavior{ rep: 0xFFFE }, std::num::NonZeroU16::new(0xFFFF).unwrap())]
    fn into_loop_behavior_non_zero_u16(
        #[case] expect: LoopBehavior,
        #[case] rep: std::num::NonZeroU16,
    ) {
        assert_eq!(expect, LoopBehavior::finite(rep));
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", LoopBehavior::infinite()), "Infinite");
        assert_eq!(format!("{:?}", LoopBehavior::once()), "Once");
        assert_eq!(
            format!("{:?}", LoopBehavior::finite(0x1234).unwrap()),
            "Finite(4660)"
        );
    }
}
