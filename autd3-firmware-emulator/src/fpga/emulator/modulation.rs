use autd3_driver::{
    derive::{LoopBehavior, Segment, TransitionMode},
    ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE},
    firmware::fpga::GPIOIn,
};

use super::{super::params::*, memory::Memory, FPGAEmulator};

impl FPGAEmulator {
    pub fn modulation_freq_division(&self, segment: Segment) -> u16 {
        Memory::read_bram_as::<u16>(
            &self.mem.controller_bram(),
            match segment {
                Segment::S0 => ADDR_MOD_FREQ_DIV0,
                Segment::S1 => ADDR_MOD_FREQ_DIV1,
                _ => unimplemented!(),
            },
        )
    }

    pub fn modulation_cycle(&self, segment: Segment) -> usize {
        self.mem.controller_bram()[match segment {
            Segment::S0 => ADDR_MOD_CYCLE0,
            Segment::S1 => ADDR_MOD_CYCLE1,
            _ => unimplemented!(),
        }] as usize
            + 1
    }

    pub fn modulation_loop_behavior(&self, segment: Segment) -> LoopBehavior {
        match Memory::read_bram_as::<u16>(
            &self.mem.controller_bram(),
            match segment {
                Segment::S0 => ADDR_MOD_REP0,
                Segment::S1 => ADDR_MOD_REP1,
                _ => unimplemented!(),
            },
        ) {
            0xFFFF => LoopBehavior::infinite(),
            v => LoopBehavior::finite(v + 1).unwrap(),
        }
    }

    pub fn modulation_at(&self, segment: Segment, idx: usize) -> u8 {
        let m = match segment {
            Segment::S0 => &self.mem.modulation_bram_0()[idx >> 1],
            Segment::S1 => &self.mem.modulation_bram_1()[idx >> 1],
            _ => unimplemented!(),
        };
        let m = if idx % 2 == 0 { m & 0xFF } else { m >> 8 };
        m as u8
    }

    pub fn modulation(&self, segment: Segment) -> Vec<u8> {
        (0..self.modulation_cycle(segment))
            .map(|i| self.modulation_at(segment, i))
            .collect()
    }

    pub fn modulation_transition_mode(&self) -> TransitionMode {
        match self.mem.controller_bram()[ADDR_MOD_TRANSITION_MODE] as u8 {
            TRANSITION_MODE_SYNC_IDX => TransitionMode::SyncIdx,
            TRANSITION_MODE_SYS_TIME => TransitionMode::SysTime(
                DcSysTime::from_utc(ECAT_DC_SYS_TIME_BASE).unwrap()
                    + std::time::Duration::from_nanos(Memory::read_bram_as::<u64>(
                        &self.mem.controller_bram(),
                        ADDR_MOD_TRANSITION_VALUE_0,
                    )),
            ),
            TRANSITION_MODE_GPIO => TransitionMode::GPIO(
                match Memory::read_bram_as::<u64>(
                    &self.mem.controller_bram(),
                    ADDR_MOD_TRANSITION_VALUE_0,
                ) {
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

    pub fn req_modulation_segment(&self) -> Segment {
        match self.mem.controller_bram()[ADDR_MOD_REQ_RD_SEGMENT] {
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
        fpga.mem.modulation_bram_0_mut()[0] = 0x1234;
        fpga.mem.modulation_bram_0_mut()[1] = 0x5678;
        fpga.mem.controller_bram_mut()[ADDR_MOD_CYCLE0] = 3 - 1;
        assert_eq!(3, fpga.modulation_cycle(Segment::S0));
        assert_eq!(0x34, fpga.modulation_at(Segment::S0, 0));
        assert_eq!(0x12, fpga.modulation_at(Segment::S0, 1));
        assert_eq!(0x78, fpga.modulation_at(Segment::S0, 2));
        let m = fpga.modulation(Segment::S0);
        assert_eq!(3, m.len());
        assert_eq!(0x34, m[0]);
        assert_eq!(0x12, m[1]);
        assert_eq!(0x78, m[2]);
    }
}