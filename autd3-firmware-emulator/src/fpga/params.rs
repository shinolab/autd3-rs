pub const VERSION_NUM: u8 = 0x90;
pub const VERSION_NUM_MINOR: u8 = 0x00;

pub const BRAM_SELECT_CONTROLLER: u8 = 0x0;
pub const BRAM_SELECT_MOD: u8 = 0x1;
pub const BRAM_SELECT_DUTY_TABLE: u8 = 0x2;
pub const BRAM_SELECT_STM: u8 = 0x3;

pub const BRAM_CNT_SEL_MAIN: usize = 0x00;
pub const BRAM_CNT_SEL_FILTER: usize = 0x01;

// pub const STM_MODE_FOCUS: u16 = 0x0;
pub const STM_MODE_GAIN: u16 = 0x1;

pub const SILNCER_MODE_FIXED_COMPLETION_STEPS: u16 = 0x0;
// pub const SILNCER_MODE_FIXED_UPDATE_RATE: u16 = 0x1;

pub const ADDR_CTL_FLAG: usize = 0x00;
pub const ADDR_FPGA_STATE: usize = 0x01;
// pub const ADDR_ECAT_SYNC_TIME_0: usize = 0x11;
// pub const ADDR_ECAT_SYNC_TIME_1: usize = ADDR_ECAT_SYNC_TIME_0 + 1;
// pub const ADDR_ECAT_SYNC_TIME_2: usize = ADDR_ECAT_SYNC_TIME_0 + 2;
// pub const ADDR_ECAT_SYNC_TIME_3: usize = ADDR_ECAT_SYNC_TIME_0 + 3;
pub const ADDR_MOD_MEM_WR_SEGMENT: usize = 0x20;
pub const ADDR_MOD_REQ_RD_SEGMENT: usize = 0x21;
pub const ADDR_MOD_CYCLE_0: usize = 0x22;
pub const ADDR_MOD_FREQ_DIV_0_0: usize = 0x23;
pub const ADDR_MOD_FREQ_DIV_0_1: usize = 0x24;
pub const ADDR_MOD_CYCLE_1: usize = 0x25;
pub const ADDR_MOD_FREQ_DIV_1_0: usize = 0x26;
pub const ADDR_MOD_FREQ_DIV_1_1: usize = 0x27;
pub const ADDR_MOD_REP_0_0: usize = 0x28;
pub const ADDR_MOD_REP_0_1: usize = 0x29;
pub const ADDR_MOD_REP_1_0: usize = 0x2A;
pub const ADDR_MOD_REP_1_1: usize = 0x2B;
pub const ADDR_VERSION_NUM_MAJOR: usize = 0x30;
pub const ADDR_VERSION_NUM_MINOR: usize = 0x31;
pub const ADDR_SILENCER_MODE: usize = 0x40;
pub const ADDR_SILENCER_UPDATE_RATE_INTENSITY: usize = 0x41;
pub const ADDR_SILENCER_UPDATE_RATE_PHASE: usize = 0x42;
pub const ADDR_SILENCER_COMPLETION_STEPS_INTENSITY: usize = 0x43;
pub const ADDR_SILENCER_COMPLETION_STEPS_PHASE: usize = 0x44;
pub const ADDR_STM_MEM_WR_SEGMENT: usize = 0x50;
pub const ADDR_STM_MEM_WR_PAGE: usize = 0x51;
pub const ADDR_STM_REQ_RD_SEGMENT: usize = 0x52;
pub const ADDR_STM_CYCLE_0: usize = 0x54;
pub const ADDR_STM_FREQ_DIV_0_0: usize = 0x55;
pub const ADDR_STM_FREQ_DIV_0_1: usize = 0x56;
pub const ADDR_STM_CYCLE_1: usize = 0x57;
pub const ADDR_STM_FREQ_DIV_1_0: usize = 0x58;
pub const ADDR_STM_FREQ_DIV_1_1: usize = 0x59;
pub const ADDR_STM_REP_0_0: usize = 0x5A;
pub const ADDR_STM_REP_0_1: usize = 0x5B;
pub const ADDR_STM_REP_1_0: usize = 0x5C;
pub const ADDR_STM_REP_1_1: usize = 0x5D;
pub const ADDR_STM_MODE_0: usize = 0x5E;
pub const ADDR_STM_MODE_1: usize = 0x5F;
pub const ADDR_STM_SOUND_SPEED_0_0: usize = 0x60;
pub const ADDR_STM_SOUND_SPEED_0_1: usize = 0x61;
pub const ADDR_STM_SOUND_SPEED_1_0: usize = 0x62;
pub const ADDR_STM_SOUND_SPEED_1_1: usize = 0x63;
pub const ADDR_PULSE_WIDTH_ENCODER_TABLE_WR_PAGE: usize = 0xE0;
pub const ADDR_PULSE_WIDTH_ENCODER_FULL_WIDTH_START: usize = 0xE1;
pub const BRAM_ADDR_DEBUG_TYPE_0: usize = 0xF0;
pub const BRAM_ADDR_DEBUG_VALUE_0: usize = 0xF1;
pub const BRAM_ADDR_DEBUG_TYPE_1: usize = 0xF2;
pub const BRAM_ADDR_DEBUG_VALUE_1: usize = 0xF3;
pub const BRAM_ADDR_DEBUG_TYPE_2: usize = 0xF4;
pub const BRAM_ADDR_DEBUG_VALUE_2: usize = 0xF5;
pub const BRAM_ADDR_DEBUG_TYPE_3: usize = 0xF6;
pub const BRAM_ADDR_DEBUG_VALUE_3: usize = 0xF7;

// pub const CTL_FLAG_MOD_SET_BIT: u8 = 0;
// pub const CTL_FLAG_STM_SET_BIT: u8 = 1;
// pub const CTL_FLAG_SILENCER_SET_BIT: u8 = 2;
// pub const CTL_FLAG_PULSE_WIDTH_ENCODER_SET_BIT: u8 = 3;
// pub const CTL_FLAG_DEBUG_SET_BIT: u8 = 4;
// pub const CTL_FLAG_SYNC_SET_BIT: u8 = 5;

pub const CTL_FLAG_FORCE_FAN_BIT: u8 = 13;

pub const ENABLED_EMULATOR_BIT: u8 = 0x80;
pub const ENABLED_FEATURES_BITS: u8 = ENABLED_EMULATOR_BIT;
