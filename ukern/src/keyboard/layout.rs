//! The standard US keyboard layout.
use super::KeyboardState;
use super::keycodes::*;

/// Maps a keycode to its 8-bit character given the keyboard state.
///
/// See [`KeyAction::map`].
pub fn map(keycode: u16, state: KeyboardState) -> Result<u8, u16> {
    let key = match keycode {
        KEY_GRAVE if !state.shift => Ok(b'`'),
        KEY_GRAVE if state.shift => Ok(b'~'),

        KEY_1..=KEY_9 if !state.shift => Ok((keycode - KEY_1) as u8 + b'1'),
        KEY_0 if !state.shift => Ok(b'0'),
        KEY_1 => Ok(b'!'),
        KEY_2 => Ok(b'@'),
        KEY_3 => Ok(b'#'),
        KEY_4 => Ok(b'$'),
        KEY_5 => Ok(b'%'),
        KEY_6 => Ok(b'^'),
        KEY_7 => Ok(b'&'),
        KEY_8 => Ok(b'*'),
        KEY_9 => Ok(b'('),
        KEY_0 => Ok(b')'),

        KEY_MINUS if !state.shift => Ok(b'-'),
        KEY_MINUS if state.shift => Ok(b'_'),
        KEY_EQUAL if !state.shift => Ok(b'='),
        KEY_EQUAL if state.shift => Ok(b'+'),
        KEY_BACKSPACE => Ok(0x08),

        KEY_A => Ok(b'A'),
        KEY_B => Ok(b'B'),
        KEY_C => Ok(b'C'),
        KEY_D => Ok(b'D'),
        KEY_E => Ok(b'E'),
        KEY_F => Ok(b'F'),
        KEY_G => Ok(b'G'),
        KEY_H => Ok(b'H'),
        KEY_I => Ok(b'I'),
        KEY_J => Ok(b'J'),
        KEY_K => Ok(b'K'),
        KEY_L => Ok(b'L'),
        KEY_M => Ok(b'M'),
        KEY_N => Ok(b'N'),
        KEY_O => Ok(b'O'),
        KEY_P => Ok(b'P'),
        KEY_Q => Ok(b'Q'),
        KEY_R => Ok(b'R'),
        KEY_S => Ok(b'S'),
        KEY_T => Ok(b'T'),
        KEY_U => Ok(b'U'),
        KEY_V => Ok(b'V'),
        KEY_W => Ok(b'W'),
        KEY_X => Ok(b'X'),
        KEY_Y => Ok(b'Y'),
        KEY_Z => Ok(b'Z'),

        KEY_LEFTBRACE if !state.shift => Ok(b'['),
        KEY_LEFTBRACE if state.shift => Ok(b'{'),
        KEY_RIGHTBRACE if !state.shift => Ok(b']'),
        KEY_RIGHTBRACE if state.shift => Ok(b'}'),
        KEY_BACKSLASH if !state.shift => Ok(b'\\'),
        KEY_BACKSLASH if state.shift => Ok(b'|'),
        KEY_SEMICOLON if !state.shift => Ok(b';'),
        KEY_SEMICOLON if state.shift => Ok(b':'),
        KEY_APOSTROPHE if !state.shift => Ok(b'\''),
        KEY_APOSTROPHE if state.shift => Ok(b'"'),
        KEY_COMMA if !state.shift => Ok(b','),
        KEY_COMMA if state.shift => Ok(b'<'),
        KEY_DOT if !state.shift => Ok(b'.'),
        KEY_DOT if state.shift => Ok(b'>'),
        KEY_SLASH if !state.shift => Ok(b'/'),
        KEY_SLASH if state.shift => Ok(b'?'),
        KEY_SPACE => Ok(b' '),
        KEY_ENTER => Ok(b'\n'),

        KEY_KPSLASH => Ok(b'/'),
        KEY_KPASTERISK => Ok(b'*'),
        KEY_KPMINUS => Ok(b'-'),
        KEY_KPPLUS => Ok(b'+'),
        KEY_KPDOT => Ok(b','),
        KEY_KPENTER => Ok(b'\n'),

        KEY_KP0 => Ok(b'0'),
        KEY_KP1 => Ok(b'1'),
        KEY_KP2 => Ok(b'2'),
        KEY_KP3 => Ok(b'3'),
        KEY_KP4 => Ok(b'4'),
        KEY_KP5 => Ok(b'5'),
        KEY_KP6 => Ok(b'6'),
        KEY_KP7 => Ok(b'7'),
        KEY_KP8 => Ok(b'8'),
        KEY_KP9 => Ok(b'9'),

        _ => Err(keycode),
    };

    if let Ok(key) = key {
        if b'A' <= key && key <= b'Z' && !state.uppercase() {
            return Ok(key | (1 << 5));
        }
    }

    key
}
