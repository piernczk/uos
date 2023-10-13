use core::default;

use bitflags::bitflags;
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};

use crate::io::{self, IoPort};

pub const COM_PORTS_COUNT: usize = 8;
static mut COM_PORTS: [Option<ComPort>; COM_PORTS_COUNT] =
    [None, None, None, None, None, None, None, None];
pub const DEFAULT_PORT_ADDRESSES: [IoPort; 8] =
    [0x3F8, 0x2F8, 0x3E8, 0x2E8, 0x5F8, 0x4F8, 0x5E8, 0x4E8];

const CHARACTER_BITS_COUNT_MASK: u8 = 0b11;
const STOP_BITS_COUNT_MASK: u8 = 0b100;
const PARITY_MASK: u8 = 0b111_000;
const DIVISOR_LATCH_MASK: u8 = 0b1000_0000;

const DATA_PORT: IoPort = 0;
const INTERRPUT_ENABLE_PORT: IoPort = 1;
const DIVISOR_LATCH_LOW_PORT: IoPort = 0;
const DIVISOR_LATCH_HIGH_PORT: IoPort = 1;
const INTERRPUT_IDENTIFICATION_PORT: IoPort = 2;
const LINE_CONTROL_PORT: IoPort = 3;
const MODEM_CONTROL_PORT: IoPort = 4;
const LINE_STATUS_PORT: IoPort = 5;
const MODEM_STATUS_PORT: IoPort = 6;
const SCRATCH_PORT: IoPort = 7;

pub unsafe fn com(i: usize) -> Option<&'static mut ComPort> {
    COM_PORTS.get_mut(i).and_then(|x| x.as_mut())
}

pub unsafe fn init_default(i: usize) -> bool {
    if i >= DEFAULT_PORT_ADDRESSES.len() {
        return false;
    }

    init(i, DEFAULT_PORT_ADDRESSES[i])
}

pub unsafe fn init(i: usize, port_address: IoPort) -> bool {
    close(i);
    COM_PORTS[i] = Some(ComPort::new(port_address, LineControlRegister::default()));
    true
}

pub unsafe fn close_all() {
    for i in 0..COM_PORTS_COUNT {
        close(i);
    }
}

pub unsafe fn close(i: usize) -> bool {
    if i >= COM_PORTS_COUNT || COM_PORTS[i].is_none() {
        return false;
    }

    COM_PORTS[i]
        .as_mut()
        .unwrap()
        .set_line_control_register(LineControlRegister::default());
    COM_PORTS[i] = None;
    true
}

#[derive(Debug)]
pub struct ComPort {
    port_address: IoPort,
    line_control_register: LineControlRegister,
}

#[derive(Debug, Clone, Copy)]
pub struct LineControlRegister(u8);

#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
pub enum CharacterBitsCount {
    Bits5 = 0b00,
    Bits6 = 0b01,
    Bits7 = 0b10,
    #[default]
    Bits8 = 0b11,
}

#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
pub enum StopBitsCount {
    #[default]
    Bits1 = 0b000,
    /// 2 or somehow 1.5 for CharacterBitsCount::Bits5
    Bits2 = 0b100,
}

#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
pub enum Parity {
    #[default]
    None = 0b000_000,
    Odd = 0b001_000,
    Even = 0b011_000,
    Mark = 0b101_000,
    Space = 0b111_000,
}

bitflags! {
    pub struct LineStatus: u8 {
        const DATA_READY = 1 << 0;
        const OVERRUN_ERROR = 1 << 1;
        const PARITY_ERROR = 1 << 2;
        const FRAMING_ERROR = 1 << 3;
        const BREAK_INDICATOR = 1 << 4;
        const TRANSMISSION_BUFFER_EMPTY = 1 << 5;
        const TRANSMITTER_EMPTY = 1 << 6;
        const IMPEDING_ERROR = 1 << 7;
    }
}

impl ComPort {
    fn new(port_address: IoPort, line_control_register: LineControlRegister) -> Self {
        let com = Self {
            port_address,
            line_control_register,
        };
        com.flush_line_control_register();
        com
    }

    fn inb(&self, register_port: IoPort) -> u8 {
        io::inb(self.port_address + register_port)
    }

    fn outb(&self, register_port: IoPort, data: u8) {
        io::outb(self.port_address + register_port, data);
    }

    pub fn read(&mut self, tries: u8) -> Option<u8> {
        if self.line_control_register.get_divisor_latch() {
            self.line_control_register.set_divisor_latch(false);
            self.flush_line_control_register();

            let data = self.read_unchecked(tries);

            self.line_control_register.set_divisor_latch(true);
            self.flush_line_control_register();
            data
        } else {
            self.read_unchecked(tries)
        }
    }

    pub fn read_unchecked(&self, tries: u8) -> Option<u8> {
        for _ in 0..tries {
            if self.line_status().contains(LineStatus::DATA_READY) {
                return Some(self.inb(DATA_PORT));
            }
        }
        None
    }

