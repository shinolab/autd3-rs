use std::collections::HashMap;

use crate::FPGAEmulator;

use autd3_core::firmware::Segment;
use getset::{Getters, MutGetters};

use super::super::params::*;

#[cfg(not(feature = "thread-safe"))]
mod mem {
    pub struct Bram<T: Copy> {
        pub(crate) mem: std::cell::LazyCell<std::cell::RefCell<Vec<T>>>,
    }

    impl<T: Copy> Bram<T> {
        pub fn mem(&self) -> std::cell::Ref<'_, Vec<T>> {
            self.mem.borrow()
        }

        pub fn mem_mut(&self) -> std::cell::RefMut<'_, Vec<T>> {
            self.mem.borrow_mut()
        }
    }

    pub struct Brom<T> {
        pub(crate) mem: std::cell::LazyCell<Vec<T>>,
    }

    impl<T> Brom<T> {
        pub fn new(f: fn() -> Vec<T>) -> Self {
            Brom {
                mem: std::cell::LazyCell::new(f),
            }
        }
    }
}

#[cfg(feature = "thread-safe")]
mod mem {
    pub struct Bram<T: Copy> {
        pub(crate) mem: std::sync::LazyLock<std::sync::RwLock<Vec<T>>>,
    }

    impl<T: Copy> Bram<T> {
        pub fn mem(&self) -> std::sync::RwLockReadGuard<'_, Vec<T>> {
            self.mem.read().unwrap()
        }

        pub fn mem_mut(&self) -> std::sync::RwLockWriteGuard<'_, Vec<T>> {
            self.mem.write().unwrap()
        }
    }

    pub struct Brom<T> {
        pub(crate) mem: std::sync::LazyLock<Vec<T>>,
    }

    impl<T> Brom<T> {
        pub fn new(f: fn() -> Vec<T>) -> Self {
            Brom {
                mem: std::sync::LazyLock::new(f),
            }
        }
    }
}

impl<T: Copy> Bram<T> {
    #[must_use]
    pub fn read(&self, index: usize) -> T {
        self.mem()[index]
    }

    #[must_use]
    pub fn read_bram_as<S>(&self, addr: usize) -> S {
        unsafe { (self.mem().as_ptr().add(addr) as *const S).read_unaligned() }
    }

    pub fn write(&self, index: usize, value: T) {
        self.mem_mut()[index] = value;
    }

    pub fn fill(&self, value: T) {
        self.mem_mut().fill(value);
    }
}

impl<T> Brom<T> {
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.mem.iter()
    }
}

impl<T> std::ops::Index<usize> for Brom<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.mem[index]
    }
}

use mem::*;

macro_rules! create_bram {
    ($f:block) => {
        Bram {
            #[cfg(not(feature = "thread-safe"))]
            mem: std::cell::LazyCell::new(|| std::cell::RefCell::new($f)),
            #[cfg(feature = "thread-safe")]
            mem: std::sync::LazyLock::new(|| std::sync::RwLock::new($f)),
        }
    };
}

#[derive(Getters, MutGetters)]
pub struct Memory {
    pub(crate) num_transducers: usize,
    pub(crate) controller_bram: Bram<u16>,
    #[getset(get = "pub", get_mut = "pub")]
    pub(crate) phase_corr_bram: Bram<u16>,
    pub(crate) output_mask_bram: Bram<u16>,
    pub(crate) modulation_bram: HashMap<Segment, Bram<u16>>,
    pub(crate) stm_bram: HashMap<Segment, Bram<u16>>,
    pub(crate) duty_table_bram: Bram<u16>,
    pub(crate) tr_pos: Brom<u64>,
    pub(crate) sin_table: Brom<u8>,
    pub(crate) atan_table: Brom<u8>,
}

