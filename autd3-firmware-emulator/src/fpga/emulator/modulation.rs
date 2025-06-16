use std::num::NonZeroU16;

use autd3_core::datagram::{GPIOIn, LoopBehavior, Segment, TransitionMode};
use autd3_driver::ethercat::DcSysTime;

use super::{super::params::*, FPGAEmulator};

impl FPGAEmulator {
    #[must_use]
    pub fn modulation_freq_divide(&self, segment: Segment) -> u16 {
        self.mem
            .controller_bram
            .read(ADDR_MOD_FREQ_DIV0 + segment as usize)
    }

    #[must_use]
    pub fn modulation_cycle(&self, segment: Segment) -> usize {
        self.mem
            .controller_bram
            .read(ADDR_MOD_CYCLE0 + segment as usize) as usize
            + 1
    }

    #[must_use]
    pub fn modulation_loop_behavior(&self, segment: Segment) -> LoopBehavior {
        match &self
            .mem
            .controller_bram
            .read(ADDR_MOD_REP0 + segment as usize)
        {
            0xFFFF => LoopBehavior::Infinite,
            v => LoopBehavior::Finite(NonZeroU16::new(v + 1).unwrap()),
        }
    }

    #[must_use]
    pub fn modulation(&self) -> u8 {
        self.modulation_at(self.current_mod_segment(), self.current_mod_idx())
    }

    #[must_use]
    pub fn modulation_at(&self, segment: Segment, idx: usize) -> u8 {
        let m = &self.mem.modulation_bram[&segment].read(idx >> 1);
        let m = if idx % 2 == 0 { m & 0xFF } else { m >> 8 };
        m as u8
    }

    #[must_use]
    pub fn modulation_buffer(&self, segment: Segment) -> Vec<u8> {
        let mut dst = vec![0; self.modulation_cycle(segment)];
        self.modulation_buffer_inplace(segment, &mut dst);
        dst
    }

    pub fn modulation_buffer_inplace(&self, segment: Segment, dst: &mut [u8]) {
        (0..self.modulation_cycle(segment)).for_each(|i| dst[i] = self.modulation_at(segment, i));
    }

    #[must_use]
    pub fn modulation_transition_mode(&self) -> TransitionMode {
        match self.mem.controller_bram.read(ADDR_MOD_TRANSITION_MODE) as u8 {
            TRANSITION_MODE_SYNC_IDX => TransitionMode::SyncIdx,
            TRANSITION_MODE_SYS_TIME => TransitionMode::SysTime(
                DcSysTime::ZERO
                    + std::time::Duration::from_nanos(
                        self.mem
                            .controller_bram
                            .read_bram_as::<u64>(ADDR_MOD_TRANSITION_VALUE_0),
                    ),
            ),
            TRANSITION_MODE_GPIO => TransitionMode::GPIO(
                match self
                    .mem
                    .controller_bram
                    .read_bram_as::<u64>(ADDR_MOD_TRANSITION_VALUE_0)
                {
                    0 => GPIOIn::I0,
                    1 => GPIOIn::I1,
                    2 => GPIOIn::I2,
                    3 => GPIOIn::I3,
                    _ => unreachable!(),
                },
            ),
            TRANSITION_MODE_EXT => TransitionMode::Ext,
            TRANSITION_MODE_IMMEDIATE => TransitionMode::Immediate,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn req_modulation_segment(&self) -> Segment {
        match self.mem.controller_bram.read(ADDR_MOD_REQ_RD_SEGMENT) {
            0 => Segment::S0,
            1 => Segment::S1,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modulation() {
        let fpga = FPGAEmulator::new(249);
        fpga.mem.modulation_bram[&Segment::S0].write(0, 0x1234);
        fpga.mem.modulation_bram[&Segment::S0].write(1, 0x5678);
        fpga.mem.controller_bram.write(ADDR_MOD_CYCLE0, 3 - 1);
        assert_eq!(3, fpga.modulation_cycle(Segment::S0));
        assert_eq!(0x34, fpga.modulation());
        assert_eq!(0x34, fpga.modulation_at(Segment::S0, 0));
        assert_eq!(0x12, fpga.modulation_at(Segment::S0, 1));
        assert_eq!(0x78, fpga.modulation_at(Segment::S0, 2));
        assert_eq!(vec![0x34, 0x12, 0x78], fpga.modulation_buffer(Segment::S0));
    }
}
