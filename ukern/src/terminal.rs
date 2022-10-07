//! VGA text mode terminal.

use crate::io;

/// VGA text buffer located at `0xB8000`.
pub const BUFFER: *mut VgaEntry = 0xB8000 as *mut VgaEntry;
/// VGA text width.
pub const VGA_WIDTH: u8 = 80;
/// VGA text height.
pub const VGA_HEIGHT: u8 = 25;

/// VGA text mode palette.
///
/// There are overall 16 4-bit colors. The MSB is a modifier brightness bit,
/// while the last three bits represent the actual color used.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VgaColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

impl TryFrom<u8> for VgaColor {
    type Error = ();

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        Ok(match byte {
            0 => VgaColor::Black,
            1 => VgaColor::Blue,
            2 => VgaColor::Green,
            3 => VgaColor::Cyan,
            4 => VgaColor::Red,
            5 => VgaColor::Magenta,
            6 => VgaColor::Brown,
            7 => VgaColor::LightGray,
            8 => VgaColor::DarkGray,
            9 => VgaColor::LightBlue,
            10 => VgaColor::LightGreen,
            11 => VgaColor::LightCyan,
            12 => VgaColor::LightRed,
            13 => VgaColor::LightMagenta,
            14 => VgaColor::LightBrown,
            15 => VgaColor::White,
            _ => return Err(()),
        })
    }
}

impl From<VgaColor> for u8 {
    fn from(color: VgaColor) -> Self {
        color as u8
    }
}

/// VGA text foreground and background color.
///
/// As one color can be represented with only 4 bits (see [`VgaColor`]), the
/// foreground and background colors need only one byte. The 4 higher bits are
/// the background color and the 4 lower bits are the foreground color.
///
/// Therefore this struct is nothing else than a `repr(transparent)` `u8` with
/// some abstractions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VgaEntryColor(pub u8);

impl VgaEntryColor {
    /// Constructs a new VGA entry color from background and foreground colors.
    #[inline]
    pub const fn new(background: VgaColor, foreground: VgaColor) -> Self {
        Self((foreground as u8) | ((background as u8) << 4))
    }

    /// Sets the foreground.
    #[inline]
    pub fn set_foreground(&mut self, foreground: VgaColor) {
        self.0 &= !15 | foreground as u8
    }

    /// Sets the background.
    #[inline]
    pub fn set_background(&mut self, background: VgaColor) {
        self.0 &= 15 | (background as u8) << 4
    }

    /// Returns the foreground.
    #[inline]
    pub fn foreground(self) -> VgaColor {
        VgaColor::try_from(self.0 | 15).unwrap()
    }

    /// Returns the background.
    #[inline]
    pub fn background(self) -> VgaColor {
        VgaColor::try_from(self.0 >> 4).unwrap()
    }
}

impl Default for VgaEntryColor {
    fn default() -> Self {
        Self::new(VgaColor::Black, VgaColor::White)
    }
}

impl From<u8> for VgaEntryColor {
    fn from(code: u8) -> Self {
        Self(code)
    }
}

impl From<VgaEntryColor> for u8 {
    fn from(color: VgaEntryColor) -> Self {
        color.0
    }
}

/// A single VGA text entry.
///
/// VGA text mode operates on a [buffer](BUFFER), which is by default a `80x25`
/// grid. Each of its cells is what's known here as an entry. An entry consists
/// of three parts, from the highest bit: background color (4 bits), foreground
/// color (4 bits) and code point (8 bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct VgaEntry {
    pub ch: u8,
    pub color: VgaEntryColor,
}

impl VgaEntry {
    /// Constructs a new VGA text entry.
    #[inline]
    pub const fn new(color: VgaEntryColor, ch: u8) -> Self {
        Self { color, ch }
    }
}

impl From<VgaEntry> for u16 {
    fn from(entry: VgaEntry) -> Self {
        (u8::from(entry.color) as u16) << 8 | entry.ch as u16
    }
}

/// Coordinates of a single VGA entry.
///
/// Basically, it's a 2D point limited to a [`VGA_WIDTH`]`x`[`VGA_HEIGHT`]
/// rectangle. Due to that, every [`VgaPoint`] can be assigned to a unique index
/// based on a formula: `y * VGA_WIDTH + x`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VgaPoint {
    index: u16,
}

