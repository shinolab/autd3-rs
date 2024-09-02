pub const VERSION_NUM_MAJOR: u8 = 0xA2;
pub const VERSION_NUM_MINOR: u8 = 0x00;

pub const BRAM_SELECT_CONTROLLER: u8 = 0x0;
pub const BRAM_SELECT_MOD: u8 = 0x1;
pub const BRAM_SELECT_PWE_TABLE: u8 = 0x2;
pub const BRAM_SELECT_STM: u8 = 0x3;

pub const BRAM_CNT_SEL_MAIN: usize = 0x00;

pub const TRANSITION_MODE_SYNC_IDX: u8 = 0x00;
pub const TRANSITION_MODE_SYS_TIME: u8 = 0x01;
pub const TRANSITION_MODE_GPIO: u8 = 0x02;
pub const TRANSITION_MODE_EXT: u8 = 0xF0;
pub const TRANSITION_MODE_IMMEDIATE: u8 = 0xFF;

// pub const STM_MODE_FOCUS: u16 = 0x0;
pub const STM_MODE_GAIN: u16 = 0x1;

pub const SILENCER_FLAG_BIT_FIXED_UPDATE_RATE_MODE: u16 = 0;
pub const SILENCER_FLAG_FIXED_UPDATE_RATE_MODE: u16 = 1 << SILENCER_FLAG_BIT_FIXED_UPDATE_RATE_MODE;
pub const SILENCER_FLAG_BIT_PULSE_WIDTH: u16 = 1;
pub const SILENCER_FLAG_PULSE_WIDTH: u16 = 1 << SILENCER_FLAG_BIT_PULSE_WIDTH;

pub const ADDR_CTL_FLAG: usize = 0x00;
pub const ADDR_FPGA_STATE: usize = 0x01;
pub const ADDR_VERSION_NUM_MAJOR: usize = 0x02;
pub const ADDR_VERSION_NUM_MINOR: usize = 0x03;
pub const ADDR_ECAT_SYNC_TIME_0: usize = 0x10;
pub const ADDR_ECAT_SYNC_TIME_1: usize = 0x11;
pub const ADDR_ECAT_SYNC_TIME_2: usize = 0x12;
pub const ADDR_ECAT_SYNC_TIME_3: usize = 0x13;
pub const ADDR_MOD_MEM_WR_SEGMENT: usize = 0x20;
pub const ADDR_MOD_REQ_RD_SEGMENT: usize = 0x21;
pub const ADDR_MOD_CYCLE0: usize = 0x22;
pub const ADDR_MOD_FREQ_DIV0: usize = 0x23;
pub const ADDR_MOD_REP0: usize = 0x24;
pub const ADDR_MOD_CYCLE1: usize = 0x25;
pub const ADDR_MOD_FREQ_DIV1: usize = 0x26;
pub const ADDR_MOD_REP1: usize = 0x27;
pub const ADDR_MOD_TRANSITION_MODE: usize = 0x28;
pub const ADDR_MOD_TRANSITION_VALUE_0: usize = 0x29;
pub const ADDR_MOD_TRANSITION_VALUE_1: usize = 0x2A;
pub const ADDR_MOD_TRANSITION_VALUE_2: usize = 0x2B;
pub const ADDR_MOD_TRANSITION_VALUE_3: usize = 0x2C;
pub const ADDR_SILENCER_FLAG: usize = 0x40;
pub const ADDR_SILENCER_UPDATE_RATE_INTENSITY: usize = 0x41;
pub const ADDR_SILENCER_UPDATE_RATE_PHASE: usize = 0x42;
pub const ADDR_SILENCER_COMPLETION_STEPS_INTENSITY: usize = 0x43;
pub const ADDR_SILENCER_COMPLETION_STEPS_PHASE: usize = 0x44;
pub const ADDR_STM_MEM_WR_SEGMENT: usize = 0x50;
pub const ADDR_STM_MEM_WR_PAGE: usize = 0x51;
pub const ADDR_STM_REQ_RD_SEGMENT: usize = 0x52;
pub const ADDR_STM_CYCLE0: usize = 0x53;
pub const ADDR_STM_FREQ_DIV0: usize = 0x54;
pub const ADDR_STM_REP0: usize = 0x55;
pub const ADDR_STM_MODE0: usize = 0x56;
pub const ADDR_STM_SOUND_SPEED0: usize = 0x57;
pub const ADDR_STM_NUM_FOCI0: usize = 0x58;
pub const ADDR_STM_CYCLE1: usize = 0x59;
pub const ADDR_STM_FREQ_DIV1: usize = 0x5A;
pub const ADDR_STM_REP1: usize = 0x5B;
pub const ADDR_STM_MODE1: usize = 0x5C;
pub const ADDR_STM_SOUND_SPEED1: usize = 0x5D;
pub const ADDR_STM_NUM_FOCI1: usize = 0x5E;
pub const ADDR_STM_TRANSITION_MODE: usize = 0x5F;
pub const ADDR_STM_TRANSITION_VALUE_0: usize = 0x60;
pub const ADDR_STM_TRANSITION_VALUE_1: usize = 0x61;
pub const ADDR_STM_TRANSITION_VALUE_2: usize = 0x62;
pub const ADDR_STM_TRANSITION_VALUE_3: usize = 0x63;
pub const ADDR_DEBUG_TYPE0: usize = 0xF0;
pub const ADDR_DEBUG_VALUE0: usize = 0xF1;
pub const ADDR_DEBUG_TYPE1: usize = 0xF2;
pub const ADDR_DEBUG_VALUE1: usize = 0xF3;
pub const ADDR_DEBUG_TYPE2: usize = 0xF4;
pub const ADDR_DEBUG_VALUE2: usize = 0xF5;
pub const ADDR_DEBUG_TYPE3: usize = 0xF6;
pub const ADDR_DEBUG_VALUE3: usize = 0xF7;

