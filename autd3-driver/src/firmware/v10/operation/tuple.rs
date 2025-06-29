use super::OperationGenerator;
use crate::firmware::driver::NullOp;

use autd3_core::{datagram::CombinedOperationGenerator, geometry::Device};

impl<O1, O2> OperationGenerator for CombinedOperationGenerator<O1, O2>
where
    O1: OperationGenerator<O2 = NullOp>,
    O2: OperationGenerator<O2 = NullOp>,
{
    type O1 = O1::O1;
    type O2 = O2::O1;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        match (self.o1.generate(device), self.o2.generate(device)) {
            (Some((o1, _)), Some((o2, _))) => Some((o1, o2)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestOpGen {
        has: bool,
    }

    impl OperationGenerator for TestOpGen {
        type O1 = NullOp;
        type O2 = NullOp;

        fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
            self.has.then_some((NullOp {}, NullOp {}))
        }
    }

    #[rstest::rstest]
    #[case(true, true)]
    #[case(false, false)]
    #[test]
    fn combined_operation_generator(#[case] expect: bool, #[case] has: bool) {
        let device = crate::autd3_device::tests::create_device();
        let mut generator = CombinedOperationGenerator {
            o1: TestOpGen { has },
            o2: TestOpGen { has },
        };
        assert_eq!(expect, generator.generate(&device).is_some());
    }
}
