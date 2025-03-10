// GRCOV_EXCL_START
#[must_use]
pub(crate) const fn round_frac(decimal: u64, precision: u64) -> u64 {
    if decimal & (1 << (10 - precision)) != 0 {
        decimal + (1 << (10 - precision))
    } else {
        decimal
    }
}

#[must_use]
pub(crate) const fn mmcm_divider(divide: u64) -> u64 {
    let mut duty_cycle = 50000;
    if divide >= 64 {
        let duty_cycle_min = ((divide - 64) * 100_000) / divide;
        let duty_cycle_max = ((64.5 / divide as f32) * 100000.) as u64;
        if duty_cycle > duty_cycle_max {
            duty_cycle = duty_cycle_max;
        }
        if duty_cycle < duty_cycle_min {
            duty_cycle = duty_cycle_min;
        }
    }

    let duty_cycle_fix = (duty_cycle << 10) / 100_000;

    let mut high_time: u64;
    let low_time: u64;
    let no_count: u64;
    let mut w_edge: u64;
    if divide == 1 {
        high_time = 1;
        w_edge = 0;
        low_time = 1;
        no_count = 1;
    } else {
        let temp = round_frac(duty_cycle_fix * divide, 1);

        high_time = (temp & 0b111111100000000000) >> 11;
        w_edge = (temp & 0b10000000000) >> 10;

        if high_time == 0 {
            high_time = 1;
            w_edge = 0;
        }

        if high_time == divide {
            high_time = divide - 1;
            w_edge = 1;
        }

        low_time = divide - high_time;
        no_count = 0;
    };

    (w_edge << 13) | (no_count << 12) | ((high_time & 0b111111) << 6) | (low_time & 0b111111)
}

#[must_use]
pub(crate) const fn mmcm_count_calc(divide: u64) -> u64 {
    let div_calc = mmcm_divider(divide);
    let phase_calc = 0;

    ((phase_calc & 0b11000000000) << 15)
        | ((div_calc & 0b11000000000000) << 10)
        | ((phase_calc & 0b111111) << 16)
        | ((phase_calc & 0b111000000) << 13)
        | (div_calc & 0b111111111111)
}

#[must_use]
pub(crate) const fn mmcm_lock_lookup(divide: u64) -> u64 {
    let lookup: [u64; 64] = [
        0b0011_0001_1011_1110_1000_1111_1010_0100_0000_0001,
        0b0011_0001_1011_1110_1000_1111_1010_0100_0000_0001,
        0b0100_0010_0011_1110_1000_1111_1010_0100_0000_0001,
        0b0101_1010_1111_1110_1000_1111_1010_0100_0000_0001,
        0b0111_0011_1011_1110_1000_1111_1010_0100_0000_0001,
        0b1000_1100_0111_1110_1000_1111_1010_0100_0000_0001,
        0b1001_1100_1111_1110_1000_1111_1010_0100_0000_0001,
        0b1011_0101_1011_1110_1000_1111_1010_0100_0000_0001,
        0b1100_1110_0111_1110_1000_1111_1010_0100_0000_0001,
        0b1110_0111_0011_1110_1000_1111_1010_0100_0000_0001,
        0b1111_1111_1111_1000_0100_1111_1010_0100_0000_0001,
        0b1111_1111_1111_0011_1001_1111_1010_0100_0000_0001,
        0b1111_1111_1110_1110_1110_1111_1010_0100_0000_0001,
        0b1111_1111_1110_1011_1100_1111_1010_0100_0000_0001,
        0b1111_1111_1110_1000_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1110_0111_0001_1111_1010_0100_0000_0001,
        0b1111_1111_1110_0011_1111_1111_1010_0100_0000_0001,
        0b1111_1111_1110_0010_0110_1111_1010_0100_0000_0001,
        0b1111_1111_1110_0000_1101_1111_1010_0100_0000_0001,
        0b1111_1111_1101_1111_0100_1111_1010_0100_0000_0001,
        0b1111_1111_1101_1101_1011_1111_1010_0100_0000_0001,
        0b1111_1111_1101_1100_0010_1111_1010_0100_0000_0001,
        0b1111_1111_1101_1010_1001_1111_1010_0100_0000_0001,
        0b1111_1111_1101_1001_0000_1111_1010_0100_0000_0001,
        0b1111_1111_1101_1001_0000_1111_1010_0100_0000_0001,
        0b1111_1111_1101_0111_0111_1111_1010_0100_0000_0001,
        0b1111_1111_1101_0101_1110_1111_1010_0100_0000_0001,
        0b1111_1111_1101_0101_1110_1111_1010_0100_0000_0001,
        0b1111_1111_1101_0100_0101_1111_1010_0100_0000_0001,
        0b1111_1111_1101_0100_0101_1111_1010_0100_0000_0001,
        0b1111_1111_1101_0010_1100_1111_1010_0100_0000_0001,
        0b1111_1111_1101_0010_1100_1111_1010_0100_0000_0001,
        0b1111_1111_1101_0010_1100_1111_1010_0100_0000_0001,
        0b1111_1111_1101_0001_0011_1111_1010_0100_0000_0001,
        0b1111_1111_1101_0001_0011_1111_1010_0100_0000_0001,
        0b1111_1111_1101_0001_0011_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
        0b1111_1111_1100_1111_1010_1111_1010_0100_0000_0001,
    ];
    lookup[divide as usize - 1]
}

