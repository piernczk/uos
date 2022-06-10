//! x86 I/O bus Rust abstration.

use core::arch::asm;

/// x86 I/O bus port type (`u8`).
type IOPort = u8;

/// Gets a byte from the I/O bus.
pub fn inb(port: IOPort) -> u8 {
    let ret;

    unsafe {
        asm!(
            "in al, dx",
            in("dx") port as u16,
            out("al") ret
        );
    }

    ret
} 

/// Sends a byte to the given I/O bus port.
pub fn outb(port: IOPort, data: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port as u16,
            in("al") data
        )
    }
}

/// Gets a word from the I/O bus.
pub fn inw(port: IOPort) -> u16 {
    let ret;

    unsafe {
        asm!(
            "in ax, dx",
            in("dx") port as u16,
            out("ax") ret
        );
    }

    ret
} 

/// Sends a word to the given I/O bus port.
pub fn outw(port: IOPort, data: u16) {
    unsafe {
        asm!(
            "out dx, ax",
            in("dx") port as u16,
            in("ax") data
        )
    }
}

/// Gets a double word from the I/O bus.
pub fn ind(port: IOPort) -> u32 {
    let ret;

    unsafe {
        asm!(
            "in eax, dx",
            in("dx") port as u16,
            out("eax") ret
        );
    }

    ret
} 

/// Sends a double word to the given I/O bus port.
pub fn outd(port: IOPort, data: u32) {
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") port as u16,
            in("eax") data
        )
    }
}
