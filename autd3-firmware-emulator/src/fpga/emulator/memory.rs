use std::{
    cell::{LazyCell, RefCell},
    collections::HashMap,
};

use crate::FPGAEmulator;

use autd3_derive::Builder;
use autd3_driver::firmware::fpga::Segment;

use super::super::params::*;

#[derive(Builder)]
pub struct Memory {
    pub(crate) num_transducers: usize,
    #[get]
    pub(crate) controller_bram: LazyCell<RefCell<Vec<u16>>>,
    #[get]
    pub(crate) phase_corr_bram: LazyCell<RefCell<Vec<u16>>>,
    #[cfg(feature = "dynamic_freq")]
    #[get]
    pub(crate) drp_bram: LazyCell<RefCell<Vec<u16>>>,
    #[get]
    pub(crate) modulation_bram: LazyCell<RefCell<HashMap<Segment, Vec<u16>>>>,
    #[get]
    pub(crate) stm_bram: LazyCell<RefCell<HashMap<Segment, Vec<u16>>>>,
    #[get]
    pub(crate) duty_table_bram: LazyCell<RefCell<Vec<u16>>>,
    pub(crate) tr_pos: LazyCell<Vec<u64>>,
    pub(crate) sin_table: LazyCell<Vec<u8>>,
    pub(crate) atan_table: LazyCell<Vec<u8>>,
}

