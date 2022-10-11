//! Various utilities.
//!
//! I know you shouldn't make a module this general, these things just really
//! don't fit anywhere.

use core::arch::asm;

/*pub fn cpuid() -> CPUID {
    // TODO
}*/

/// Disables the interrupts.
///
/// CLI means Clear Interrupt Flag. Call [`sti`] to enable the interrupts back.
pub fn cli() {
    unsafe {
        asm!("cli");
    }
}

/// Enable the interrupts.
///
/// STI means Set Interrupt Flag. Call [`cli`] to disable the interrupts.
pub fn sti() {
    unsafe {
        asm!("sti");
    }
}

/// Waits for an interrupt to be raised and held.
pub fn halt() {
    unsafe {
        asm!("hlt");
    }
}

/// Stops everything from running.
///
/// Basically it disables interrupts by [`cli`] and waits for an interrupt to
/// come with [`halt`]. as it won't happen, it enters endless idle state.
pub fn cease() {
    cli();
    loop {
        halt();
    }
}
