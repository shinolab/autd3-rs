mod drp;

use zerocopy::{Immutable, IntoBytes};

use crate::{
    defined::{DRP_ROM_SIZE, Freq, ULTRASOUND_PERIOD_COUNT},
    error::AUTDDriverError,
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

const DIVCLK_DIVIDE_MIN: u64 = 1;
const DIVCLK_DIVIDE_MAX: u64 = 106;
const MULT_MIN: f32 = 2.0;
const MULT_MAX: f32 = 64.0;
const DIV_MIN: f32 = 1.0;
const DIV_MAX: f32 = 128.0;
const INCREMENTS: f32 = 0.125;
const VCO_MIN: f32 = 600.0e6;
const VCO_MAX: f32 = 1600.0e6;

#[derive(Clone, Copy)]
#[repr(C)]
#[derive(IntoBytes, Immutable)]
pub struct ClkControlFlags(u8);

bitflags::bitflags! {
    impl ClkControlFlags : u8 {
        const NONE  = 0;
        const BEGIN = 1 << 0;
        const END   = 1 << 1;
    }
}

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct Clk {
    tag: TypeTag,
    flag: ClkControlFlags,
    size: u16,
}

pub struct ConfigureClockOp {
    ultrasound_freq: Freq<u32>,
    rom: Vec<u64>,
    remains: usize,
}

fn calculate_mult_div(frequency: u32) -> Option<(u64, u64, u64)> {
    const FPGA_BASE_CLK_FREQ: u64 = 25600000;

    let f = frequency as u64;
    let b = FPGA_BASE_CLK_FREQ;
    itertools::iproduct!(
        DIVCLK_DIVIDE_MIN..=DIVCLK_DIVIDE_MAX,
        (MULT_MIN / INCREMENTS) as u64..=(MULT_MAX / INCREMENTS) as u64,
        (DIV_MIN / INCREMENTS) as u64..=(DIV_MAX / INCREMENTS) as u64
    )
    .find(|&(div, m, d)| {
        if !(VCO_MIN..=VCO_MAX).contains(&(b as f32 * m as f32 * INCREMENTS / div as f32)) {
            return false;
        }
        f * d == b * m
    })
}

impl ConfigureClockOp {
    pub const fn new(ultrasound_freq: Freq<u32>) -> Self {
        Self {
            ultrasound_freq,
            rom: vec![],
            remains: DRP_ROM_SIZE,
        }
    }
}

impl Operation for ConfigureClockOp {
    type Error = AUTDDriverError;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        let sent = DRP_ROM_SIZE - self.remains;

        if sent == 0 {
            let fpga_clk_freq = self.ultrasound_freq.hz() * ULTRASOUND_PERIOD_COUNT as u32;
            if fpga_clk_freq % 125 != 0 {
                return Err(AUTDDriverError::InvalidFrequency(self.ultrasound_freq));
            }
            let (clkdiv, mult, div) = calculate_mult_div(fpga_clk_freq)
                .ok_or(AUTDDriverError::InvalidFrequency(self.ultrasound_freq))?;

            let mut rom = vec![0; DRP_ROM_SIZE];

            let clkout0_frac = drp::mmcm_frac_count_calc(div / 8, (div % 8) * 125);
            let divclk = drp::mmcm_count_calc(clkdiv);
            let clkfbout_frac = drp::mmcm_frac_count_calc(mult / 8, (mult % 8) * 125);
            let lock = drp::mmcm_lock_lookup(mult / 8);
            let digital_filt = drp::mmcm_filter_lookup(mult / 8);

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

            self.rom = rom;
        }

        let size = self
            .remains
            .min((tx.len() - size_of::<Clk>()) / size_of::<u64>());

        super::write_to_tx(
            tx,
            Clk {
                tag: TypeTag::ConfigFPGAClock,
                flag: if sent == 0 {
                    ClkControlFlags::BEGIN
                } else {
                    ClkControlFlags::NONE
                } | if sent + size == DRP_ROM_SIZE {
                    ClkControlFlags::END
                } else {
                    ClkControlFlags::NONE
                },
                size: size as _,
            },
        );

        tx[size_of::<Clk>()..]
            .chunks_mut(size_of::<u64>())
            .zip(self.rom[sent..].iter())
            .for_each(|(dst, &src)| {
                super::write_to_tx(dst, src);
            });

        self.remains -= size;
        Ok(size_of::<Clk>() + size * size_of::<u64>())
    }

    fn required_size(&self, _: &Device) -> usize {
        size_of::<Clk>() + size_of::<u64>()
    }

    fn is_done(&self) -> bool {
        self.remains == 0
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use zerocopy::FromBytes;

    use crate::{
        defined::{Freq, Hz},
        firmware::operation::tests::create_device,
    };

    use super::*;

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[rstest::rstest]
    #[test]
    #[case::f40k(vec![
        0x000000280000ffff,
        0x0000000980006c00,
        0x000000081000071c,
        0x0000000a10000041,
        0x0000000bfc000040,
        0x0000000c10000041,
        0x0000000dfc000040,
        0x0000000e10000041,
        0x0000000ffc000040,
        0x0000001010000041,
        0x00000011fc000040,
        0x0000000610000041,
        0x00000007c0001c40,
        0x0000001210000000,
        0x00000013c0003040,
        0x00000016c0001041,
        0x00000014100002cb,
        0x0000001580004800,
        0x00000018fc0001a9,
        0x0000001980007c01,
        0x0000001a80007fe9,
        0x0000004e66ff1100,
        0x0000004f666f9000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000001,
    ], 40000*Hz)]
    #[case::f41k(vec![
        0x000000280000ffff,
        0x0000000980004c00,
        0x000000081000079e,
        0x0000000a10000041,
        0x0000000bfc000040,
        0x0000000c10000041,
        0x0000000dfc000040,
        0x0000000e10000041,
        0x0000000ffc000040,
        0x0000001010000041,
        0x00000011fc000040,
        0x0000000610000041,
        0x00000007c0001440,
        0x0000001210000000,
        0x00000013c0003040,
        0x00000016c0001041,
        0x000000141000030c,
        0x0000001580005800,
        0x00000018fc000190,
        0x0000001980007c01,
        0x0000001a80007fe9,
        0x0000004e66ff1100,
        0x0000004f666f9000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000001,
    ], 41000*Hz)]
    fn config_clk(#[case] expect_rom: Vec<u64>, #[case] freq: Freq<u32>) {
        const FRAME_SIZE: usize = size_of::<Clk>() + 12 * size_of::<u64>();

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut op = ConfigureClockOp::new(freq);

        // First frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<Clk>() + size_of::<u64>()
            );

            assert_eq!(op.remains, DRP_ROM_SIZE);

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<Clk>() + 12 * size_of::<u64>())
            );
            assert_eq!(op.remains, DRP_ROM_SIZE - 12);

            assert_eq!(TypeTag::ConfigFPGAClock as u8, tx[0]);
            assert_eq!(ClkControlFlags::BEGIN.bits(), tx[offset_of!(Clk, flag)]);
            assert_eq!(12, tx[offset_of!(Clk, size)]);
            (0..12).for_each(|i| {
                let offset = size_of::<Clk>() + i * size_of::<u64>();
                assert_eq!(
                    expect_rom[i],
                    u64::read_from_bytes(&tx[offset..offset + size_of::<u64>()]).unwrap(),
                );
            });
        }

        // Second frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<Clk>() + size_of::<u64>()
            );

            assert_eq!(op.remains, DRP_ROM_SIZE - 12);

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<Clk>() + 12 * size_of::<u64>())
            );

            assert_eq!(op.remains, DRP_ROM_SIZE - 24);

            assert_eq!(TypeTag::ConfigFPGAClock as u8, tx[0]);
            assert_eq!(ClkControlFlags::NONE.bits(), tx[offset_of!(Clk, flag)]);
            assert_eq!(12, tx[offset_of!(Clk, size)]);
            (0..12).for_each(|i| {
                let offset = size_of::<Clk>() + i * size_of::<u64>();
                assert_eq!(
                    expect_rom[12 + i],
                    u64::read_from_bytes(&tx[offset..offset + size_of::<u64>()]).unwrap(),
                );
            });
        }

        // Final frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<Clk>() + size_of::<u64>()
            );

            assert_eq!(op.remains, DRP_ROM_SIZE - 12 - 12);

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<Clk>() + u64::BITS as usize)
            );

            assert_eq!(op.remains, 0);

            assert_eq!(TypeTag::ConfigFPGAClock as u8, tx[0]);
            assert_eq!(ClkControlFlags::END.bits(), tx[offset_of!(Clk, flag)]);
            assert_eq!(8, tx[offset_of!(Clk, size)]);
            (0..8).for_each(|i| {
                let offset = size_of::<Clk>() + i * size_of::<u64>();
                assert_eq!(
                    expect_rom[24 + i],
                    u64::read_from_bytes(&tx[offset..offset + size_of::<u64>()]).unwrap(),
                );
            });
        }
    }

    #[rstest::rstest]
    #[test]
    #[case::f40k(Ok(()), 40000*Hz)]
    #[case::f1(Err(AUTDDriverError::InvalidFrequency(1*Hz)), 1*Hz)]
    #[case::f32(Err(AUTDDriverError::InvalidFrequency(125*Hz)), 125*Hz)]
    fn config_clk_validate(#[case] expect: Result<(), AUTDDriverError>, #[case] freq: Freq<u32>) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);
        let mut tx = vec![0x00u8; size_of::<Clk>() + DRP_ROM_SIZE * size_of::<u64>()];

        let mut op = ConfigureClockOp::new(freq);
        assert_eq!(expect, op.pack(&device, &mut tx).map(|_| ()));
    }
}
