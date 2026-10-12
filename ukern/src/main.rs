#![no_std]
#![no_main]
#![no_builtins]
#![crate_type = "bin"]
#![feature(lang_items, core_intrinsics)]

use core::intrinsics;
use core::panic::PanicInfo;
use ukern::{arch, drive, ps2, serial, terminal};

/// Panic handler.
#[panic_handler]
fn rust_begin_panic(_info: &PanicInfo<'_>) -> ! {
    intrinsics::abort_immediate()
}

/// eh.
#[lang = "eh_personality"]
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
    terminal::init();
    terminal::print("Initialized 80x25 VGA terminal.\n");

    unsafe {
        serial::init_default(0);
        serial::com(0).unwrap().write_buf(8, b"Hello, world!\r\n");
        terminal::print("Initialized COM0.\n");
        match ps2::init() {
            Ok(_) => terminal::print("Initialized the PS/2 controller.\n"),
            Err(ps2::InitError::AllPorts(a, b)) => {
                terminal::print("PS/2 controller ports init err #");
                terminal::put_char(b'0' + a); // there is no itoa or anything currently
                terminal::put_char(b'0' + b);
                terminal::print(".\n");
                return;
            }
            Err(ps2::InitError::FirstPort(a)) => {
                terminal::print("PS/2 controller port #1 init err #");
                terminal::put_char(b'0' + a);
                terminal::print(".\n");
                return;
            }
            Err(ps2::InitError::SecondPort(b)) => {
                terminal::print("PS/2 controller port #2 init err #");
                terminal::put_char(b'0' + b);
                terminal::print(".\n");
                return;
            }
            Err(ps2::InitError::SelfTest) => {
                terminal::print("PS/2 controller self test failed.\n");
                return;
            }
        }

        arch::int::init();
        terminal::print("Enabled interrupts.\n");
        arch::pit::init(1000);
        terminal::print("Enabled the PIT.\n");
        terminal::print("Initializing paging...\n");
        arch::paging::init();
        terminal::print("Enabled paging.\n");

        drive::pata::init();
        let mut buffer = [0; 512];
        if let Some(drive) = drive::pata::Controller::Primary.drive(drive::pata::DriveNumber::First)
        {
            terminal::print("IDE: primary controller, first drive found and initialized.\n");
            if drive
                .read_into(1.try_into().unwrap(), 1, &mut buffer)
                .is_ok()
            {
                for i in 0..20 {
                    terminal::put_char(buffer[i as usize]);
                }
            } else {
                terminal::print("couldn't read from drive.\n");
            }
        }
        if let Some(drive) =
            drive::pata::Controller::Primary.drive(drive::pata::DriveNumber::Second)
        {
            terminal::print("IDE: primary controller, second drive found and initialized.\n");
            if drive
                .read_into(1.try_into().unwrap(), 1, &mut buffer)
                .is_ok()
            {
                for i in 0..20 {
                    terminal::put_char(buffer[i as usize]);
                }
            } else {
                terminal::print("couldn't read from drive.\n");
            }
        }
        if drive::pata::Controller::Secondary
            .drive(drive::pata::DriveNumber::First)
            .is_some()
        {
            terminal::print("IDE: secondary controller, first drive found and initialized.\n");
        }
        if drive::pata::Controller::Secondary
            .drive(drive::pata::DriveNumber::Second)
            .is_some()
        {
            terminal::print("IDE: secondary controller, second drive found and initialized.\n");
        }
    }

    terminal::clear();
    let mut kbd_state = keyboard::KeyboardState::default();
    loop {
        let action = keyboard::key_action();
        if let Ok(c) = action.map() {
            terminal::put_char(c);
        }
        keyboard::set_led_state(kbd_state.into());
    }
}