    pub fn write(&mut self, tries: u8, data: u8) -> bool {
        if self.line_control_register.get_divisor_latch() {
            self.line_control_register.set_divisor_latch(false);
            self.flush_line_control_register();

            let res = self.write_unchecked(tries, data);

            self.line_control_register.set_divisor_latch(true);
            self.flush_line_control_register();
            res
        } else {
            self.write_unchecked(tries, data)
        }
    }

    pub fn write_unchecked(&self, tries: u8, data: u8) -> bool {
        for _ in 0..tries {
            while !self
                .line_status()
                .contains(LineStatus::TRANSMISSION_BUFFER_EMPTY)
            {}
            self.outb(DATA_PORT, data);

            if !self.line_status().intersects(
                LineStatus::OVERRUN_ERROR
                    | LineStatus::PARITY_ERROR
                    | LineStatus::FRAMING_ERROR
                    | LineStatus::BREAK_INDICATOR,
            ) {
                return true;
            }
        }
        false
    }

    pub fn read_buf(&mut self, tries: u8, buf: &mut [u8]) {
        // FIXME i think iters don't work now so it simply indexes buf
        for i in 0..buf.len() {
            match self.read(tries) {
                Some(d) => buf[i] = d,
                None => break,
            }
        }
    }

    pub fn write_buf(&mut self, tries: u8, buf: &[u8]) {
        for i in 0..buf.len() {
            self.write(tries, buf[i]);
        }
    }

    pub fn set_divisor_latch(&mut self, latch: u16) {
        if self.line_control_register.get_divisor_latch() {
            self.outb(DIVISOR_LATCH_LOW_PORT, latch as u8);
            self.outb(DIVISOR_LATCH_HIGH_PORT, (latch >> 8) as u8);
        } else {
            self.line_control_register.set_divisor_latch(true);
            self.flush_line_control_register();

            self.outb(DIVISOR_LATCH_LOW_PORT, latch as u8);
            self.outb(DIVISOR_LATCH_HIGH_PORT, (latch >> 8) as u8);

            self.line_control_register.set_divisor_latch(false);
            self.flush_line_control_register();
        }
    }

    pub fn line_status(&self) -> LineStatus {
        self.inb(LINE_STATUS_PORT).into()
    }

    pub fn get_line_control_register(&self) -> LineControlRegister {
        self.line_control_register
    }

    pub fn set_line_control_register(&mut self, line_control_register: LineControlRegister) {
        self.line_control_register = line_control_register;
        self.flush_line_control_register();
    }

    fn flush_line_control_register(&self) {
        self.outb(LINE_CONTROL_PORT, self.line_control_register.into());
    }
}

impl LineControlRegister {
    pub fn new(
        character_bits: CharacterBitsCount,
        stop_bits: StopBitsCount,
        parity: Parity,
        divisor_latch: bool,
    ) -> Self {
        Self(
            u8::from(character_bits)
                | u8::from(stop_bits)
                | u8::from(parity)
                | (u8::from(divisor_latch) * DIVISOR_LATCH_MASK),
        )
    }

    pub fn get_character_bits_count(&self) -> CharacterBitsCount {
        (self.0 & CHARACTER_BITS_COUNT_MASK).try_into().unwrap()
    }

    pub fn set_character_bits_count(&mut self, bits_count: CharacterBitsCount) {
        self.0 &= !CHARACTER_BITS_COUNT_MASK;
        self.0 |= u8::from(bits_count);
    }

    pub fn get_stop_bits_count(&self) -> StopBitsCount {
        (self.0 & STOP_BITS_COUNT_MASK).try_into().unwrap()
    }

    pub fn set_stop_bits_count(&mut self, bits_count: StopBitsCount) {
        self.0 &= !STOP_BITS_COUNT_MASK;
        self.0 |= u8::from(bits_count);
    }

    pub fn get_parity_check(&self) -> Parity {
        (self.0 & PARITY_MASK).try_into().unwrap()
    }

    pub fn set_parity_check(&mut self, parity: Parity) {
        self.0 &= !PARITY_MASK;
        self.0 |= u8::from(parity);
    }

    pub fn get_divisor_latch(&self) -> bool {
        self.0 & DIVISOR_LATCH_MASK != 0
    }

    pub fn set_divisor_latch(&mut self, divisor_latch: bool) {
        self.0 &= !DIVISOR_LATCH_MASK;
        self.0 |= u8::from(divisor_latch);
    }
}

impl Default for LineControlRegister {
    fn default() -> Self {
        Self::new(
            Default::default(),
            Default::default(),
            Default::default(),
            false,
        )
    }
}

impl From<LineControlRegister> for u8 {
    fn from(value: LineControlRegister) -> Self {
        value.0
    }
}

impl TryFrom<u8> for LineControlRegister {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self::new(
            TryFrom::try_from(value & CHARACTER_BITS_COUNT_MASK).map_err(|_| value)?,
            TryFrom::try_from(value & STOP_BITS_COUNT_MASK).map_err(|_| value)?,
            TryFrom::try_from(value & PARITY_MASK).map_err(|_| value)?,
            value & DIVISOR_LATCH_MASK != 0,
        ))
    }
}

impl From<u8> for LineStatus {
    fn from(value: u8) -> Self {
        unsafe { Self::from_bits_unchecked(value) }
    }
}
