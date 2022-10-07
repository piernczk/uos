use crate::io;
use crate::arch;

pub const MIN_FREQ: u32 = 18;
pub const MAX_FREQ: u32 = 1193181;

pub const CHANNEL0_PORT: io::IOPort = 0x40;
pub const CHANNEL1_PORT: io::IOPort = 0x41;
pub const CHANNEL2_PORT: io::IOPort = 0x42;
pub const COMMAND_PORT: io::IOPort = 0x43;
pub const PCSPK_PORT: io::IOPort = 0x61;

pub const SELECT_CHANNEL0: u8 = 0b00 << 6;
pub const SELECT_CHANNEL1: u8 = 0b01 << 6;
pub const SELECT_CHANNEL2: u8 = 0b10 << 6;
pub const SELECT_READBACK: u8 = 0b11 << 6;

pub const LATCH_COUNT: u8 = 0b00 << 4;
pub const LOBYTE: u8 = 0b01 << 4;
pub const HIBYTE: u8 = 0b10 << 4;
pub const LOBYTE_HIBYTE: u8 = 0b11 << 4;

pub const INT_TERMINAL: u8 = 0b000 << 1;
pub const HW_RETRIG_ONESHOT: u8 = 0b001 << 1;
pub const RATE_GEN: u8 = 0b010 << 1;
pub const SQUARE_WAVE_GEN: u8 = 0b011 << 1;
pub const SW_STROBE: u8 = 0b100 << 1;
pub const HW_STROBE: u8 = 0b101 << 1;

pub const BCD_MODE: u8 = 1;

pub const PCSPK_POS: u8 = 1;
pub const PCSPK_PIT: u8 = 2;

static mut RELOAD_VALUE: u16 = 1;
static mut FREQ: u32 = MAX_FREQ;
/// Fractions of the reload value
static mut IRQ0_FRACTS: u16 = 0;
static mut IRQ0_MILLIS: u32 = 0;
static mut SYS_FRACTS: u16 = 0;
static mut SYS_MILLIS: u64 = 0;
static mut CNT_FRACTS: u16 = 0;
static mut CNT_MILLIS: u32 = 0;
static mut CNT_TICKS: u32 = 0;
static mut SLEEP_FRACTS: u16 = 0;
static mut SLEEP_MILLIS: u32 = 0;
static mut SLEEP_TICKS: u32 = 0;

pub unsafe fn init(freq: u32) {
    FREQ = freq;
    let old_reload_value = RELOAD_VALUE;
    RELOAD_VALUE = calc_reload_value(freq);

    IRQ0_MILLIS = 1000 / FREQ;
    IRQ0_FRACTS = (1000 % FREQ) as u16;

    // may be wrong but idc
    if old_reload_value != RELOAD_VALUE && old_reload_value != 0 {
        let mult = RELOAD_VALUE / old_reload_value;
        SYS_FRACTS *= mult;
        CNT_FRACTS *= mult;
    }

    // init channel0 to the reload value
    io::outb(COMMAND_PORT, SELECT_CHANNEL0 | LOBYTE_HIBYTE | RATE_GEN);
    io::outb(CHANNEL0_PORT, (RELOAD_VALUE & 0xFF) as u8);
    io::outb(CHANNEL0_PORT, (RELOAD_VALUE >> 8) as u8);
}

pub fn play_sound(freq: u32) {
    // init channel2 to the reload value
    let reload_value = calc_reload_value(freq);
    io::outb(COMMAND_PORT, SELECT_CHANNEL2 | LOBYTE_HIBYTE | SQUARE_WAVE_GEN);
    io::outb(CHANNEL2_PORT, (reload_value & 0xFF) as u8);
    io::outb(CHANNEL2_PORT, (reload_value >> 8) as u8);

    // set the "play sound" fields if they weren't already
    let byte = io::inb(PCSPK_PORT);
    if byte != byte | PCSPK_PIT | PCSPK_POS {
        io::outb(PCSPK_PORT, byte | PCSPK_PIT | PCSPK_POS);
    }
}

pub fn stop_sound() {
    let byte = io::inb(PCSPK_PORT) & !PCSPK_PIT & !PCSPK_POS;
    io::outb(PCSPK_PORT, byte);
}

pub fn calc_reload_value(freq: u32) -> u16 {
    if freq <= MIN_FREQ {
        return 0; // 0 = u16::MAX + 1 due to overflow
    } else if freq >= MAX_FREQ {
        return 1;
    }

    // floored reload value
    let mut reload_value = MAX_FREQ / freq;
    // abs diff between the floored reload value and the actually used frequency
    let diff_floor = MAX_FREQ - reload_value * freq;
    // abs diff between the ceiled reload value and the actually used frequency
    let diff_ceil = (reload_value + 1) * freq - MAX_FREQ;

    if diff_floor > diff_ceil {
        // the desired frequency is closer to one where the reload value is ceiled
        reload_value += 1;
    }

    reload_value as u16
}

pub unsafe fn tick() {
    SYS_FRACTS += IRQ0_FRACTS;
    SYS_MILLIS = SYS_MILLIS.wrapping_add(IRQ0_MILLIS as u64) + (SYS_FRACTS / RELOAD_VALUE) as u64;
SYS_FRACTS %= RELOAD_VALUE;

    CNT_FRACTS += IRQ0_FRACTS;
    CNT_MILLIS = CNT_MILLIS.wrapping_add(IRQ0_MILLIS) + (CNT_FRACTS / RELOAD_VALUE) as u32;
    CNT_FRACTS %= RELOAD_VALUE;

    SLEEP_FRACTS += IRQ0_FRACTS;
    SLEEP_MILLIS = SLEEP_MILLIS.wrapping_add(IRQ0_MILLIS) + (SLEEP_FRACTS / RELOAD_VALUE) as u32;
    SLEEP_FRACTS %= RELOAD_VALUE;

    CNT_TICKS = CNT_TICKS.wrapping_add(1);
    SLEEP_TICKS = SLEEP_TICKS.wrapping_add(1);
}

pub fn reset_counter() {
    unsafe {
        CNT_FRACTS = 0;
        CNT_MILLIS = 0;
        CNT_TICKS = 0;
    }
}

pub fn elapsed() -> u32 {
    unsafe {
        CNT_MILLIS
    }
}

pub fn elapsed_ticks() -> u32 {
    unsafe {
        CNT_TICKS
    }
}

pub fn sleep(millis: u32) {
    unsafe {
        SLEEP_FRACTS = 0;
        SLEEP_MILLIS = 0;
        while SLEEP_MILLIS < millis {
            arch::util::halt();
        }
    }
}

pub fn sleep_ticks(ticks: u32) {
    unsafe {
        SLEEP_TICKS = 0;
        while SLEEP_TICKS < ticks {
            arch::util::halt();
        }
    }
}
