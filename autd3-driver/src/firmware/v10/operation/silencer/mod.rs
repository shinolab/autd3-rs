mod completion_steps;
mod completion_time;
mod update_rate;

pub(crate) use completion_steps::FixedCompletionStepsOp;
pub(crate) use completion_time::FixedCompletionTimeOp;
pub(crate) use update_rate::FixedUpdateRateOp;

use zerocopy::{Immutable, IntoBytes};

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
