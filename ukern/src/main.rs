#![no_std]
#![no_main]
#![no_builtins]
#![crate_type = "bin"]
#![feature(lang_items, core_intrinsics)]

use core::intrinsics;
use core::panic::PanicInfo;
use ukern::{keyboard, terminal, arch};

/// Panic handler.
#[panic_handler]
#[no_mangle]
extern "C" fn rust_begin_panic(_info: &PanicInfo<'_>) -> ! {
    intrinsics::abort();
}

/// eh.
#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn rust_eh_personality() {}

/// The kernel couldn't compile without it so I just left it here.
///
/// Basically, cargo was throwing errors at me for an "undefined reference to
/// `_Unwind_Resume`". Also, overriding this function via a `lang` item was
/// possible some versions ago, but they removed it for some reason.
///
/// cargo bad.
///
/// Don't touch it btw.
#[no_mangle]
#[ignore = "snake_case"]
extern "C" fn _Unwind_Resume() {}

/// The main function of the high-level kernel.
#[no_mangle]
fn kernel_main() {
    unsafe { arch::paging::init(); }

    let mut kbd_state = keyboard::KeyboardState::default();
    terminal::clear();
    terminal::print("uOS\n");
    loop {
        let action = keyboard::key_action();
        if let Some(c) = action.map(&mut kbd_state) {
            terminal::put_char(c);
        }
        keyboard::set_led_state(kbd_state.into());
    }
}