impl Memory {
    pub fn new(num_transducers: usize) -> Self {
        Self {
            num_transducers,
            controller_bram: LazyCell::new(|| {
                let mut v = vec![0x0000; 256];
                v[ADDR_VERSION_NUM_MAJOR] =
                    (ENABLED_FEATURES_BITS as u16) << 8 | VERSION_NUM_MAJOR as u16;
                v[ADDR_VERSION_NUM_MINOR] = VERSION_NUM_MINOR as u16;
                RefCell::new(v)
            }),
            phase_corr_bram: LazyCell::new(|| {
                RefCell::new(vec![0x0000; 256 / std::mem::size_of::<u16>()])
            }),
            #[cfg(feature = "dynamic_freq")]
            drp_bram: LazyCell::new(|| RefCell::new(vec![0x0000; 32 * std::mem::size_of::<u64>()])),
            modulation_bram: LazyCell::new(|| {
                RefCell::new(
                    [
                        (
                            Segment::S0,
                            vec![0x0000; 32768 / std::mem::size_of::<u16>()],
                        ),
                        (
                            Segment::S1,
                            vec![0x0000; 32768 / std::mem::size_of::<u16>()],
                        ),
                    ]
                    .into_iter()
                    .collect(),
                )
            }),
            duty_table_bram: LazyCell::new(|| {
                let mut v = vec![0x0000; 256 / std::mem::size_of::<u16>()];
                let pwe_init_data = include_bytes!("asin.dat");
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        pwe_init_data.as_ptr(),
                        v.as_mut_ptr() as _,
                        pwe_init_data.len(),
                    );
                }
                RefCell::new(v)
            }),
            stm_bram: LazyCell::new(|| {
                RefCell::new(
                    [
                        (Segment::S0, vec![0x0000; 1024 * 256]),
                        (Segment::S1, vec![0x0000; 1024 * 256]),
                    ]
                    .into_iter()
                    .collect(),
                )
            }),
            tr_pos: LazyCell::new(|| {
                vec![
                    0x00000000, 0x01960000, 0x032d0000, 0x04c30000, 0x065a0000, 0x07f00000,
                    0x09860000, 0x0b1d0000, 0x0cb30000, 0x0e4a0000, 0x0fe00000, 0x11760000,
                    0x130d0000, 0x14a30000, 0x163a0000, 0x17d00000, 0x19660000, 0x1afd0000,
                    0x00000196, 0x04c30196, 0x065a0196, 0x07f00196, 0x09860196, 0x0b1d0196,
                    0x0cb30196, 0x0e4a0196, 0x0fe00196, 0x11760196, 0x130d0196, 0x14a30196,
                    0x163a0196, 0x17d00196, 0x1afd0196, 0x0000032d, 0x0196032d, 0x032d032d,
                    0x04c3032d, 0x065a032d, 0x07f0032d, 0x0986032d, 0x0b1d032d, 0x0cb3032d,
                    0x0e4a032d, 0x0fe0032d, 0x1176032d, 0x130d032d, 0x14a3032d, 0x163a032d,
                    0x17d0032d, 0x1966032d, 0x1afd032d, 0x000004c3, 0x019604c3, 0x032d04c3,
                    0x04c304c3, 0x065a04c3, 0x07f004c3, 0x098604c3, 0x0b1d04c3, 0x0cb304c3,
                    0x0e4a04c3, 0x0fe004c3, 0x117604c3, 0x130d04c3, 0x14a304c3, 0x163a04c3,
                    0x17d004c3, 0x196604c3, 0x1afd04c3, 0x0000065a, 0x0196065a, 0x032d065a,
                    0x04c3065a, 0x065a065a, 0x07f0065a, 0x0986065a, 0x0b1d065a, 0x0cb3065a,
                    0x0e4a065a, 0x0fe0065a, 0x1176065a, 0x130d065a, 0x14a3065a, 0x163a065a,
                    0x17d0065a, 0x1966065a, 0x1afd065a, 0x000007f0, 0x019607f0, 0x032d07f0,
                    0x04c307f0, 0x065a07f0, 0x07f007f0, 0x098607f0, 0x0b1d07f0, 0x0cb307f0,
                    0x0e4a07f0, 0x0fe007f0, 0x117607f0, 0x130d07f0, 0x14a307f0, 0x163a07f0,
                    0x17d007f0, 0x196607f0, 0x1afd07f0, 0x00000986, 0x01960986, 0x032d0986,
                    0x04c30986, 0x065a0986, 0x07f00986, 0x09860986, 0x0b1d0986, 0x0cb30986,
                    0x0e4a0986, 0x0fe00986, 0x11760986, 0x130d0986, 0x14a30986, 0x163a0986,
                    0x17d00986, 0x19660986, 0x1afd0986, 0x00000b1d, 0x01960b1d, 0x032d0b1d,
                    0x04c30b1d, 0x065a0b1d, 0x07f00b1d, 0x09860b1d, 0x0b1d0b1d, 0x0cb30b1d,
                    0x0e4a0b1d, 0x0fe00b1d, 0x11760b1d, 0x130d0b1d, 0x14a30b1d, 0x163a0b1d,
                    0x17d00b1d, 0x19660b1d, 0x1afd0b1d, 0x00000cb3, 0x01960cb3, 0x032d0cb3,
                    0x04c30cb3, 0x065a0cb3, 0x07f00cb3, 0x09860cb3, 0x0b1d0cb3, 0x0cb30cb3,
                    0x0e4a0cb3, 0x0fe00cb3, 0x11760cb3, 0x130d0cb3, 0x14a30cb3, 0x163a0cb3,
                    0x17d00cb3, 0x19660cb3, 0x1afd0cb3, 0x00000e4a, 0x01960e4a, 0x032d0e4a,
                    0x04c30e4a, 0x065a0e4a, 0x07f00e4a, 0x09860e4a, 0x0b1d0e4a, 0x0cb30e4a,
                    0x0e4a0e4a, 0x0fe00e4a, 0x11760e4a, 0x130d0e4a, 0x14a30e4a, 0x163a0e4a,
                    0x17d00e4a, 0x19660e4a, 0x1afd0e4a, 0x00000fe0, 0x01960fe0, 0x032d0fe0,
                    0x04c30fe0, 0x065a0fe0, 0x07f00fe0, 0x09860fe0, 0x0b1d0fe0, 0x0cb30fe0,
                    0x0e4a0fe0, 0x0fe00fe0, 0x11760fe0, 0x130d0fe0, 0x14a30fe0, 0x163a0fe0,
                    0x17d00fe0, 0x19660fe0, 0x1afd0fe0, 0x00001176, 0x01961176, 0x032d1176,
                    0x04c31176, 0x065a1176, 0x07f01176, 0x09861176, 0x0b1d1176, 0x0cb31176,
                    0x0e4a1176, 0x0fe01176, 0x11761176, 0x130d1176, 0x14a31176, 0x163a1176,
                    0x17d01176, 0x19661176, 0x1afd1176, 0x0000130d, 0x0196130d, 0x032d130d,
                    0x04c3130d, 0x065a130d, 0x07f0130d, 0x0986130d, 0x0b1d130d, 0x0cb3130d,
                    0x0e4a130d, 0x0fe0130d, 0x1176130d, 0x130d130d, 0x14a3130d, 0x163a130d,
                    0x17d0130d, 0x1966130d, 0x1afd130d, 0x000014a3, 0x019614a3, 0x032d14a3,
                    0x04c314a3, 0x065a14a3, 0x07f014a3, 0x098614a3, 0x0b1d14a3, 0x0cb314a3,
                    0x0e4a14a3, 0x0fe014a3, 0x117614a3, 0x130d14a3, 0x14a314a3, 0x163a14a3,
                    0x17d014a3, 0x196614a3, 0x1afd14a3, 0x00000000, 0x00000000, 0x00000000,
                    0x00000000, 0x00000000, 0x00000000, 0x00000000,
                ]
            }),
            sin_table: LazyCell::new(|| include_bytes!("sin.dat").to_vec()),
            atan_table: LazyCell::new(|| include_bytes!("atan.dat").to_vec()),
        }
    }

    pub fn read_bram_as<T>(bram: &[u16], addr: usize) -> T {
        unsafe { (bram.as_ptr().add(addr) as *const T).read_unaligned() }
    }

    pub fn write(&mut self, addr: u16, data: u16) {
        let select = ((addr >> 14) & 0x0003) as u8;
        let addr = (addr & 0x3FFF) as usize;
        match select {
            BRAM_SELECT_CONTROLLER => match addr >> 8 {
                BRAM_CNT_SEL_MAIN => self.controller_bram_mut()[addr] = data,
                BRAM_CNT_SEL_PHASE_CORR => self.phase_corr_bram_mut()[addr & 0xFF] = data,
                #[cfg(feature = "dynamic_freq")]
                BRAM_CNT_SEL_CLOCK => self.drp_bram_mut()[addr & 0xFF] = data,
                _ => unreachable!(),
            },
            BRAM_SELECT_MOD => {
                let segment = match self.controller_bram()[ADDR_MOD_MEM_WR_SEGMENT] {
                    0 => Segment::S0,
                    1 => Segment::S1,
                    _ => unreachable!(),
                };
                self.modulation_bram_mut().get_mut(&segment).unwrap()[addr] = data;
            }
            BRAM_SELECT_PWE_TABLE => {
                self.duty_table_bram_mut()[addr] = data;
            }
            BRAM_SELECT_STM => {
                let segment = match self.controller_bram()[ADDR_STM_MEM_WR_SEGMENT] {
                    0 => Segment::S0,
                    1 => Segment::S1,
                    _ => unreachable!(),
                };
                self.stm_bram_mut().get_mut(&segment).unwrap()
                    [(self.controller_bram()[ADDR_STM_MEM_WR_PAGE] as usize) << 14 | addr] = data;
            }
            _ => unreachable!(),
        }
    }

    pub fn update(&mut self, fpga_state: u16) {
        self.controller_bram_mut()[ADDR_FPGA_STATE] = fpga_state;
    }
}

impl FPGAEmulator {
    pub(crate) fn read(&self, addr: u16) -> u16 {
        let select = ((addr >> 14) & 0x0003) as u8;
        let addr = (addr & 0x3FFF) as usize;
        match select {
            BRAM_SELECT_CONTROLLER => self.mem.controller_bram()[addr],
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    #[cfg_attr(miri, ignore)]
    fn read_panic() {
        let fpga = FPGAEmulator::new(249);
        let addr = (BRAM_SELECT_MOD as u16) << 14;
        fpga.read(addr as _);
    }
}
