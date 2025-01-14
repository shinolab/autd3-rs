/// GPIO output pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GPIOOut {
    /// Output 0
    O0 = 0,
    /// Output 1
    O1 = 1,
    /// Output 2
    O2 = 2,
    /// Output 3
    O3 = 3,
}

/// GPIO input pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GPIOIn {
    /// Input 0
    I0 = 0,
    /// Input 1
    I1 = 1,
    /// Input 2
    I2 = 2,
    /// Input 3
    I3 = 3,
}
