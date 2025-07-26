use std::collections::HashMap;

use autd3_core::firmware::{
    Segment,
    transition_mode::{Immediate, TransitionMode, TransitionModeParams},
};
use autd3_driver::{
    common::{Freq, Hz},
    ethercat::DcSysTime,
};

use crate::fpga::params::{
    TRANSITION_MODE_EXT, TRANSITION_MODE_GPIO, TRANSITION_MODE_SYNC_IDX, TRANSITION_MODE_SYS_TIME,
};

use super::FPGAEmulator;

const FPGA_MAIN_CLK_FREQ: u32 = 20480000;

pub(crate) struct Swapchain<const SET: u16> {
    sys_time: DcSysTime,
    pub(crate) fpga_clk_freq: Freq<u32>,
    rep: u16,
    start_lap: HashMap<Segment, usize>,
    freq_div: HashMap<Segment, u16>,
    cycle: HashMap<Segment, usize>,
    tic_idx_offset: HashMap<Segment, usize>,
    cur_segment: Segment,
    req_segment: Segment,
    cur_idx: usize,
    transition_params: TransitionModeParams,
    stop: bool,
    ext_mode: bool,
    ext_last_lap: usize,
    state: State,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    WaitStart,
    FiniteLoop,
    InfiniteLoop,
}

impl<const SET: u16> Swapchain<SET> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sys_time: DcSysTime::now(),
            fpga_clk_freq: FPGA_MAIN_CLK_FREQ * Hz,
            rep: 0,
            freq_div: [(Segment::S0, 10u16), (Segment::S1, 10u16)]
                .into_iter()
                .collect(),
            cycle: [(Segment::S0, 1), (Segment::S1, 1)].into_iter().collect(),
            tic_idx_offset: [(Segment::S0, 0), (Segment::S1, 0)].into_iter().collect(),
            start_lap: [(Segment::S0, 0), (Segment::S1, 0)].into_iter().collect(),
            cur_segment: Segment::S0,
            req_segment: Segment::S0,
            cur_idx: 0,
            transition_params: Immediate.params(),
            stop: false,
            ext_mode: false,
            ext_last_lap: 0,
            state: State::WaitStart,
        }
    }

    pub fn init(&mut self) {
        self.cur_segment = Segment::S0;
    }

    pub fn update(&mut self, gpio_in: [bool; 4], sys_time: DcSysTime) {
        let (last_lap, _) = self.lap_and_idx(self.req_segment, self.sys_time);
        let (lap, idx) = self.lap_and_idx(self.req_segment, sys_time);
        match self.state {
            State::WaitStart => match self.transition_params.mode {
                TRANSITION_MODE_SYNC_IDX => {
                    if last_lap < lap {
                        self.stop = false;
                        self.start_lap.insert(self.req_segment, lap);
                        self.tic_idx_offset.insert(self.req_segment, 0);
                        self.cur_segment = self.req_segment;
                        self.state = State::FiniteLoop;
                    }
                }
                TRANSITION_MODE_SYS_TIME => {
                    if self.transition_params.value <= sys_time.sys_time() {
                        self.stop = false;
                        self.start_lap.insert(self.req_segment, lap);
                        self.cur_segment = self.req_segment;
                        self.tic_idx_offset.insert(self.req_segment, idx);
                        self.state = State::FiniteLoop;
                    }
                }
                TRANSITION_MODE_GPIO => {
                    if gpio_in[self.transition_params.value as usize] {
                        self.stop = false;
                        self.start_lap.insert(self.req_segment, lap);
                        self.cur_segment = self.req_segment;
                        self.tic_idx_offset.insert(self.req_segment, idx);
                        self.state = State::FiniteLoop;
                    }
                }
                _ => unreachable!(),
            },
            State::FiniteLoop => {
                if (self.start_lap[&self.cur_segment] + self.rep as usize) + 1 < lap {
                    self.stop = true;
                }
                if ((self.start_lap[&self.cur_segment] + self.rep as usize) < lap)
                    && (self.tic_idx_offset[&self.cur_segment] <= idx)
                {
                    self.stop = true;
                }
            }
            State::InfiniteLoop => {
                if self.ext_mode && self.ext_last_lap < lap && self.ext_last_lap % 2 != lap % 2 {
                    self.ext_last_lap = lap;
                    self.cur_segment = match self.cur_segment {
                        Segment::S0 => Segment::S1,
                        Segment::S1 => Segment::S0,
                    };
                }
            }
        }
        let (_, idx) = self.lap_and_idx(self.cur_segment, sys_time);
        if self.stop {
            self.cur_idx = self.cycle[&self.cur_segment] - 1;
        } else {
            self.cur_idx = (idx + self.cycle[&self.cur_segment]
                - self.tic_idx_offset[&self.cur_segment])
                % self.cycle[&self.cur_segment];
        }
    }

    pub fn set(
        &mut self,
        sys_time: DcSysTime,
        rep: u16,
        freq_div: u16,
        cycle: usize,
        req_segment: Segment,
        transition_params: TransitionModeParams,
    ) {
        if self.cur_segment == req_segment {
            self.stop = false;
            self.ext_mode = transition_params.mode == TRANSITION_MODE_EXT;
            let (lap, _) = self.lap_and_idx(req_segment, sys_time);
            self.ext_last_lap = lap;
            self.tic_idx_offset.insert(req_segment, 0);
            self.state = State::InfiniteLoop;
        } else if rep == 0xFFFF {
            self.stop = false;
            self.cur_segment = req_segment;
            self.ext_mode = transition_params.mode == TRANSITION_MODE_EXT;
            let (lap, _) = self.lap_and_idx(req_segment, sys_time);
            self.ext_last_lap = lap;
            self.tic_idx_offset.insert(req_segment, 0);
            self.state = State::InfiniteLoop;
        } else {
            self.rep = rep;
            self.req_segment = req_segment;
            self.state = State::WaitStart;
        }
        self.sys_time = sys_time;

        self.freq_div.insert(req_segment, freq_div);
        self.cycle.insert(req_segment, cycle);
        self.transition_params = transition_params;
    }

    #[must_use]
    const fn fpga_sys_time(&self, dc_sys_time: DcSysTime) -> u64 {
        ((dc_sys_time.sys_time() as u128 * self.fpga_clk_freq.hz() as u128) / 1000000000) as _
    }

    #[must_use]
    fn lap_and_idx(&self, segment: Segment, sys_time: DcSysTime) -> (usize, usize) {
        let a = ((self.fpga_sys_time(sys_time) >> 9) / self.freq_div[&segment] as u64) as usize;
        let b = self.cycle[&segment];
        let lap = a / b;
        let idx = a % b;
        (lap, idx)
    }
}

