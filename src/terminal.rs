pub const BUFFER: *mut u16 = 0xB8000 as *mut u16;
pub const VGA_WIDTH: u16 = 80;
pub const VGA_HEIGHT: u16 = 25;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VgaEntryColor(pub u8);

impl VgaEntryColor {
    #[inline]
    pub const fn new(background: VgaColor, foreground: VgaColor) -> Self {
        Self((foreground as u8) | ((background as u8) << 4))
    }

    #[inline]
    pub fn set_foreground(&mut self, foreground: VgaColor) {
        self.0 &= !15 | foreground as u8
    }

    #[inline]
    pub fn set_background(&mut self, background: VgaColor) {
        self.0 &= 15 | (background as u8) << 4
    }

    #[inline]
    pub fn foreground(self) -> VgaColor {
        VgaColor::try_from(self.0 | 15).unwrap()
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct VgaEntry {
    pub color: VgaEntryColor,
    pub ch: u8,
}

impl VgaEntry {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VgaPoint {
    x: u16,
    y: u16,
}

impl VgaPoint {
    #[inline]
    pub const fn new(x: u16, y: u16) -> Self {
        assert!(x < VGA_WIDTH);
        assert!(y < VGA_HEIGHT);
        Self { x, y }
    }

    #[inline]
    pub const fn from_index(index: u16) -> Self {
        assert!(index < VGA_WIDTH * VGA_HEIGHT);
        Self {
            x: index % VGA_WIDTH,
            y: index / VGA_WIDTH,
        }
    }

    pub fn next(&mut self) {
        if self.y < VGA_WIDTH - 1 {
            self.x += 1;
        } else {
            self.scroll();
        }
    }

    pub fn scroll(&mut self) {
        self.x = 0;

        if self.y < VGA_HEIGHT - 1 {
            self.y += 1;
        } else {
            self.y = 0;
        }
    }

    #[inline]
    pub fn index(self) -> u16 {
        self.y * VGA_WIDTH + self.x
    }

    #[inline]
    pub fn x(self) -> u16 {
        self.x
    }

    #[inline]
    pub fn set_x(&mut self, x: u16) {
        assert!(x < VGA_WIDTH);
        self.x = x;
    }

    #[inline]
    pub fn y(self) -> u16 {
        self.y
    }

    #[inline]
    pub fn set_y(&mut self, y: u16) {
        assert!(y < VGA_WIDTH);
        self.y = y;
    }
}

pub static mut CURSOR: VgaPoint = VgaPoint::new(0, 0);
pub static mut COLOR: VgaEntryColor = VgaEntryColor::new(VgaColor::Black, VgaColor::White);

pub fn clear() {
    unsafe {
        for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
            *BUFFER.add(i as usize) = 0;
        }
        CURSOR = VgaPoint::new(0, 0);
    }
}

pub fn scroll() {
    unsafe {
        for i in 0..(VGA_WIDTH * (VGA_HEIGHT - 1)) {
            *BUFFER.add(i as usize) = *BUFFER.add((i + VGA_WIDTH) as usize);
        }
    }
}

pub fn print_at(s: &str, mut pos: VgaPoint) -> VgaPoint {
    for b in s.bytes() {
        put_char_at(b, pos);
        if pos.index() < (VGA_HEIGHT - 1) * VGA_WIDTH {
            pos.next();
        } else {
            scroll();
            pos.set_x(0);
        }
    }
    pos
}

#[inline]
pub fn print(s: &str) -> VgaPoint {
    unsafe {
        print_at(s, CURSOR)
    }
}

#[inline]
pub fn put_entry_at(entry: VgaEntry, pos: VgaPoint) {
    unsafe {
        *BUFFER.add(pos.index() as usize) = entry.into();
    }
}

pub fn put_entry(entry: VgaEntry) {
    unsafe {
        put_entry_at(entry, CURSOR);
        if CURSOR.index() < (VGA_HEIGHT - 1) * VGA_WIDTH {
            CURSOR.next();
        } else {
            scroll();
            CURSOR.set_x(0);
        }
    }
}

#[inline]
pub fn put_char_at(ch: u8, pos: VgaPoint) {
    unsafe {
        put_entry_at(VgaEntry::new(COLOR, ch), pos);
    }
}

#[inline]
pub fn put_char(ch: u8) {
    unsafe {
        put_entry(VgaEntry::new(COLOR, ch))
    }
}
