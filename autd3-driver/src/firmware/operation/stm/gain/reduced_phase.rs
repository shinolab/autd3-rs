use crate::firmware::fpga::Drive;

#[repr(C)]
pub(crate) struct PhaseFull<const N: usize> {
    phase_0: u8,
    phase_1: u8,
}

impl PhaseFull<0> {
    pub fn set(&mut self, d: Drive) {
        self.phase_0 = d.phase().value();
    }
}

impl PhaseFull<1> {
    pub fn set(&mut self, d: Drive) {
        self.phase_1 = d.phase().value();
    }
}

#[repr(C)]
pub(crate) struct PhaseHalf<const N: usize> {
    phase_01: u8,
    phase_23: u8,
}

impl PhaseHalf<0> {
    pub fn set(&mut self, d: Drive) {
        self.phase_01 = (self.phase_01 & 0xF0) | ((d.phase().value() >> 4) & 0x0F);
    }
}

impl PhaseHalf<1> {
    pub fn set(&mut self, d: Drive) {
        self.phase_01 = (self.phase_01 & 0x0F) | (d.phase().value() & 0xF0);
    }
}

impl PhaseHalf<2> {
    pub fn set(&mut self, d: Drive) {
        self.phase_23 = (self.phase_23 & 0xF0) | ((d.phase().value() >> 4) & 0x0F);
    }
}

impl PhaseHalf<3> {
    pub fn set(&mut self, d: Drive) {
        self.phase_23 = (self.phase_23 & 0x0F) | (d.phase().value() & 0xF0);
    }
}
