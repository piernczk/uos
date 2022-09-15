//! PS/2 keyboard driver.

pub mod keycodes;
pub mod layout;
pub mod set2;

use bitflags::bitflags;
use crate::ps2;
use num_enum::{TryFromPrimitive, IntoPrimitive};

use crate::io;
use crate::terminal;

/// Acknowledge byte sent by the keyboard.
pub const ACK: u8 = 0xFA;
/// A byte sent by the keyboard when it asks to resend the command or data.
pub const RESEND: u8 = 0xFE;

pub static mut SCANCODE_SET: ScanCodeSet = ScanCodeSet::Set2;

/// Sends a command to the keyboard.
///
/// It's a wrapper around [`ps2::send_byte`] that also checks for the
/// [`RESEND`] byte.
pub fn send_command(command: u8) {
    let mut res = RESEND;
    while res == RESEND {
        ps2::send_byte(command);
        res = ps2::read_byte();
    }
}

pub fn read_data() -> [u8; 8] {
    let mut bytes = [0u8; 8];
    let mut status = 0;
    while status & 1 == 0 {
        status = io::inb(ps2::STATUS_PORT);
    }

    for i in 0..bytes.len() {
        bytes[i] = io::inb(ps2::DATA_PORT);
        if io::inb(ps2::STATUS_PORT) & 1 == 0 {
            break;
        }
    }

    bytes
}

/// Sends data to the keyboard.
///
/// Basically a single call to [`send_command`] with `inline(always)`.
#[inline(always)]
pub fn send_data(data: u8) {
    send_command(data);
}

/// Waits for the user to press a key and returns it.
#[inline]
pub fn key_action() -> KeyAction {
    let data = read_data();
    let keyaction = match unsafe { SCANCODE_SET } {
        ScanCodeSet::Set2 => {
            if data == set2::PAUSE {
                KeyAction::Released(keycodes::KEY_PAUSE)
            } else if data == set2::PRTSCR_RELEASED {
                KeyAction::Released(keycodes::KEY_PRINT)
            } else if data == set2::PRTSCR_PRESSED {
                KeyAction::Pressed(keycodes::KEY_PRINT)
            } else {
                if data[0] == set2::BYTE2 && data[1] == set2::RELEASE {
                    KeyAction::Released(set2::SCANCODE_SET2_BYTE2[data[2] as usize])
                } else if data[0] == set2::BYTE2 && data[1] != set2::RELEASE {
                    KeyAction::Pressed(set2::SCANCODE_SET2_BYTE2[data[1] as usize])
                } else if data[0] == set2::RELEASE {
                    KeyAction::Released(set2::SCANCODE_SET2[data[1] as usize])
                } else {
                    KeyAction::Pressed(set2::SCANCODE_SET2[data[0] as usize])
                }
            }
        }
        _ => KeyAction::Pressed(keycodes::KEY_RESERVED)
    };
    keyaction
}

/// Represents the current keyboard state.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardState {
    pub scroll_lock: bool,
    pub num_lock: bool,
    pub caps_lock: bool,
    pub shift: bool,
}

impl KeyboardState {
    /// Checks whether to use uppercase letters.
    #[inline(always)]
    pub fn uppercase(&self) -> bool {
        self.caps_lock != self.shift
    }
}

/// Whether and which key gets pressed or released.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Pressed(u16),
    Released(u16),
}

impl KeyAction {
    /// Returns the contained key.
    pub fn key(self) -> u16 {
        match self {
            KeyAction::Pressed(key) => key,
            KeyAction::Released(key) => key,
        }
    }

    pub fn map(self, state: &mut KeyboardState) -> Result<u8, u16> {
        match self {
            Self::Pressed(key) => {
                if let Ok(key_mapped) = layout::map(key, *state) {
                    return Ok(key_mapped);
                } else if key == keycodes::KEY_LEFTSHIFT || key == keycodes::KEY_RIGHTSHIFT {
                    state.shift = true;
                }
            }
            Self::Released(key) => match key {
                keycodes::KEY_CAPSLOCK => state.caps_lock = !state.caps_lock,
                keycodes::KEY_SCROLLLOCK => state.scroll_lock = !state.scroll_lock,
                keycodes::KEY_NUMLOCK => state.num_lock = !state.num_lock,
                keycodes::KEY_LEFTSHIFT | keycodes::KEY_RIGHTSHIFT => state.shift = false,
                _ => {},
            }
        }

        Err(self.key())
    }
}

bitflags! {
    /// Keyboard LED state.
    pub struct LEDState: u8 {
        const SCROLL_LOCK = 1;
        const NUM_LOCK = 2;
        const CAPS_LOCK = 4;
    }
}

impl From<KeyboardState> for LEDState {
    fn from(kbd: KeyboardState) -> Self {
        let mut leds = Self::empty();

        if kbd.scroll_lock {
            leds |= Self::SCROLL_LOCK;
        }
        if kbd.num_lock {
            leds |= Self::NUM_LOCK;
        }
        if kbd.caps_lock {
            leds |= Self::CAPS_LOCK;
        }

        leds
    }
}

/// Sets keyboard's LED state.
#[inline]
pub fn set_led_state(leds: LEDState) {
    send_data(leds.bits());
    send_command(0xED);
}

/// Scan code sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ScanCodeSet {
    Set1 = 1,
    Set2 = 2,
    Set3 = 3,
}

/// Returns the currently used scan code set.
pub fn get_scan_code_set() -> ScanCodeSet {
    send_command(0xF0);
    send_data(0);
    unsafe {
        SCANCODE_SET = match ps2::read_byte() {
            0x43 => ScanCodeSet::Set1,
            0x41 => ScanCodeSet::Set2,
            0x3F => ScanCodeSet::Set3,
            _ => ScanCodeSet::Set2,
        };
        SCANCODE_SET
    }
}

/// Sets scan code set.
#[inline]
pub fn set_scan_code_set(set: ScanCodeSet) {
    send_command(0xF0);
    send_data(set.into());
    unsafe { SCANCODE_SET = set; }
}

/// Delay between the moment when a key is pressed and the moment when it starts
/// getting repeated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum TypematicDelay {
    Delay250ms = 0,
    Delay500ms = 1,
    Delay750ms = 2,
    Delay1000ms = 3,
}

/// Typematic delay and repeat rate combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypematicByte {
    pub repeat_rate: u8,
    pub delay: TypematicDelay,
}

impl From<u8> for TypematicByte {
    fn from(byte: u8) -> Self {
        Self {
            repeat_rate: byte & 32,
            delay: TypematicDelay::try_from((byte >> 5) & 2).unwrap(),
        }
    }
}

impl From<TypematicByte> for u8 {
    fn from(byte: TypematicByte) -> Self {
        (byte.repeat_rate & 5) | (u8::from(byte.delay) << 5)
    }
}

/// Sets the typematic byte.
#[inline]
pub fn set_typematic_byte(byte: TypematicByte) {
    send_command(0xF3);
    send_data(byte.into());
}

/// Enables the keyboard.
#[inline]
pub fn enable() {
    send_command(0xF4);
}

/// Disables the keyboard.
///
/// It can reset the settings.
#[inline]
pub fn disable() {
    send_command(0xF5);
}

/// Resets the keyboard's settings.
#[inline]
pub fn default() {
    send_command(0xF6);
}

/// Resends the last command.
#[inline]
pub fn resend() -> u8 {
    send_command(RESEND);
    ps2::read_byte()
}

/// Performs a self test.
///
/// Returns true on success.
#[inline]
pub fn self_test() -> bool {
    send_command(0xFF);
    ps2::read_byte() == 0xAA
}
