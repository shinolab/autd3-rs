pub const NANOSECONDS: u64 = 1;
pub const MICROSECONDS: u64 = NANOSECONDS * 1000;
pub const MILLISECONDS: u64 = MICROSECONDS * 1000;
pub const SYS_TIME_TRANSITION_MARGIN: u64 = 10 * MILLISECONDS;

pub const CPU_VERSION_MAJOR: u16 = 0xA3;
pub const CPU_VERSION_MINOR: u16 = 0x00;

pub const TRANS_NUM: usize = 249;

pub const BRAM_SELECT_CONTROLLER: u8 = 0x0;
pub const BRAM_SELECT_MOD: u8 = 0x1;
pub const BRAM_SELECT_PWE_TABLE: u8 = 0x2;
pub const BRAM_SELECT_STM: u8 = 0x3;

pub const BRAM_CNT_SEL_MAIN: u8 = 0x00;
pub const BRAM_CNT_SEL_PHASE_CORR: u8 = 0x01;

pub const TRANSITION_MODE_SYNC_IDX: u8 = 0x00;
pub const TRANSITION_MODE_SYS_TIME: u8 = 0x01;
pub const TRANSITION_MODE_GPIO: u8 = 0x02;
pub const TRANSITION_MODE_EXT: u8 = 0xF0;
pub const TRANSITION_MODE_NONE: u8 = 0xFE;
pub const TRANSITION_MODE_IMMEDIATE: u8 = 0xFF;

pub const ADDR_CTL_FLAG: u16 = 0x00;
pub const ADDR_FPGA_STATE: u16 = 0x01;
pub const ADDR_VERSION_NUM_MAJOR: u16 = 0x02;
pub const ADDR_VERSION_NUM_MINOR: u16 = 0x03;
pub const ADDR_ECAT_SYNC_TIME_0: u16 = 0x10;
pub const ADDR_ECAT_SYNC_TIME_1: u16 = 0x11;
pub const ADDR_ECAT_SYNC_TIME_2: u16 = 0x12;
pub const ADDR_ECAT_SYNC_TIME_3: u16 = 0x13;
pub const ADDR_MOD_MEM_WR_SEGMENT: u16 = 0x20;
pub const ADDR_MOD_MEM_WR_PAGE: u16 = 0x21;
pub const ADDR_MOD_REQ_RD_SEGMENT: u16 = 0x22;
pub const ADDR_MOD_CYCLE0: u16 = 0x23;
pub const ADDR_MOD_CYCLE1: u16 = 0x24;
pub const ADDR_MOD_FREQ_DIV0: u16 = 0x25;
pub const ADDR_MOD_FREQ_DIV1: u16 = 0x26;
pub const ADDR_MOD_REP0: u16 = 0x27;
pub const ADDR_MOD_REP1: u16 = 0x28;
pub const ADDR_MOD_TRANSITION_MODE: u16 = 0x29;
pub const ADDR_MOD_TRANSITION_VALUE_0: u16 = 0x2A;
pub const ADDR_MOD_TRANSITION_VALUE_1: u16 = 0x2B;
pub const ADDR_MOD_TRANSITION_VALUE_2: u16 = 0x2C;
pub const ADDR_MOD_TRANSITION_VALUE_3: u16 = 0x2D;
pub const ADDR_SILENCER_FLAG: u16 = 0x40;
pub const ADDR_SILENCER_UPDATE_RATE_INTENSITY: u16 = 0x41;
pub const ADDR_SILENCER_UPDATE_RATE_PHASE: u16 = 0x42;
pub const ADDR_SILENCER_COMPLETION_STEPS_INTENSITY: u16 = 0x43;
pub const ADDR_SILENCER_COMPLETION_STEPS_PHASE: u16 = 0x44;
pub const ADDR_STM_MEM_WR_SEGMENT: u16 = 0x50;
pub const ADDR_STM_MEM_WR_PAGE: u16 = 0x51;
pub const ADDR_STM_REQ_RD_SEGMENT: u16 = 0x52;
pub const ADDR_STM_CYCLE0: u16 = 0x53;
pub const ADDR_STM_CYCLE1: u16 = 0x54;
pub const ADDR_STM_FREQ_DIV0: u16 = 0x55;
pub const ADDR_STM_FREQ_DIV1: u16 = 0x56;
pub const ADDR_STM_REP0: u16 = 0x57;
pub const ADDR_STM_REP1: u16 = 0x58;
pub const ADDR_STM_MODE0: u16 = 0x59;
pub const ADDR_STM_MODE1: u16 = 0x5A;
pub const ADDR_STM_SOUND_SPEED0: u16 = 0x5B;
pub const ADDR_STM_SOUND_SPEED1: u16 = 0x5C;
pub const ADDR_STM_NUM_FOCI0: u16 = 0x5D;
pub const ADDR_STM_NUM_FOCI1: u16 = 0x5E;
pub const ADDR_STM_TRANSITION_MODE: u16 = 0x5F;
pub const ADDR_STM_TRANSITION_VALUE_0: u16 = 0x60;
pub const ADDR_STM_TRANSITION_VALUE_1: u16 = 0x61;
pub const ADDR_STM_TRANSITION_VALUE_2: u16 = 0x62;
pub const ADDR_STM_TRANSITION_VALUE_3: u16 = 0x63;
pub const ADDR_DEBUG_VALUE0_0: u16 = 0xF0;
pub const ADDR_DEBUG_VALUE0_1: u16 = 0xF1;
pub const ADDR_DEBUG_VALUE0_2: u16 = 0xF2;
pub const ADDR_DEBUG_VALUE0_3: u16 = 0xF3;
pub const ADDR_DEBUG_VALUE1_0: u16 = 0xF4;
pub const ADDR_DEBUG_VALUE1_1: u16 = 0xF5;
pub const ADDR_DEBUG_VALUE1_2: u16 = 0xF6;
pub const ADDR_DEBUG_VALUE1_3: u16 = 0xF7;
pub const ADDR_DEBUG_VALUE2_0: u16 = 0xF8;
pub const ADDR_DEBUG_VALUE2_1: u16 = 0xF9;
pub const ADDR_DEBUG_VALUE2_2: u16 = 0xFA;
pub const ADDR_DEBUG_VALUE2_3: u16 = 0xFB;
pub const ADDR_DEBUG_VALUE3_0: u16 = 0xFC;
pub const ADDR_DEBUG_VALUE3_1: u16 = 0xFD;
pub const ADDR_DEBUG_VALUE3_2: u16 = 0xFE;
pub const ADDR_DEBUG_VALUE3_3: u16 = 0xFF;