impl FPGAEmulator {
    #[must_use]
    pub const fn current_mod_segment(&self) -> Segment {
        self.mod_swapchain.cur_segment
    }

    #[must_use]
    pub const fn current_stm_segment(&self) -> Segment {
        self.stm_swapchain.cur_segment
    }

    #[must_use]
    pub const fn current_mod_idx(&self) -> usize {
        self.mod_swapchain.cur_idx
    }

    #[must_use]
    pub const fn current_stm_idx(&self) -> usize {
        self.stm_swapchain.cur_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use autd3_core::firmware::{
        GPIOIn,
        transition_mode::{Ext, GPIO, Immediate, SyncIdx, SysTime},
    };

    const CYCLE_PERIOD: std::time::Duration = std::time::Duration::from_micros(25);
    const FREQ_DIV: u16 = 1;

    #[test]
    fn transition_same_segment() {
        let mut fpga = FPGAEmulator::new(249);

        assert_eq!(Segment::S0, fpga.current_mod_segment());

        let sys_time = DcSysTime::ZERO;
        fpga.mod_swapchain
            .set(sys_time, 0, FREQ_DIV, 1, Segment::S0, Immediate.params());

        assert!(!fpga.mod_swapchain.stop);
        assert!(!fpga.mod_swapchain.ext_mode);
        assert_eq!(Segment::S0, fpga.current_mod_segment());
        let (lap, _) = fpga.mod_swapchain.lap_and_idx(Segment::S0, sys_time);
        assert_eq!(lap, fpga.mod_swapchain.ext_last_lap);
        assert_eq!(0, fpga.mod_swapchain.tic_idx_offset[&Segment::S0]);
        assert_eq!(State::InfiniteLoop, fpga.mod_swapchain.state);
    }

    #[test]
    fn transition_infinite() {
        let mut fpga = FPGAEmulator::new(249);

        assert_eq!(Segment::S0, fpga.current_mod_segment());

        let sys_time = DcSysTime::ZERO;
        fpga.mod_swapchain
            .set(sys_time, 0xFFFF, FREQ_DIV, 1, Segment::S1, Ext.params());

        assert!(!fpga.mod_swapchain.stop);
        assert!(fpga.mod_swapchain.ext_mode);
        assert_eq!(Segment::S1, fpga.current_mod_segment());
        let (lap, _) = fpga.mod_swapchain.lap_and_idx(Segment::S0, sys_time);
        assert_eq!(lap, fpga.mod_swapchain.ext_last_lap);
        assert_eq!(0, fpga.mod_swapchain.rep);
        assert_eq!(0, fpga.mod_swapchain.tic_idx_offset[&Segment::S1]);
        assert_eq!(State::InfiniteLoop, fpga.mod_swapchain.state);
    }

    #[test]
    fn transition_finite() {
        let mut fpga = FPGAEmulator::new(249);

        assert_eq!(Segment::S0, fpga.current_mod_segment());

        let sys_time = DcSysTime::ZERO;
        fpga.mod_swapchain
            .set(sys_time, 0, FREQ_DIV, 1, Segment::S1, SyncIdx.params());

        assert!(!fpga.mod_swapchain.stop);
        assert!(!fpga.mod_swapchain.ext_mode);
        assert_eq!(Segment::S1, fpga.mod_swapchain.req_segment);
        assert_eq!(Segment::S0, fpga.current_mod_segment());
        assert_eq!(0, fpga.mod_swapchain.rep);
        assert_eq!(0, fpga.mod_swapchain.tic_idx_offset[&Segment::S1]);
        assert_eq!(State::WaitStart, fpga.mod_swapchain.state);
    }

    #[test]
    fn transition_sync_idx() {
        let mut fpga = FPGAEmulator::new(249);

        assert_eq!(Segment::S0, fpga.current_mod_segment());

        const CYCLE: u32 = 10;
        let sys_time = DcSysTime::ZERO + CYCLE_PERIOD * 5;
        fpga.mod_swapchain.set(
            sys_time,
            0,
            FREQ_DIV,
            CYCLE as _,
            Segment::S1,
            SyncIdx.params(),
        );

        fpga.mod_swapchain.update(
            [false; 4],
            sys_time + CYCLE_PERIOD * 5 - std::time::Duration::from_nanos(1),
        );
        assert!(!fpga.mod_swapchain.stop);
        assert_eq!(0, fpga.current_mod_idx());
        assert_eq!(0, fpga.mod_swapchain.start_lap[&Segment::S1]);
        assert_eq!(0, fpga.mod_swapchain.tic_idx_offset[&Segment::S1]);
        assert_eq!(Segment::S1, fpga.mod_swapchain.req_segment);
        assert_eq!(Segment::S0, fpga.current_mod_segment());
        assert_eq!(State::WaitStart, fpga.mod_swapchain.state);

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 5);
        assert!(!fpga.mod_swapchain.stop);
        assert_eq!(0, fpga.current_mod_idx());
        assert_eq!(Segment::S1, fpga.mod_swapchain.req_segment);
        assert_eq!(Segment::S1, fpga.current_mod_segment());
        assert_eq!(State::FiniteLoop, fpga.mod_swapchain.state);

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 5);
        assert_eq!(0, fpga.current_mod_idx());
        assert!(!fpga.mod_swapchain.stop);

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * (5 + CYCLE - 1));
        assert_eq!(9, fpga.current_mod_idx());
        assert!(!fpga.mod_swapchain.stop);

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * (5 + CYCLE));
        assert_eq!(9, fpga.current_mod_idx());
        assert!(fpga.mod_swapchain.stop);
    }

    #[test]
    fn transition_sys_time() {
        let mut fpga = FPGAEmulator::new(249);

        assert_eq!(Segment::S0, fpga.current_stm_segment());

        const CYCLE: u32 = 10;
        let sys_time = DcSysTime::ZERO + CYCLE_PERIOD * 5;
        fpga.stm_swapchain.set(
            sys_time,
            0,
            FREQ_DIV,
            CYCLE as _,
            Segment::S1,
            SysTime(sys_time + CYCLE_PERIOD * 5).params(),
        );

        fpga.stm_swapchain.update(
            [false; 4],
            sys_time + CYCLE_PERIOD * 5 - std::time::Duration::from_nanos(1),
        );
        assert!(!fpga.stm_swapchain.stop);
        assert_eq!(0, fpga.current_stm_idx());
        assert_eq!(0, fpga.stm_swapchain.start_lap[&Segment::S1]);
        assert_eq!(0, fpga.stm_swapchain.tic_idx_offset[&Segment::S1]);
        assert_eq!(Segment::S1, fpga.stm_swapchain.req_segment);
        assert_eq!(Segment::S0, fpga.current_stm_segment());
        assert_eq!(State::WaitStart, fpga.stm_swapchain.state);

        fpga.stm_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 5);
        assert!(!fpga.stm_swapchain.stop);
        assert_eq!(0, fpga.current_stm_idx());
        assert_eq!(0, fpga.stm_swapchain.tic_idx_offset[&Segment::S1]);
        assert_eq!(Segment::S1, fpga.stm_swapchain.req_segment);
        assert_eq!(Segment::S1, fpga.current_stm_segment());
        assert_eq!(State::FiniteLoop, fpga.stm_swapchain.state);

        fpga.stm_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 5);
        assert_eq!(0, fpga.current_stm_idx());
        assert!(!fpga.stm_swapchain.stop);

        fpga.stm_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * (5 + CYCLE - 1));
        assert_eq!(9, fpga.current_stm_idx());
        assert!(!fpga.stm_swapchain.stop);

        fpga.stm_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * (5 + CYCLE));
        assert_eq!(9, fpga.current_stm_idx());
        assert!(fpga.stm_swapchain.stop);
    }

    #[test]
    fn transition_gpio() {
        let mut fpga = FPGAEmulator::new(249);

        assert_eq!(Segment::S0, fpga.current_mod_segment());

        const CYCLE: u32 = 10;
        let sys_time = DcSysTime::ZERO + CYCLE_PERIOD * 5;
        fpga.mod_swapchain.set(
            sys_time,
            0,
            FREQ_DIV,
            CYCLE as _,
            Segment::S1,
            GPIO(GPIOIn::I0).params(),
        );

        fpga.mod_swapchain.update(
            [false; 4],
            sys_time + CYCLE_PERIOD * 5 - std::time::Duration::from_nanos(1),
        );
        assert!(!fpga.mod_swapchain.stop);
        assert_eq!(0, fpga.current_mod_idx());
        assert_eq!(0, fpga.mod_swapchain.start_lap[&Segment::S1]);
        assert_eq!(0, fpga.mod_swapchain.tic_idx_offset[&Segment::S1]);
        assert_eq!(Segment::S1, fpga.mod_swapchain.req_segment);
        assert_eq!(Segment::S0, fpga.current_mod_segment());
        assert_eq!(State::WaitStart, fpga.mod_swapchain.state);

        fpga.mod_swapchain
            .update([true, false, false, false], sys_time + CYCLE_PERIOD * 10);
        assert!(!fpga.mod_swapchain.stop);
        assert_eq!(0, fpga.current_mod_idx());
        assert_eq!(5, fpga.mod_swapchain.tic_idx_offset[&Segment::S1]);
        assert_eq!(Segment::S1, fpga.mod_swapchain.req_segment);
        assert_eq!(Segment::S1, fpga.current_mod_segment());
        assert_eq!(State::FiniteLoop, fpga.mod_swapchain.state);

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 10);
        assert_eq!(0, fpga.current_mod_idx());
        assert!(!fpga.mod_swapchain.stop);

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 11);
        assert_eq!(1, fpga.current_mod_idx());
        assert!(!fpga.mod_swapchain.stop);

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 19);
        assert_eq!(9, fpga.current_mod_idx());
        assert!(!fpga.mod_swapchain.stop);

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 20);
        assert_eq!(9, fpga.current_mod_idx());
        assert!(fpga.mod_swapchain.stop);
    }

    #[test]
    fn transition_gpio_over() {
        let mut fpga = FPGAEmulator::new(249);

        assert_eq!(Segment::S0, fpga.current_mod_segment());

        const CYCLE: u32 = 10;
        let sys_time = DcSysTime::ZERO + CYCLE_PERIOD * 5;
        fpga.mod_swapchain.set(
            sys_time,
            0,
            FREQ_DIV,
            CYCLE as _,
            Segment::S1,
            GPIO(GPIOIn::I0).params(),
        );

        fpga.mod_swapchain.update(
            [false; 4],
            sys_time + CYCLE_PERIOD * 5 - std::time::Duration::from_nanos(1),
        );
        assert!(!fpga.mod_swapchain.stop);
        assert_eq!(0, fpga.current_mod_idx());
        assert_eq!(0, fpga.mod_swapchain.start_lap[&Segment::S1]);
        assert_eq!(0, fpga.mod_swapchain.tic_idx_offset[&Segment::S1]);
        assert_eq!(Segment::S1, fpga.mod_swapchain.req_segment);
        assert_eq!(Segment::S0, fpga.current_mod_segment());
        assert_eq!(State::WaitStart, fpga.mod_swapchain.state);

        fpga.mod_swapchain
            .update([true, false, false, false], sys_time + CYCLE_PERIOD * 10);
        assert!(!fpga.mod_swapchain.stop);
        assert_eq!(0, fpga.current_mod_idx());
        assert_eq!(5, fpga.mod_swapchain.tic_idx_offset[&Segment::S1]);
        assert_eq!(Segment::S1, fpga.mod_swapchain.req_segment);
        assert_eq!(Segment::S1, fpga.current_mod_segment());
        assert_eq!(State::FiniteLoop, fpga.mod_swapchain.state);

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 30);
        assert_eq!(9, fpga.current_mod_idx());
        assert!(fpga.mod_swapchain.stop);
    }

    #[test]
    fn transition_ext() {
        let mut fpga = FPGAEmulator::new(249);

        assert_eq!(Segment::S0, fpga.current_mod_segment());

        const CYCLE: u32 = 10;
        let sys_time = DcSysTime::ZERO;
        fpga.mod_swapchain.set(
            sys_time,
            0xFFFF,
            FREQ_DIV,
            CYCLE as _,
            Segment::S0,
            Ext.params(),
        );
        fpga.mod_swapchain.set(
            sys_time,
            0xFFFF,
            FREQ_DIV,
            CYCLE as _,
            Segment::S1,
            Ext.params(),
        );

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 5);
        assert!(!fpga.mod_swapchain.stop);
        assert_eq!(5, fpga.current_mod_idx());
        assert_eq!(Segment::S1, fpga.current_mod_segment());

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 9);
        assert!(!fpga.mod_swapchain.stop);
        assert_eq!(9, fpga.current_mod_idx());
        assert_eq!(Segment::S1, fpga.current_mod_segment());

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 10);
        assert!(!fpga.mod_swapchain.stop);
        assert_eq!(0, fpga.current_mod_idx());
        assert_eq!(Segment::S0, fpga.current_mod_segment());

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 19);
        assert!(!fpga.mod_swapchain.stop);
        assert_eq!(9, fpga.current_mod_idx());
        assert_eq!(Segment::S0, fpga.current_mod_segment());

        fpga.mod_swapchain
            .update([false; 4], sys_time + CYCLE_PERIOD * 20);
        assert!(!fpga.mod_swapchain.stop);
        assert_eq!(0, fpga.current_mod_idx());
        assert_eq!(Segment::S1, fpga.current_mod_segment());
    }
}
