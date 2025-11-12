#![allow(clippy::missing_safety_doc)]

pub mod cpu;
pub mod fpga;

pub use cpu::emulator::CPUEmulator;
pub use fpga::emulator::FPGAEmulator;