pub const CTL_FLAG_MOD_SET_BIT: u16 = 0;
pub const CTL_FLAG_STM_SET_BIT: u16 = 1;
pub const CTL_FLAG_SILENCER_SET_BIT: u16 = 2;
pub const CTL_FLAG_DEBUG_SET_BIT: u16 = 4;
pub const CTL_FLAG_SYNC_SET_BIT: u16 = 5;
pub const CTL_FLAG_BIT_GPIO_IN_0: u16 = 8;
pub const CTL_FLAG_BIT_GPIO_IN_1: u16 = 9;
pub const CTL_FLAG_BIT_GPIO_IN_2: u16 = 10;
pub const CTL_FLAG_BIT_GPIO_IN_3: u16 = 11;
pub const CTL_FLAG_FORCE_FAN_BIT: u16 = 13;

pub const CTL_FLAG_MOD_SET: u16 = 1 << CTL_FLAG_MOD_SET_BIT;
pub const CTL_FLAG_STM_SET: u16 = 1 << CTL_FLAG_STM_SET_BIT;
pub const CTL_FLAG_SILENCER_SET: u16 = 1 << CTL_FLAG_SILENCER_SET_BIT;
pub const CTL_FLAG_DEBUG_SET: u16 = 1 << CTL_FLAG_DEBUG_SET_BIT;
pub const CTL_FLAG_SYNC_SET: u16 = 1 << CTL_FLAG_SYNC_SET_BIT;
pub const CTL_FLAG_GPIO_IN_0: u16 = 1 << CTL_FLAG_BIT_GPIO_IN_0;
pub const CTL_FLAG_GPIO_IN_1: u16 = 1 << CTL_FLAG_BIT_GPIO_IN_1;
pub const CTL_FLAG_GPIO_IN_2: u16 = 1 << CTL_FLAG_BIT_GPIO_IN_2;
pub const CTL_FLAG_GPIO_IN_3: u16 = 1 << CTL_FLAG_BIT_GPIO_IN_3;
pub const CTL_FLAG_FORCE_FAN: u16 = 1 << CTL_FLAG_FORCE_FAN_BIT;

pub const FPGA_STATE_BIT_READS_FPGA_STATE_ENABLED: u8 = 7;
pub const FPGA_STATE_READS_FPGA_STATE_ENABLED: u8 = 1 << FPGA_STATE_BIT_READS_FPGA_STATE_ENABLED;

pub const STM_MODE_FOCUS: u16 = 0;
pub const STM_MODE_GAIN: u16 = 1;