impl VgaPoint {
    /// Constructs a new [`VgaPoint`].
    ///
    /// # Panics
    ///
    /// The constructor panics when `x` or `y` doesn't fit in the constraints,
    /// see [`VGA_WIDTH`] and [`VGA_HEIGHT`].
    #[inline]
    pub const fn new(x: u8, y: u8) -> Self {
        assert!(x < VGA_WIDTH);
        assert!(y < VGA_HEIGHT);
        Self { index: (y as u16) * (VGA_WIDTH as u16) + (x as u16) }
    }

    /// Constructs a new [`VgaPoint`] from an index.
    ///
    /// # Panics
    ///
    /// Panics when the index is not smaller than `VGA_WIDTH * VGA_HEIGHT`.
    #[inline]
    pub const fn from_index(index: u16) -> Self {
        assert!(index < (VGA_WIDTH as u16) * (VGA_HEIGHT as u16));
        Self { index }
    }

    /// Jumps to the previous character.
    ///
    /// If it's on the first character of a line, it invokes [`Self::unscroll`].
    pub fn prev(&mut self) {
        if self.x() > 0 {
            self.index -= 1;
        } else {
            self.unscroll();
            self.index += VGA_WIDTH as u16 - 1;
        }
    }

    /// Jumps to the next character.
    ///
    /// If it's on the last character of a line, it invokes [`Self::scroll`].
    pub fn next(&mut self) {
        if self.x() != VGA_WIDTH - 1 {
            self.index += 1;
        } else {
            self.scroll();
            self.index -= self.x() as u16;
        }
    }

    /// Scrolls the [`VgaPoint`] to the previous line.
    ///
    /// If it's on the first line, it wraps to the last one.
    pub fn unscroll(&mut self) {
        if self.y() > 0 {
            self.index -= VGA_WIDTH as u16;
        } else {
            self.index = (self.x() as u16) + (VGA_WIDTH as u16) * (VGA_HEIGHT as u16 - 1);
        }
    }

    /// Scrolls the [`VgaPoint`] to the next line.
    ///
    /// If it's on the last line, it simply wraps to 0 again.
    pub fn scroll(&mut self) {
        if self.y() < VGA_HEIGHT - 1 {
            self.index += VGA_WIDTH as u16;
        } else {
            self.index = self.x() as u16;
        }
    }

    /// Returns the index.
    #[inline]
    pub fn index(self) -> u16 {
        self.index
    }

    pub fn set_index(&mut self, index: u16) {
        assert!(index < VGA_WIDTH as u16 * VGA_HEIGHT as u16);
        self.index = index;
    }

    /// Returns the x coordinate.
    #[inline]
    pub fn x(self) -> u8 {
        (self.index % (VGA_WIDTH as u16)) as u8
    }

    /// Sets the x coordinate.
    ///
    /// # Panics
    ///
    /// Panics when `x >= VGA_WIDTH`.
    #[inline]
    pub fn set_x(&mut self, x: u8) {
        assert!(x < VGA_WIDTH);
        self.index -= self.x() as u16;
        self.index += x as u16;
    }

    /// Returns the y coordinate.
    #[inline]
    pub fn y(self) -> u8 {
        (self.index / (VGA_WIDTH as u16)) as u8
    }

    /// Sets the y coordinate.
    ///
    /// # Panics
    ///
    /// Panics when `y >= VGA_HEIGHT`.
    #[inline]
    pub fn set_y(&mut self, y: u8) {
        assert!(y < VGA_HEIGHT);
        self.index = (y as u16) * (VGA_WIDTH as u16) + (self.x() as u16);
    }
}

/// Whether the graphical cursor is enabled or not.
static mut CURSOR_ENABLED: bool = false;
/// The current cursor position.
static mut CURSOR: VgaPoint = VgaPoint::new(0, 0);
/// The current foreground and background colors.
pub static mut COLOR: VgaEntryColor = VgaEntryColor::new(VgaColor::Black, VgaColor::White);

/// Set the cursor position.
///
/// If the graphical cursor is enabled, it automatically updates its position.
pub fn set_cursor(pos: VgaPoint) {
    unsafe {
        CURSOR = pos;
        if CURSOR_ENABLED {
            io::outb(0x3D4, 0x0F);
            io::outb(0x3D5, pos.index() as u8);

            io::outb(0x3D4, 0x0E);
            io::outb(0x3D5, (pos.index() >> 8) as u8);
        }
    }
}

/// Returns the cursor position.
#[inline(always)]
pub fn get_cursor() -> VgaPoint {
    unsafe { CURSOR }
}

/// Clears the buffer and sets the cursor to `(0, 0)`.
pub fn clear() {
    unsafe {
        for i in 0..(VGA_WIDTH as u16 * VGA_HEIGHT as u16) {
            *BUFFER.add(i as usize) = VgaEntry::new(COLOR, 0);
        }
    }
    set_cursor(VgaPoint::new(0, 0));
}