#[must_use]
pub(crate) const fn mmcm_filter_lookup(divide: u64) -> u64 {
    let lookup_optimized: [u64; 64] = [
        0b00_1011_1100,
        0b01_0011_1100,
        0b01_0110_1100,
        0b01_1101_1100,
        0b11_0101_1100,
        0b11_1010_1100,
        0b11_1011_0100,
        0b11_1100_1100,
        0b11_1001_0100,
        0b11_1101_0100,
        0b11_1110_0100,
        0b11_0100_0100,
        0b11_1110_0100,
        0b11_1110_0100,
        0b11_1110_0100,
        0b11_1110_0100,
        0b11_1101_0100,
        0b11_1101_0100,
        0b11_0000_0100,
        0b11_0000_0100,
        0b11_0000_0100,
        0b01_0111_0000,
        0b01_0111_0000,
        0b01_0111_0000,
        0b01_0111_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1101_0000,
        0b00_1010_0000,
        0b00_1010_0000,
        0b00_1010_0000,
        0b00_1010_0000,
        0b00_1010_0000,
        0b01_1100_0100,
        0b01_1100_0100,
        0b01_0011_0000,
        0b01_0011_0000,
        0b01_0011_0000,
        0b01_0011_0000,
        0b01_1000_0100,
        0b01_1000_0100,
        0b01_0101_1000,
        0b01_0101_1000,
        0b01_0101_1000,
        0b00_1001_0000,
        0b00_1001_0000,
        0b00_1001_0000,
        0b00_1001_0000,
        0b01_0010_1000,
        0b00_1111_0000,
        0b00_1111_0000,
    ];
    lookup_optimized[divide as usize - 1]
}

#[must_use]
pub(crate) fn mmcm_frac_count_calc(divide: u64, frac: u64) -> u64 {
    let clkout0_divide_frac: u64 = frac / 125;
    let clkout0_divide_int: u64 = divide;

    let even_part_high: u64 = clkout0_divide_int >> 1;
    let even_part_low: u64 = even_part_high;

    let odd: u64 = clkout0_divide_int - even_part_high - even_part_low;
    let odd_and_frac: u64 = 8 * odd + clkout0_divide_frac;

    let lt_frac: u64 = even_part_high - (odd_and_frac <= 9) as u64;
    let ht_frac: u64 = even_part_low - (odd_and_frac <= 8) as u64;

    let pm_fall: u64 = ((odd & 0b1111111) << 2) + ((clkout0_divide_frac & 0b110) >> 1);

    let wf_fall_frac = (2..=9).contains(&odd_and_frac)
        || ((clkout0_divide_frac == 1) && (clkout0_divide_int == 2));
    let wf_rise_frac = (1..=8).contains(&odd_and_frac);

    let a_per_in_octets = 8 * divide + frac / 125;
    let a_phase_in_cycles = 10 * a_per_in_octets / 360000;
    let pm_rise_frac = if (a_phase_in_cycles & 0xFF) == 0 {
        0
    } else {
        (a_phase_in_cycles & 0xFF) - (a_phase_in_cycles & 0b11111000)
    };

    let dt_calc = (10 * a_per_in_octets / 8) / 360000;
    let dt = dt_calc & 0xFF;

    let pm_rise_frac_filtered = if pm_rise_frac >= 8 {
        pm_rise_frac - 8
    } else {
        pm_rise_frac
    };

    let pm_fall_frac = pm_fall + pm_rise_frac;
    let pm_fall_frac_filtered = pm_fall + pm_rise_frac - (pm_fall_frac & 0b11111000);

    let drp_regshared: u64 =
        0b110000 | ((pm_fall_frac_filtered & 0b111) << 1) | (wf_fall_frac as u64);
    let drp_reg2: u64 = 0b0000_1000_0000_0000
        | ((clkout0_divide_frac & 0b111) << 12)
        | ((wf_rise_frac as u64) << 10)
        | (dt & 0b111111);
    let drp_reg1: u64 = ((pm_rise_frac_filtered & 0b111) << 13)
        | ((ht_frac & 0b111111) << 6)
        | (lt_frac & 0b111111);
    (drp_regshared << 32) | (drp_reg2 << 16) | drp_reg1
}
// GRCOV_EXCL_STOP
