/// Silencer target.
#[derive(Debug, Clone, Copy, PartialEq, Default, Eq)]
#[repr(u8)]
pub enum SilencerTarget {
    /// Apply the silencer to the intensity (before [`PulseWidthEncoder`]).
    ///
    /// [`PulseWidthEncoder`]: crate::datagram::PulseWidthEncoder
    #[default]
    Intensity = 0,
    /// Apply the silencer to the pulse width (after [`PulseWidthEncoder`]).
    ///
    /// [`PulseWidthEncoder`]: crate::datagram::PulseWidthEncoder
    PulseWidth = 1,
}
