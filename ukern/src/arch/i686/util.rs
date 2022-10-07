use core::arch::asm;

/*pub fn cpuid() -> CPUID {
    // TODO
}*/

pub fn cli() {
    unsafe {
        asm!("cli");
    }
}

pub fn sti() {
    unsafe {
        asm!("sti");
    }
}

pub fn halt() {
    unsafe {
        asm!("hlt");
    }
}

pub fn cease() {
    cli();
    loop {
        halt();
    }
}
