#![no_std]
#![no_main]

extern crate alloc;

#[allow(unused_imports)]
use autd3_core;

#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
fn main() {}

#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
fn __libc_start_main(_main: fn() -> isize) {}

#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
fn rust_eh_personality() {}

use alloc::alloc::*;

#[derive(Default)]
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        unimplemented!()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unimplemented!()
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;