pub const SILENCER_FLAG_BIT_FIXED_UPDATE_RATE_MODE: u8 = 0;
pub const SILENCER_FLAG_FIXED_UPDATE_RATE_MODE: u8 = 1 << SILENCER_FLAG_BIT_FIXED_UPDATE_RATE_MODE;
pub const SILENCER_FLAG_STRICT_MODE: u8 = 1 << 2;

pub const TAG_NOP: u8 = 0x00;
pub const TAG_CLEAR: u8 = 0x01;
pub const TAG_SYNC: u8 = 0x02;
pub const TAG_FIRM_INFO: u8 = 0x03;
pub const TAG_CONFIG_FPGA_CLK: u8 = 0x04;
pub const TAG_MODULATION: u8 = 0x10;
pub const TAG_MODULATION_CHANGE_SEGMENT: u8 = 0x11;
pub const TAG_SILENCER: u8 = 0x21;
pub const TAG_GAIN: u8 = 0x30;
pub const TAG_GAIN_CHANGE_SEGMENT: u8 = 0x31;
pub const TAG_GAIN_STM: u8 = 0x41;
pub const TAG_FOCI_STM: u8 = 0x42;
pub const TAG_GAIN_STM_CHANGE_SEGMENT: u8 = 0x43;
pub const TAG_FOCI_STM_CHANGE_SEGMENT: u8 = 0x44;
pub const TAG_FORCE_FAN: u8 = 0x60;
pub const TAG_READS_FPGA_STATE: u8 = 0x61;
pub const TAG_CONFIG_PULSE_WIDTH_ENCODER: u8 = 0x72;
pub const TAG_PHASE_CORRECTION: u8 = 0x80;
pub const TAG_DEBUG: u8 = 0xF0;
pub const TAG_EMULATE_GPIO_IN: u8 = 0xF1;
pub const TAG_CPU_GPIO_OUT: u8 = 0xF2;

pub const INFO_TYPE_CPU_VERSION_MAJOR: u8 = 0x01;
pub const INFO_TYPE_CPU_VERSION_MINOR: u8 = 0x02;
pub const INFO_TYPE_FPGA_VERSION_MAJOR: u8 = 0x03;
pub const INFO_TYPE_FPGA_VERSION_MINOR: u8 = 0x04;
pub const INFO_TYPE_FPGA_FUNCTIONS: u8 = 0x05;
pub const INFO_TYPE_CLEAR: u8 = 0x06;

pub const GAIN_FLAG_UPDATE: u8 = 1 << 0;

pub const MODULATION_FLAG_BEGIN: u8 = 1 << 0;
pub const MODULATION_FLAG_END: u8 = 1 << 1;
pub const MODULATION_FLAG_UPDATE: u8 = 1 << 2;
pub const MODULATION_FLAG_SEGMENT: u8 = 1 << 3;

pub const FOCI_STM_FLAG_BEGIN: u8 = 1 << 0;
pub const FOCI_STM_FLAG_END: u8 = 1 << 1;
pub const FOCI_STM_FLAG_UPDATE: u8 = 1 << 2;

pub const GAIN_STM_FLAG_BEGIN: u8 = 1 << 0;
pub const GAIN_STM_FLAG_END: u8 = 1 << 1;
pub const GAIN_STM_FLAG_UPDATE: u8 = 1 << 2;
pub const GAIN_STM_FLAG_SEGMENT: u8 = 1 << 3;

pub const GAIN_STM_MODE_INTENSITY_PHASE_FULL: u8 = 0;
pub const GAIN_STM_MODE_PHASE_FULL: u8 = 1;
pub const GAIN_STM_MODE_PHASE_HALF: u8 = 2;

pub const CLK_FLAG_BEGIN: u8 = 1 << 0;
pub const CLK_FLAG_END: u8 = 1 << 1;

pub const GPIO_IN_FLAG_0: u8 = 1 << 0;
pub const GPIO_IN_FLAG_1: u8 = 1 << 1;
pub const GPIO_IN_FLAG_2: u8 = 1 << 2;
pub const GPIO_IN_FLAG_3: u8 = 1 << 3;

pub const NO_ERR: u8 = 0x00;
#[allow(clippy::identity_op)]
pub const ERR_NOT_SUPPORTED_TAG: u8 = 0x01;
pub const ERR_INVALID_MSG_ID: u8 = 0x02;
pub const ERR_INVALID_INFO_TYPE: u8 = 0x03;
pub const ERR_INVALID_GAIN_STM_MODE: u8 = 0x04;
pub const ERR_INVALID_SEGMENT_TRANSITION: u8 = 0x05;
pub const ERR_MISS_TRANSITION_TIME: u8 = 0x06;
pub const ERR_INVALID_SILENCER_SETTING: u8 = 0x07;
pub const ERR_INVALID_TRANSITION_MODE: u8 = 0x08;
