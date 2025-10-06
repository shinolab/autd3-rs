#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
#[non_exhaustive]
pub(crate) enum TypeTag {
    Nop = 0x00,
    Clear = 0x01,
    Sync = 0x02,
    FirmwareVersion = 0x03,
    Modulation = 0x10,
    ModulationSwapSegment = 0x11,
    Silencer = 0x21,
    Gain = 0x30,
    GainSwapSegment = 0x31,
    GainSTM = 0x41,
    FociSTM = 0x42,
    GainSTMSwapSegment = 0x43,
    FociSTMSwapSegment = 0x44,
    ForceFan = 0x60,
    ReadsFPGAState = 0x61,
    ConfigPulseWidthEncoder = 0x72,
    PhaseCorrection = 0x80,
    OutputMask = 0x90,
    Debug = 0xF0,
    EmulateGPIOIn = 0xF1,
    CpuGPIOOut = 0xF2,
}
