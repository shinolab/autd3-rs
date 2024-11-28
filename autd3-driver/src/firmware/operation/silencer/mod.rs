mod completion_steps;
mod update_rate;

#[derive(Clone, Copy, PartialEq, Debug, IntoBytes, Immutable)]
#[repr(C)]
pub struct SilencerControlFlags(u8);

bitflags::bitflags! {
    impl SilencerControlFlags : u8 {
        const NONE              = 0;
        const FIXED_UPDATE_RATE = 1 << 0;
        const PULSE_WIDTH       = 1 << 1;
        const STRICT_MODE       = 1 << 2;
    }
}

pub use completion_steps::SilencerFixedCompletionStepsOp;
pub use update_rate::SilencerFixedUpdateRateOp;
use zerocopy::{Immutable, IntoBytes};
