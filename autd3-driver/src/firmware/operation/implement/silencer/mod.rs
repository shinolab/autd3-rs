mod completion_steps;
mod completion_time;
mod update_rate;

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct SilencerControlFlags(pub(crate) u8);

impl SilencerControlFlags {
    const NONE: SilencerControlFlags = SilencerControlFlags(0);
    const FIXED_UPDATE_RATE: SilencerControlFlags = SilencerControlFlags(1 << 0);
    const STRICT_MODE: SilencerControlFlags = SilencerControlFlags(1 << 2);
}
