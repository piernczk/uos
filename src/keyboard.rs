//! PS/2 keyboard driver.

use bitflags::bitflags;
use crate::controller;
use num_enum::{TryFromPrimitive, IntoPrimitive};

/// Reads the return data from the keyboard.
pub use controller::read_byte as read_data;

/// Acknowledge byte sent by the keyboard.
pub const ACK: u8 = 0xFA;
/// A byte sent by the keyboard when it asks to resend the command or data.
pub const RESEND: u8 = 0xFE;

/// Sends a command to the keyboard.
///
/// It's a wrapper around [`controller::send_byte`] that also checks for the
/// [`RESEND`] byte.
pub fn send_command(command: u8) {
    let mut res = RESEND;
    while res == RESEND {
        controller::send_byte(command);
        res = controller::read_byte();
    }
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
    read_data().into()
}

/// Represents the current keyboard state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        self.scroll_lock ^ self.shift
    }
}

/// Scan code set 2 keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Set2Key {
    Null, Esc,
    Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9, Digit0,
    Minus, Equals,
    Backspace, Tab,
    Q, W, E, R, T, Y, U, I, O, P, LeftSquare, RightSquare, Enter, LeftControl,
    A, S, D, F, G, H, J, K, L, Semicolon, Apostrophe, Backtick,
    LeftShift, Backslash, Z, X, C, V, B, N, M, Comma, Dot, Slash, RightShift,
    KeypadMult,
    LeftAlt, Space, CapsLock,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10,
    NumLock, ScrollLock,
    Keypad7, Keypad8, Keypad9, KeypadMinus,
    Keypad4, Keypad5, Keypad6, KeypadPlus,
    Keypad1, Keypad2, Keypad3,
    Keypad0, KeypadDot,
    Res0x54, Res0x55, Res0x56,
    F11, F12,
    Res0x59, Res0x5A, Res0x5B, Res0x5C, Res0x5D, Res0x5E, Res0x5F,
    Res0x60, Res0x61, Res0x62, Res0x63, Res0x64, Res0x65, Res0x66, Res0x67,
    Res0x68, Res0x69, Res0x6A, Res0x6B, Res0x6C, Res0x6D, Res0x6E, Res0x6F,
    Res0x70, Res0x71, Res0x72, Res0x73, Res0x74, Res0x75, Res0x76, Res0x77,
    Res0x78, Res0x79, Res0x7A, Res0x7B, Res0x7C, Res0x7D, Res0x7E, Res0x7F,
}