pub const CTL_FLAG_MOD_SET_BIT: u16 = 0;
pub const CTL_FLAG_STM_SET_BIT: u16 = 1;
// pub const CTL_FLAG_SILENCER_SET_BIT: u8 = 2;
// pub const CTL_FLAG_PULSE_WIDTH_ENCODER_SET_BIT: u8 = 3;
// pub const CTL_FLAG_DEBUG_SET_BIT: u8 = 4;
// pub const CTL_FLAG_SYNC_SET_BIT: u8 = 5;

pub const CTL_FLAG_BIT_GPIO_IN_0: u8 = 8;
pub const CTL_FLAG_BIT_GPIO_IN_1: u8 = 9;
pub const CTL_FLAG_BIT_GPIO_IN_2: u8 = 10;
pub const CTL_FLAG_BIT_GPIO_IN_3: u8 = 11;
pub const CTL_FLAG_FORCE_FAN_BIT: u8 = 13;

pub const ENABLED_EMULATOR_BIT: u8 = 0x80;
pub const ENABLED_FEATURES_BITS: u8 = ENABLED_EMULATOR_BIT;

pub const DBG_NONE: u8 = 0x00;
pub const DBG_BASE_SIG: u8 = 0x01;
pub const DBG_THERMO: u8 = 0x02;
pub const DBG_FORCE_FAN: u8 = 0x03;
pub const DBG_SYNC: u8 = 0x10;
pub const DBG_MOD_SEGMENT: u8 = 0x20;
pub const DBG_MOD_IDX: u8 = 0x21;
pub const DBG_STM_SEGMENT: u8 = 0x50;
pub const DBG_STM_IDX: u8 = 0x51;
pub const DBG_IS_STM_MODE: u8 = 0x52;
pub const DBG_PWM_OUT: u8 = 0xE0;
pub const DBG_DIRECT: u8 = 0xF0;
