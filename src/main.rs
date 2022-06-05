#![no_std]
#![no_main]
#![no_builtins]
#![crate_type = "bin"]
#![feature(lang_items, core_intrinsics)]

use core::intrinsics;
use core::panic::PanicInfo;
use uos::terminal;

#[panic_handler]
#[no_mangle]
extern "C" fn rust_begin_panic(_info: &PanicInfo<'_>) -> ! {
    intrinsics::abort();
}

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn rust_eh_personality() {}

#[no_mangle]
#[ignore = "snake_case"]
extern "C" fn _Unwind_Resume() {
}

#[no_mangle]
fn kernel_main() {
    terminal::print("Hello, world!");
}
