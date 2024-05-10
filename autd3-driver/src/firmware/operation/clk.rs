use std::collections::HashMap;

use crate::{
    error::AUTDInternalError,
    firmware::{
        fpga::{DRP_ROM_SIZE, FPGA_BASE_CLK_FREQ, ULTRASOUND_PERIOD},
        operation::{cast, Operation, TypeTag},
    },
    geometry::{Device, Geometry},
};

use super::Remains;

const DIVCLK_DIVIDE_MIN: u64 = 1;
const DIVCLK_DIVIDE_MAX: u64 = 106;
const MULT_MIN: f64 = 2.0;
const MULT_MAX: f64 = 64.0;
const DIV_MIN: f64 = 1.0;
const DIV_MAX: f64 = 128.0;
const INCREMENTS: f64 = 0.125;
const VCO_MIN: f64 = 600.0e6;
const VCO_MAX: f64 = 1600.0e6;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ClkControlFlags(u8);

bitflags::bitflags! {
    impl ClkControlFlags : u8 {
        const NONE  = 0;
        const BEGIN = 1 << 0;
        const END   = 1 << 1;
    }
}

#[repr(C, align(2))]
struct Clk {
    tag: TypeTag,
    flag: ClkControlFlags,
    size: u16,
}

#[derive(Default)]
pub struct ConfigureClockOp {
    rom: HashMap<usize, Vec<u64>>,
    remains: Remains,
}

impl ConfigureClockOp {
    pub fn new() -> Self {
        Self {
            rom: Default::default(),
            remains: Default::default(),
        }
    }
}

// GRCOV_EXCL_START
fn round_frac(decimal: u64, precision: u64) -> u64 {
    if decimal & (1 << (10 - precision)) != 0 {
        decimal + (1 << (10 - precision))
    } else {
        decimal
    }
}