impl Set2Key {
    /// Maps a scan code to a character.
    pub fn map(self, state: KeyboardState) -> Option<u8> {
        let mut b = match self {
            Set2Key::Digit1 => b'1',
            Set2Key::Digit2 => b'2',
            Set2Key::Digit3 => b'3',
            Set2Key::Digit4 => b'4',
            Set2Key::Digit5 => b'5',
            Set2Key::Digit6 => b'6',
            Set2Key::Digit7 => b'7',
            Set2Key::Digit8 => b'8',
            Set2Key::Digit9 => b'9',
            Set2Key::Digit0 => b'0',

            Set2Key::Minus if !state.uppercase() => b'-',
            Set2Key::Minus if state.uppercase() => b'_',
            Set2Key::Equals if !state.uppercase() => b'=',
            Set2Key::Equals if state.uppercase() => b'+',
            Set2Key::Backspace => b'\x08',
            Set2Key::Tab => b'\t',

            Set2Key::Q => b'q',
            Set2Key::W => b'w',
            Set2Key::E => b'e',
            Set2Key::R => b'r',
            Set2Key::T => b't',
            Set2Key::Y => b'y',
            Set2Key::U => b'u',
            Set2Key::I => b'i',
            Set2Key::O => b'o',
            Set2Key::P => b'p',

            Set2Key::LeftSquare => b'[',
            Set2Key::RightSquare => b']',

            Set2Key::Enter => b'\n',

            Set2Key::A => b'a',
            Set2Key::S => b's',
            Set2Key::D => b'd',
            Set2Key::F => b'f',
            Set2Key::G => b'g',
            Set2Key::H => b'h',
            Set2Key::J => b'j',
            Set2Key::K => b'k',
            Set2Key::L => b'l',

            Set2Key::Semicolon if !state.uppercase() => b';',
            Set2Key::Semicolon if state.uppercase() => b':',
            Set2Key::Apostrophe if !state.uppercase() => b'\'',
            Set2Key::Apostrophe if state.uppercase() => b'"',
            Set2Key::Backtick if !state.uppercase() => b'`',
            Set2Key::Backtick if state.uppercase() => b'~',

            Set2Key::Backslash => b'\\',

            Set2Key::Z => b'z',
            Set2Key::X => b'x',
            Set2Key::C => b'c',
            Set2Key::V => b'v',
            Set2Key::B => b'b',
            Set2Key::N => b'n',
            Set2Key::M => b'm',

            Set2Key::Comma if !state.uppercase() => b',',
            Set2Key::Comma if state.uppercase() => b'<',
            Set2Key::Dot if !state.uppercase() => b'.',
            Set2Key::Dot if state.uppercase() => b'>',
            Set2Key::Slash if !state.uppercase() => b'/',
            Set2Key::Slash if state.uppercase() => b'?',

            Set2Key::Space => b' ',

            Set2Key::KeypadMult => b'*',
            Set2Key::KeypadMinus => b'-',
            Set2Key::KeypadPlus => b'+',
            Set2Key::KeypadDot => b'.',

            Set2Key::Keypad1 if state.num_lock => b'1',
            Set2Key::Keypad2 if state.num_lock => b'2',
            Set2Key::Keypad3 if state.num_lock => b'3',
            Set2Key::Keypad4 if state.num_lock => b'4',
            Set2Key::Keypad5 if state.num_lock => b'5',
            Set2Key::Keypad6 if state.num_lock => b'6',
            Set2Key::Keypad7 if state.num_lock => b'7',
            Set2Key::Keypad8 if state.num_lock => b'8',
            Set2Key::Keypad9 if state.num_lock => b'9',
            Set2Key::Keypad0 if state.num_lock => b'0',

            _ => return None,
        };

        if state.uppercase() && b'a' < b && b < b']' {
            b &= !(1 << 5);
        }

        Some(b)
    }
}

/// Whether and which key gets pressed or released.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Pressed(Set2Key),
    Released(Set2Key),
}

impl KeyAction {
    /// Returns the contained key.
    pub fn key(self) -> Set2Key {
        match self {
            KeyAction::Pressed(key) => key,
            KeyAction::Released(key) => key,
        }
    }
}

impl From<u8> for KeyAction {
    fn from(b: u8) -> Self {
        if b & 128 == 0 {
            KeyAction::Pressed(Set2Key::try_from(b).unwrap())
        } else {
            KeyAction::Released(Set2Key::try_from(b & 127).unwrap())
        }
    }
}

impl From<KeyAction> for u8 {
    fn from(action: KeyAction) -> Self {
        match action {
            KeyAction::Pressed(key) => Self::from(key),
            KeyAction::Released(key) => Self::from(key) | 128,
        }
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
    send_command(0xED);
    send_data(leds.bits());
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
    match read_data() {
        0x43 => ScanCodeSet::Set1,
        0x41 => ScanCodeSet::Set2,
        0x3F => ScanCodeSet::Set3,
        _ => ScanCodeSet::Set1,
    }
}

/// Sets scan code set.
#[inline]
pub fn set_scan_code_set(set: ScanCodeSet) {
    send_command(0xF0);
    send_data(set.into());
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
    read_data()
}

/// Performs a self test.
///
/// Returns true on success.
#[inline]
pub fn self_test() -> bool {
    send_command(0xFF);
    read_data() == 0xAA
}
