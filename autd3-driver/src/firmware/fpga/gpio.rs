#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GPIOOut {
    O0 = 0,
    O1 = 1,
    O2 = 2,
    O3 = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GPIOIn {
    I0 = 0,
    I1 = 1,
    I2 = 2,
    I3 = 3,
}