fn mmcm_divider(divide: u64) -> u64 {
    let mut duty_cycle = 50000;
    if divide >= 64 {
        let duty_cycle_min = ((divide - 64) * 100_000) / divide;
        let duty_cycle_max = ((64.5 / divide as f64) * 100000.) as u64;
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

fn mmcm_count_calc(divide: u64) -> u64 {
    let div_calc = mmcm_divider(divide);
    let phase_calc = 0;

    ((phase_calc & 0b11000000000) << 15)
        | ((div_calc & 0b11000000000000) << 10)
        | ((phase_calc & 0b111111) << 16)
        | ((phase_calc & 0b111000000) << 13)
        | (div_calc & 0b111111111111)
}

fn mmcm_lock_lookup(divide: u64) -> u64 {
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

fn mmcm_filter_lookup(divide: u64) -> u64 {
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

fn mmcm_frac_count_calc(divide: u64, frac: u64) -> u64 {
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
        0b110000 | (pm_fall_frac_filtered & 0b111) << 1 | (wf_fall_frac as u64);
    let drp_reg2: u64 = 0b0000_1000_0000_0000
        | (clkout0_divide_frac & 0b111) << 12
        | (wf_rise_frac as u64) << 10
        | (dt & 0b111111);
    let drp_reg1: u64 =
        (pm_rise_frac_filtered & 0b111) << 13 | (ht_frac & 0b111111) << 6 | (lt_frac & 0b111111);
    (drp_regshared << 32) | (drp_reg2 << 16) | drp_reg1
}
// GRCOV_EXCL_STOP

fn calculate_mult_div(frequency: u32) -> Option<(u64, u64, u64)> {
    let f = frequency as u64;
    let b = FPGA_BASE_CLK_FREQ as u64;
    itertools::iproduct!(
        DIVCLK_DIVIDE_MIN..=DIVCLK_DIVIDE_MAX,
        (MULT_MIN / INCREMENTS) as u64..=(MULT_MAX / INCREMENTS) as u64,
        (DIV_MIN / INCREMENTS) as u64..=(DIV_MAX / INCREMENTS) as u64
    )
    .find(|&(div, m, d)| {
        if !(VCO_MIN..=VCO_MAX).contains(&(b as f64 * m as f64 * INCREMENTS / div as f64)) {
            return false;
        }
        f * d == b * m
    })
}

impl Operation for ConfigureClockOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let sent = DRP_ROM_SIZE - self.remains[device];

        let offset = std::mem::size_of::<Clk>();

        let size = (DRP_ROM_SIZE - sent).min((tx.len() - offset) / std::mem::size_of::<u64>());
        assert!(size > 0);

        *cast::<Clk>(tx) = Clk {
            tag: TypeTag::ConfigFPGAClk,
            flag: ClkControlFlags::NONE,
            size: size as _,
        };
        if sent == 0 {
            cast::<Clk>(tx).flag.set(ClkControlFlags::BEGIN, true);
        }

        if sent + size == DRP_ROM_SIZE {
            cast::<Clk>(tx).flag.set(ClkControlFlags::END, true);
        }

        (0..size).for_each(|i| {
            *cast::<u64>(&mut tx[offset + i * std::mem::size_of::<u64>()..]) =
                self.rom[&device.idx()][sent + i];
        });

        self.remains[device] -= size;
        Ok(std::mem::size_of::<Clk>() + size * std::mem::size_of::<u64>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<Clk>() + std::mem::size_of::<u64>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains.init(geometry, |_| DRP_ROM_SIZE);

        geometry.devices().try_for_each(|dev| {
            let fpga_clk_freq = dev.ultrasound_freq() * ULTRASOUND_PERIOD;
            if fpga_clk_freq % 2000 != 0 {
                return Err(AUTDInternalError::InvalidFrequencyError(
                    dev.ultrasound_freq(),
                ));
            }
            let (clkdiv, mult, div) = calculate_mult_div(fpga_clk_freq).ok_or(
                AUTDInternalError::InvalidFrequencyError(dev.ultrasound_freq()),
            )?;

            let mut rom = vec![0; DRP_ROM_SIZE];

            let clkout0_frac = mmcm_frac_count_calc(div / 8, (div % 8) * 125);
            let divclk = mmcm_count_calc(clkdiv);
            let clkfbout_frac = mmcm_frac_count_calc(mult / 8, (mult % 8) * 125);
            let lock = mmcm_lock_lookup(mult / 8);
            let digital_filt = mmcm_filter_lookup(mult / 8);

            let clkout_unused = 0x0000400041;

            rom[0] = 0x28_0000_FFFF;

            rom[1] = 0x09_8000_0000 | (clkout0_frac & 0xFFFF0000) >> 16;
            rom[2] = 0x08_1000_0000 | (clkout0_frac & 0xFFFF);

            rom[3] = 0x0A_1000_0000 | (clkout_unused & 0xFFFF);
            rom[4] = 0x0B_FC00_0000 | (clkout_unused & 0xFFFF0000) >> 16;

            rom[5] = 0x0C_1000_0000 | (clkout_unused & 0xFFFF);
            rom[6] = 0x0D_FC00_0000 | (clkout_unused & 0xFFFF0000) >> 16;

            rom[7] = 0x0E_1000_0000 | (clkout_unused & 0xFFFF);
            rom[8] = 0x0F_FC00_0000 | (clkout_unused & 0xFFFF0000) >> 16;

            rom[9] = 0x10_1000_0000 | (clkout_unused & 0xFFFF);
            rom[10] = 0x11_FC00_0000 | (clkout_unused & 0xFFFF0000) >> 16;

            rom[11] = 0x06_1000_0000 | (clkout_unused & 0xFFFF);
            rom[12] = 0x07_C000_0000
                | (clkout_unused & 0xC0000000) >> 16
                | (clkout0_frac & 0xF00000000) >> 22
                | (clkout_unused & 0x3FF0000) >> 16;

            rom[13] = 0x12_1000_0000;
            rom[14] = 0x13_C000_0000
                | (clkout_unused & 0xC0000000) >> 16
                | (clkfbout_frac & 0xF00000000) >> 22
                | (clkout_unused & 0x3FF0000) >> 16;

            rom[15] = 0x16_C000_0000 | (divclk & 0xC00000) >> 10 | (divclk & 0xFFF);

            rom[16] = 0x14_1000_0000 | (clkfbout_frac & 0xFFFF);
            rom[17] = 0x15_8000_0000 | (clkfbout_frac & 0xFFFF0000) >> 16;

            rom[18] = 0x18_FC00_0000 | (lock & 0x3FF00000) >> 20;
            rom[19] = 0x19_8000_0000 | (lock & 0x7C0000000) >> 20 | (lock & 0x3FF);
            rom[20] = 0x1A_8000_0000 | (lock & 0xF800000000) >> 25 | (lock & 0xFFC00) >> 10;

            rom[21] = 0x4E_66FF_0000
                | (digital_filt & 0b1000000000) << 6
                | (digital_filt & 0b0110000000) << 4
                | (digital_filt & 0b0001000000) << 2;

            rom[22] = 0x4F_666F_0000
                | (digital_filt & 0b0000100000) << 10
                | (digital_filt & 0b0000011000) << 8
                | (digital_filt & 0b0000000110) << 6
                | (digital_filt & 0b0000000001) << 4;

            rom[31] = 1;

            self.rom.insert(dev.idx(), rom);

            Ok(())
        })?;

        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use crate::geometry::tests::create_geometry;

    use super::*;

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[rstest::rstest]
    #[test]
    #[case::f40k(vec![
        0x280000ffff,
        0x980003800,
        0x81000038e,
        0xa10000041,
        0xbfc000040,
        0xc10000041,
        0xdfc000040,
        0xe10000041,
        0xffc000040,
        0x1010000041,
        0x11fc000040,
        0x610000041,
        0x7c0002840,
        0x1210000000,
        0x13c0003040,
        0x16c0001041,
        0x14100002cb,
        0x1580004800,
        0x18fc0001a9,
        0x1980007c01,
        0x1a80007fe9,
        0x4e66ff1100,
        0x4f666f9000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000001,
    ], 40000)]
    #[case::f41k(vec![
        0x280000ffff,
        0x980002800,
        0x8100003cf,
        0xa10000041,
        0xbfc000040,
        0xc10000041,
        0xdfc000040,
        0xe10000041,
        0xffc000040,
        0x1010000041,
        0x11fc000040,
        0x610000041,
        0x7c0002840,
        0x1210000000,
        0x13c0003040,
        0x16c0001041,
        0x141000030c,
        0x1580005800,
        0x18fc000190,
        0x1980007c01,
        0x1a80007fe9,
        0x4e66ff1100,
        0x4f666f9000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000000,
        0x00000001,
    ], 41000)]
    fn config_clk(#[case] expect_rom: Vec<u64>, #[case] freq: u32) {
        const FRAME_SIZE: usize = size_of::<Clk>() + 12 * size_of::<u64>();

        let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, freq);

        let mut op = ConfigureClockOp::new();

        assert!(op.init(&geometry).is_ok());

        // First frame
        {
            geometry.devices().for_each(|dev| {
                assert_eq!(op.required_size(dev), size_of::<Clk>() + size_of::<u64>())
            });

            geometry
                .devices()
                .for_each(|dev| assert_eq!(op.remains[dev], DRP_ROM_SIZE));

            geometry.devices().for_each(|dev| {
                assert_eq!(
                    op.pack(
                        dev,
                        &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                    ),
                    Ok(size_of::<Clk>() + 12 * size_of::<u64>())
                );
            });

            geometry
                .devices()
                .for_each(|dev| assert_eq!(op.remains[dev], DRP_ROM_SIZE - 12));

            geometry.devices().for_each(|dev| {
                assert_eq!(TypeTag::ConfigFPGAClk as u8, tx[dev.idx() * FRAME_SIZE]);
                assert_eq!(
                    ClkControlFlags::BEGIN.bits(),
                    tx[dev.idx() * FRAME_SIZE + offset_of!(Clk, flag)]
                );
                assert_eq!(12, tx[dev.idx() * FRAME_SIZE + offset_of!(Clk, size)]);
                (0..12).for_each(|i| {
                    assert_eq!(
                        expect_rom[i],
                        *cast::<u64>(
                            &mut tx[dev.idx() * FRAME_SIZE
                                + size_of::<Clk>()
                                + i * size_of::<u64>()..]
                        )
                    );
                });
            });
        }

        // Second frame
        {
            geometry.devices().for_each(|dev| {
                assert_eq!(op.required_size(dev), size_of::<Clk>() + size_of::<u64>())
            });

            geometry
                .devices()
                .for_each(|dev| assert_eq!(op.remains[dev], DRP_ROM_SIZE - 12));

            geometry.devices().for_each(|dev| {
                assert_eq!(
                    op.pack(
                        dev,
                        &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                    ),
                    Ok(size_of::<Clk>() + 12 * size_of::<u64>())
                );
            });

            geometry
                .devices()
                .for_each(|dev| assert_eq!(op.remains[dev], DRP_ROM_SIZE - 24));

            geometry.devices().for_each(|dev| {
                assert_eq!(TypeTag::ConfigFPGAClk as u8, tx[dev.idx() * FRAME_SIZE]);
                assert_eq!(
                    ClkControlFlags::NONE.bits(),
                    tx[dev.idx() * FRAME_SIZE + offset_of!(Clk, flag)]
                );
                assert_eq!(12, tx[dev.idx() * FRAME_SIZE + offset_of!(Clk, size)]);
                (0..12).for_each(|i| {
                    assert_eq!(
                        expect_rom[12 + i],
                        *cast::<u64>(
                            &mut tx[dev.idx() * FRAME_SIZE
                                + size_of::<Clk>()
                                + i * size_of::<u64>()..]
                        )
                    );
                });
            });
        }

        // Final frame
        {
            geometry.devices().for_each(|dev| {
                assert_eq!(op.required_size(dev), size_of::<Clk>() + size_of::<u64>())
            });

            geometry
                .devices()
                .for_each(|dev| assert_eq!(op.remains[dev], DRP_ROM_SIZE - 12 - 12));

            geometry.devices().for_each(|dev| {
                assert_eq!(
                    op.pack(
                        dev,
                        &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                    ),
                    Ok(size_of::<Clk>() + u64::BITS as usize)
                );
            });

            geometry
                .devices()
                .for_each(|dev| assert_eq!(op.remains[dev], 0));

            geometry.devices().for_each(|dev| {
                assert_eq!(TypeTag::ConfigFPGAClk as u8, tx[dev.idx() * FRAME_SIZE]);
                assert_eq!(
                    ClkControlFlags::END.bits(),
                    tx[dev.idx() * FRAME_SIZE + offset_of!(Clk, flag)]
                );
                assert_eq!(8, tx[dev.idx() * FRAME_SIZE + offset_of!(Clk, size)]);
                (0..8).for_each(|i| {
                    assert_eq!(
                        expect_rom[24 + i],
                        *cast::<u64>(
                            &mut tx[dev.idx() * FRAME_SIZE
                                + size_of::<Clk>()
                                + i * size_of::<u64>()..]
                        )
                    );
                });
            });
        }
    }

    #[rstest::rstest]
    #[test]
    #[case::f40k(Ok(()), 40000)]
    #[case::f1(Err(AUTDInternalError::InvalidFrequencyError(1)), 1)]
    #[case::f32(Err(AUTDInternalError::InvalidFrequencyError(125)), 125)]
    fn config_clk_validate(#[case] expect: Result<(), AUTDInternalError>, #[case] freq: u32) {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT, freq);

        let mut op = ConfigureClockOp::new();

        assert_eq!(expect, op.init(&geometry));
    }
}