impl Memory {
    #[must_use]
    pub fn new(num_transducers: usize) -> Self {
        Self {
            num_transducers,
            controller_bram: create_bram!({
                let mut v = vec![0x0000; 256];
                v[ADDR_VERSION_NUM_MAJOR] =
                    ((ENABLED_FEATURES_BITS as u16) << 8) | VERSION_NUM_MAJOR as u16;
                v[ADDR_VERSION_NUM_MINOR] = VERSION_NUM_MINOR as u16;
                v
            }),
            phase_corr_bram: create_bram!({ vec![0x0000; 256 / std::mem::size_of::<u16>()] }),
            output_mask_bram: create_bram!({ vec![0xFFFF; 32] }),
            modulation_bram: HashMap::from([
                (
                    Segment::S0,
                    create_bram!({ vec![0x0000; 65536 / std::mem::size_of::<u16>()] }),
                ),
                (
                    Segment::S1,
                    create_bram!({ vec![0x0000; 65536 / std::mem::size_of::<u16>()] }),
                ),
            ]),
            duty_table_bram: create_bram!({
                let pwe_init_data: &[u8; 512] = include_bytes!("asin.dat");
                Vec::from_iter((0..256).map(|i| {
                    u16::from_le_bytes([pwe_init_data[(i << 1) + 1], pwe_init_data[i << 1]])
                }))
            }),
            stm_bram: {
                HashMap::from([
                    (Segment::S0, create_bram!({ vec![0x0000; 1024 * 256] })),
                    (Segment::S1, create_bram!({ vec![0x0000; 1024 * 256] })),
                ])
            },
            tr_pos: Brom::new(|| {
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
            sin_table: Brom::new(|| include_bytes!("sin.dat").to_vec()),
            atan_table: Brom::new(|| include_bytes!("atan.dat").to_vec()),
        }
    }

    pub fn write(&mut self, addr: u16, data: u16) {
        let select = ((addr >> 14) & 0x0003) as u8;
        let addr = (addr & 0x3FFF) as usize;
        match select {
            BRAM_SELECT_CONTROLLER => match addr >> 8 {
                BRAM_CNT_SEL_MAIN => self.controller_bram.write(addr, data),
                BRAM_CNT_SEL_PHASE_CORR => self.phase_corr_bram.write(addr & 0xFF, data),
                BRAM_CNT_SEL_OUTPUT_MASK => self.output_mask_bram.write(addr & 0xFF, data),
                _ => unreachable!(),
            },
            BRAM_SELECT_MOD => {
                let segment = match self.controller_bram.read(ADDR_MOD_MEM_WR_SEGMENT) {
                    0 => Segment::S0,
                    1 => Segment::S1,
                    _ => unreachable!(),
                };
                self.modulation_bram.get_mut(&segment).unwrap().write(
                    ((self.controller_bram.read(ADDR_MOD_MEM_WR_PAGE) as usize) << 14) | addr,
                    data,
                );
            }
            BRAM_SELECT_PWE_TABLE => {
                self.duty_table_bram.write(addr, data);
            }
            BRAM_SELECT_STM => {
                let segment = match self.controller_bram.read(ADDR_STM_MEM_WR_SEGMENT) {
                    0 => Segment::S0,
                    1 => Segment::S1,
                    _ => unreachable!(),
                };
                self.stm_bram.get_mut(&segment).unwrap().write(
                    ((self.controller_bram.read(ADDR_STM_MEM_WR_PAGE) as usize) << 14) | addr,
                    data,
                );
            }
            _ => unreachable!(),
        }
    }

    pub fn update(&mut self, fpga_state: u16) {
        self.controller_bram.write(ADDR_FPGA_STATE, fpga_state);
    }
}

impl FPGAEmulator {
    #[must_use]
    pub(crate) fn read(&self, addr: u16) -> u16 {
        let select = ((addr >> 14) & 0x0003) as u8;
        let addr = (addr & 0x3FFF) as usize;
        match select {
            BRAM_SELECT_CONTROLLER => self.mem.controller_bram.read(addr),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn read_panic() {
        let fpga = FPGAEmulator::new(249);
        let addr = (BRAM_SELECT_MOD as u16) << 14;
        _ = fpga.read(addr as _);
    }
}
