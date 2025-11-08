use crate::{firmware::operation::Operation, geometry::Device};

pub struct NullOp;

impl Operation<'_> for NullOp {
    type Error = std::convert::Infallible;

    fn required_size(&self, _: &Device) -> usize {
        0
    }

    fn pack(&mut self, _: &Device, _: &mut [u8]) -> Result<usize, Self::Error> {
        unreachable!()
    }

    fn is_done(&self) -> bool {
        true
    }
}

impl Default for Box<dyn Operation<'_, Error = std::convert::Infallible>> {
    fn default() -> Self {
        Box::new(NullOp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_op() {
        let device = crate::tests::create_device();
        let op = NullOp;
        assert_eq!(op.required_size(&device), 0);
        assert!(op.is_done());
    }

    #[test]
    #[should_panic]
    fn pack() {
        let device = crate::tests::create_device();
        let mut op = NullOp;
        let mut buf = [];
        let _ = op.pack(&device, &mut buf);
    }

    #[test]
    fn default() {
        let _op: Box<dyn Operation<'_, Error = std::convert::Infallible>> = Default::default();
    }
}
