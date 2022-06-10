//! VGA text mode terminal.

/// VGA text buffer located at `0xB8000`.
pub const BUFFER: *mut u16 = 0xB8000 as *mut u16;
/// VGA text width, which is `80`.
pub const VGA_WIDTH: u16 = 80;
/// VGA text height, which is `25`.
pub const VGA_HEIGHT: u16 = 25;

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
/// Therefore this struct is nothing else than a `u8` with some abstractions.
/// Also, it's [`repr(transparent)`].
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
/// VGA text mode operates on a [buffer](BUFFER), which is in turn a `80x25`
/// grid. Each of its cells is what's known here as an entry. An entry consists
/// of three parts, from the highest byte: background color, foreground color
/// and code point.
///
/// *Note*: VGA text mode uses [code page 737] by default, not ASCII.
///
/// [code page 737]: https://en.wikipedia.org/wiki/Code_page_737
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct VgaEntry {
    pub color: VgaEntryColor,
    pub ch: u8,
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
/// Basically, it's a 2D point limited to a `80x25` rectangle. Due to that,
/// every [`VgaPoint`] can be assigned to a unique index based on a formula:
/// `y * 80 + x`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VgaPoint {
    x: u16,
    y: u16,
}

impl VgaPoint {
    /// Constructs a new [`VgaPoint`].
    ///
    /// # Panics
    ///
    /// The constructor panics when `x` or `y` doesn't fit in the constraints of
    /// VGA text mode, see [`VGA_WIDTH`] and [`VGA_HEIGHT`].
    #[inline]
    pub const fn new(x: u16, y: u16) -> Self {
        assert!(x < VGA_WIDTH);
        assert!(y < VGA_HEIGHT);
        Self { x, y }
    }

    /// Constructs a new [`VgaPoint`] from and index.
    ///
    /// # Panics
    ///
    /// Panics when the index is not smaller than `80 * 25`.
    #[inline]
    pub const fn from_index(index: u16) -> Self {
        assert!(index < VGA_WIDTH * VGA_HEIGHT);
        Self {
            x: index % VGA_WIDTH,
            y: index / VGA_WIDTH,
        }
    }

    /// Jumps to the previous character.
    ///
    /// If it's on the first character of a line, it invokes [`unscroll`].
    pub fn prev(&mut self) {
        if self.x > 0 {
            self.x -= 1;
        } else {
            self.unscroll();
        }
    }

    /// Jumps to the next character.
    ///
    /// If it's on the last character of a line, it invokes [`scroll`].
    pub fn next(&mut self) {
        if self.x < VGA_WIDTH - 1 {
            self.x += 1;
        } else {
            self.scroll();
        }
    }

    /// Scrolls the [`VgaPoint`] to the previous line.
    ///
    /// If it's on the first line, it wraps to the last one.
    pub fn unscroll(&mut self) {
        self.x = VGA_WIDTH - 1;
        if self.y > 0 {
            self.y -= 1;
        } else {
            self.y = VGA_HEIGHT - 1;
        }
    }

    /// Scrolls the [`VgaPoint`] to the next line.
    ///
    /// If it's on the last line, it simply wraps to 0 again.
    pub fn scroll(&mut self) {
        self.x = 0;

        if self.y < VGA_HEIGHT - 1 {
            self.y += 1;
        } else {
            self.y = 0;
        }
    }

    /// Returns the index.
    #[inline]
    pub fn index(self) -> u16 {
        self.y * VGA_WIDTH + self.x
    }

    /// Returns the x coordinate.
    #[inline]
    pub fn x(self) -> u16 {
        self.x
    }

    /// Sets the x coordinate.
    ///
    /// # Panics
    ///
    /// Panics when x is not smaller than 80.
    #[inline]
    pub fn set_x(&mut self, x: u16) {
        assert!(x < VGA_WIDTH);
        self.x = x;
    }

    /// Returns the y coordinate.
    #[inline]
    pub fn y(self) -> u16 {
        self.y
    }

    /// Sets the y coordinate.
    ///
    /// # Panics
    ///
    /// Panics when y is not smaller than 25.
    #[inline]
    pub fn set_y(&mut self, y: u16) {
        assert!(y < VGA_WIDTH);
        self.y = y;
    }
}

/// The current cursor position.
pub static mut CURSOR: VgaPoint = VgaPoint::new(0, 0);
/// The current foreground and background colors.
pub static mut COLOR: VgaEntryColor = VgaEntryColor::new(VgaColor::Black, VgaColor::White);

/// Clears the buffer and sets the cursor to `(0, 0)`.
pub fn clear() {
    unsafe {
        for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
            *BUFFER.add(i as usize) = 0;
        }
        CURSOR = VgaPoint::new(0, 0);
    }
}

/// Scrolls the buffer.
///
/// Every line is moved upwards. The cursor position is updated accordingly.
pub fn scroll() {
    unsafe {
        for i in 0..(VGA_WIDTH * (VGA_HEIGHT - 1)) {
            *BUFFER.add(i as usize) = *BUFFER.add((i + VGA_WIDTH) as usize);
        }

        if CURSOR.y() > 0 {
            CURSOR.set_y(CURSOR.y() - 1);
        }
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

/// Prints a string at the [`CURSOR`].
///
/// It uses the currently set [`COLOR`].
#[inline]
pub fn print(s: &str) {
    unsafe {
        CURSOR = print_at(s, CURSOR)
    }
}

/// Puts an entry at the given point.
///
/// Returns the new cursor position.
pub fn put_entry_at(entry: VgaEntry, mut pos: VgaPoint) -> VgaPoint {
    match entry.ch {
        b'\n' => {
            if pos.y < VGA_HEIGHT - 1 {
                pos.scroll();
            } else {
                pos.y += 1;
            }

            pos.set_x(0);
        },
        b'\t' => pos.x = (pos.x / 8 + 1) * 8,
        b'\x08' => if pos.index() > 0 {
            pos.prev();
            put_entry_at(VgaEntry::new(VgaEntryColor::default(), b' '), pos);
        }
        b'\x0C' => clear(),
        _ => unsafe {
            *BUFFER.add(pos.index() as usize) = entry.into();
            if pos.index() < (VGA_HEIGHT - 1) * VGA_WIDTH {
                pos.next();
            } else {
                scroll();
                pos.set_x(0);
            }
        }
    }

    pos
}

/// Puts an entry at the [`CURSOR`].
pub fn put_entry(entry: VgaEntry) {
    unsafe {
        CURSOR = put_entry_at(entry, CURSOR);
    }
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

/// Puts a character at the [`CURSOR`].
///
/// It uses the currently set [`COLOR`].
#[inline]
pub fn put_char(ch: u8) {
    unsafe {
        put_entry(VgaEntry::new(COLOR, ch))
    }
}
