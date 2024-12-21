use autd3_driver::{
    derive::{LoopBehavior, Segment, TransitionMode},
    ethercat::DcSysTime,
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
            },
        )
    }

    pub fn modulation_cycle(&self, segment: Segment) -> usize {
        self.mem.controller_bram()[match segment {
            Segment::S0 => ADDR_MOD_CYCLE0,
            Segment::S1 => ADDR_MOD_CYCLE1,
        }] as usize
            + 1
    }

    pub fn modulation_loop_behavior(&self, segment: Segment) -> LoopBehavior {
        match Memory::read_bram_as::<u16>(
            &self.mem.controller_bram(),
            match segment {
                Segment::S0 => ADDR_MOD_REP0,
                Segment::S1 => ADDR_MOD_REP1,
            },
        ) {
            0xFFFF => LoopBehavior::infinite(),
            v => LoopBehavior::finite(v + 1).unwrap(),
        }
    }

    pub fn modulation(&self) -> u8 {
        self.modulation_at(self.current_mod_segment(), self.current_mod_idx())
    }

    pub fn modulation_at(&self, segment: Segment, idx: usize) -> u8 {
        let m = &self.mem.modulation_bram()[&segment][idx >> 1];
        let m = if idx % 2 == 0 { m & 0xFF } else { m >> 8 };
        m as u8
    }

    pub fn modulation_buffer(&self, segment: Segment) -> Vec<u8> {
        let mut dst = vec![0; self.modulation_cycle(segment)];
        self.modulation_buffer_inplace(segment, &mut dst);
        dst
    }

    pub fn modulation_buffer_inplace(&self, segment: Segment, dst: &mut [u8]) {
        (0..self.modulation_cycle(segment)).for_each(|i| dst[i] = self.modulation_at(segment, i));
    }

    pub fn modulation_transition_mode(&self) -> TransitionMode {
        match self.mem.controller_bram()[ADDR_MOD_TRANSITION_MODE] as u8 {
            TRANSITION_MODE_SYNC_IDX => TransitionMode::SyncIdx,
            TRANSITION_MODE_SYS_TIME => TransitionMode::SysTime(
                DcSysTime::ZERO
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
        fpga.mem
            .modulation_bram_mut()
            .get_mut(&Segment::S0)
            .unwrap()[0] = 0x1234;
        fpga.mem
            .modulation_bram_mut()
            .get_mut(&Segment::S0)
            .unwrap()[1] = 0x5678;
        fpga.mem.controller_bram_mut()[ADDR_MOD_CYCLE0] = 3 - 1;
        assert_eq!(3, fpga.modulation_cycle(Segment::S0));
        assert_eq!(0x34, fpga.modulation());
        assert_eq!(0x34, fpga.modulation_at(Segment::S0, 0));
        assert_eq!(0x12, fpga.modulation_at(Segment::S0, 1));
        assert_eq!(0x78, fpga.modulation_at(Segment::S0, 2));
        assert_eq!(vec![0x34, 0x12, 0x78], fpga.modulation_buffer(Segment::S0));
    }
}
