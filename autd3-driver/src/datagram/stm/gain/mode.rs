/// [`GainSTM`] transmission mode.
///
/// [`GainSTM`]: crate::datagram::GainSTM
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GainSTMMode {
    #[default]
    /// This mode uses both phase and intensity data.
    PhaseIntensityFull = 0,
    /// This mode uses only phase data.
    PhaseFull = 1,
    /// This mode uses only half-compressed phase data.
    PhaseHalf = 2,
}
