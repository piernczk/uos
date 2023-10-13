//! x86 I/O bus Rust abstration.

use core::arch::asm;

/// x86 I/O bus port type.
pub type IoPort = u16;

/// Wait a very small amount of time by writing to `0x80`.
#[inline]
pub fn wait() {
    outb(0x80, 0);
}

/// Gets a byte from the I/O bus.
pub fn inb(port: IoPort) -> u8 {
    let ret;

    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") ret
        );
    }

    ret
}

/// Sends a byte to the given I/O bus port.
pub fn outb(port: IoPort, data: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") data
        )
    }
}

/// Gets a word from the I/O bus.
pub fn inw(port: IoPort) -> u16 {
    let ret;

    unsafe {
        asm!(
            "in ax, dx",
            in("dx") port,
            out("ax") ret
        );
    }

    ret
}

/// Sends a word to the given I/O bus port.
pub fn outw(port: IoPort, data: u16) {
    unsafe {
        asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") data
        )
    }
}

/// Gets a double word from the I/O bus.
pub fn inl(port: IoPort) -> u32 {
    let ret;

    unsafe {
        asm!(
            "in eax, dx",
            in("dx") port,
            out("eax") ret
        );
    }

    ret
}

/// Sends a double word to the given I/O bus port.
pub fn outl(port: IoPort, data: u32) {
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") data
        )
    }
}