/// Enables the graphical cursor.
pub fn enable_cursor() {
	io::outb(0x3D4, 0x0A);
	io::outb(0x3D5, 0);
 
	io::outb(0x3D4, 0x0B);
	io::outb(0x3D5, 31);

    unsafe {
        CURSOR_ENABLED = true;
    }

    set_cursor(get_cursor()); // refresh the position
}

/// Disables the graphical cursor.
pub fn disable_cursor() {
    io::outb(0x3D4, 0x0A);
    io::outb(0x3D5, 0x20);

    unsafe {
        CURSOR_ENABLED = false;
    }
}

/// Whether the graphical cursor is enabled or not.
pub fn is_cursor_enabled() -> bool {
    unsafe { CURSOR_ENABLED }
}

/// Initialize the terminal.
pub fn init() {
    enable_cursor();
    clear();
}

/// Scrolls the buffer.
///
/// Every line is moved upwards. The cursor position is updated accordingly.
pub fn scroll() {
    unsafe {
        let last_line = VGA_WIDTH as u16 * (VGA_HEIGHT as u16 - 1);
        for i in (VGA_WIDTH as u16)..last_line {
            *BUFFER.add(i as usize) = *BUFFER.add(i as usize + VGA_WIDTH as usize);
        }

        for i in 0..VGA_WIDTH {
            *BUFFER.add((i as u16) as usize) = VgaEntry::new(COLOR, 0);
            *BUFFER.add((i as u16 + last_line) as usize) = VgaEntry::new(COLOR, 0);
        }
    }

    let mut cursor = get_cursor();
    if cursor.y() > 0 {
        cursor.set_y(cursor.y() - 1);
        set_cursor(cursor);
    }
}

/// Prints a string from the given point.
///
/// It uses the currently set [`COLOR`] and returns the new cursor position.
pub fn print_at(s: &str, mut pos: VgaPoint) -> VgaPoint {
    for b in s.bytes() {
        pos = put_char_at(b, pos);
    }
    pos
}

/// Prints a string at the cursor position.
///
/// It uses the currently set [`COLOR`].
#[inline]
pub fn print(s: &str) {
    set_cursor(print_at(s, get_cursor()));
}

/// Puts an entry at the given point.
///
/// Returns the new cursor position.
pub fn put_entry_at(entry: VgaEntry, mut pos: VgaPoint) -> VgaPoint {
    match entry.ch {
        b'\n' => {
            put_entry_at(VgaEntry::new(entry.color, b' '), pos);
            if pos.y() < VGA_HEIGHT - 1 {
                pos.scroll();
            } else {
                scroll();
            }

            pos.set_x(0);
        },
        b'\t' => pos.set_x((pos.x() / 8 + 1) * 8),
        b'\x08' => if pos.index() > 0 {
            loop {
                pos.prev();
                if unsafe { (*BUFFER.add(pos.index() as usize)).ch } != 0 {
                    break;
                }
            }
            put_entry_at(VgaEntry::new(VgaEntryColor::default(), 0), pos);
        }
        b'\x0C' => clear(),
        _ => unsafe {
            *BUFFER.add(pos.index() as usize) = entry.into();
            pos.next();
            if pos.y() >= VGA_HEIGHT - 1 {
                scroll();
                pos.unscroll();
            }
        }
    }

    pos
}

/// Puts an entry at the cursor position.
pub fn put_entry(entry: VgaEntry) {
    set_cursor(put_entry_at(entry, get_cursor()));
}

/// Puts a character at the given point.
///
/// It uses the currently set [`COLOR`]. Returns the new cursor position.
#[inline]
pub fn put_char_at(ch: u8, pos: VgaPoint) -> VgaPoint {
    unsafe {
        put_entry_at(VgaEntry::new(COLOR, ch), pos)
    }
}

/// Puts a character at the cursor position.
///
/// It uses the currently set [`COLOR`].
#[inline]
pub fn put_char(ch: u8) {
    unsafe {
        put_entry(VgaEntry::new(COLOR, ch))
    }
}

pub fn put_byte(ch: u8) {
    unsafe {
        *BUFFER.add(CURSOR.index() as usize) = VgaEntry::new(COLOR, ch).into();
        CURSOR.next();
        if CURSOR.y() >= VGA_HEIGHT - 1 {
            scroll();
            CURSOR.unscroll();
        }
    }
}
