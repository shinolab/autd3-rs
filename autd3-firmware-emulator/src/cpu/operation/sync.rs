use crate::{cpu::params::ERR_NONE, CPUEmulator};

#[repr(C, align(2))]
struct Sync {
    tag: u8,
}

impl CPUEmulator {
    pub(crate) fn synchronize(&mut self, data: &[u8]) -> u8 {
        let _d = Self::cast::<Sync>(data);

        self.synchronized = true;

        // Do nothing to sync

        ERR_NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_memory_layout() {
        assert_eq!(2, std::mem::size_of::<Sync>());
        assert_eq!(0, memoffset::offset_of!(Sync, tag));
    }
}
