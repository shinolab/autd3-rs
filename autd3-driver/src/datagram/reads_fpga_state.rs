use crate::{datagram::*, defined::DEFAULT_TIMEOUT, geometry::Device};

/// Datagram for configure reads_fpga_state
pub struct ReadsFPGAState<F: Fn(&Device) -> bool> {
    f: F,
}

impl<F: Fn(&Device) -> bool> ReadsFPGAState<F> {
    /// constructor
    pub const fn new(f: F) -> Self {
        Self { f }
    }

    // GRCOV_EXCL_START
    pub const fn f(&self) -> &F {
        &self.f
    }
    // GRCOV_EXCL_STOP
}

impl<F: Fn(&Device) -> bool> Datagram for ReadsFPGAState<F> {
    type O1 = crate::firmware::operation::ReadsFPGAStateOp<F>;
    type O2 = crate::firmware::operation::NullOp;

    fn operation(self) -> (Self::O1, Self::O2) {
        (Self::O1::new(self.f), Self::O2::default())
    }

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // GRCOV_EXCL_START
    fn f(dev: &Device) -> bool {
        dev.idx() == 0
    }
    // GRCOV_EXCL_STOP

    #[test]
    fn test() {
        let datagram = ReadsFPGAState::new(f);
        let _ = datagram.operation();
    }
}
