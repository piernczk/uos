use bitflags::bitflags;
use core::ops::Add;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::io;

pub const LOGICAL_SECTOR_SIZE: u64 = 512;

pub const PORT_DATA: io::IoPort = 0x00;
pub const PORT_ERROR: io::IoPort = 0x01;
pub const PORT_FEATURES: io::IoPort = 0x01;
pub const PORT_SECTOR_COUNT: io::IoPort = 0x02;
pub const PORT_SECTOR_NUMBER: io::IoPort = 0x03;
pub const PORT_CYLINDER_LOW: io::IoPort = 0x04;
pub const PORT_CYLINDER_HIGH: io::IoPort = 0x05;
pub const PORT_DRIVE_HEAD: io::IoPort = 0x06;
pub const PORT_STATUS: io::IoPort = 0x07;
pub const PORT_COMMAND: io::IoPort = 0x07;

pub const PORT_STATUS_ALT: io::IoPort = 0x00;
pub const PORT_DEVICE_CONTROL: io::IoPort = 0x00;
pub const PORT_DRIVE_ADDRESS: io::IoPort = 0x01;

pub const PORT_PRIMARY_IO: io::IoPort = 0x1F0;
pub const PORT_PRIMARY_CONTROL: io::IoPort = 0x3F6;
pub const PORT_SECONDARY_IO: io::IoPort = 0x170;
pub const PORT_SECONDARY_CONTROL: io::IoPort = 0x376;

pub static mut DRIVES: Drives = Drives::new();
static mut CURRENT_DRIVE_PRIMARY: DriveNumber = DriveNumber::Second;
static mut CURRENT_DRIVE_SECONDARY: DriveNumber = DriveNumber::Second;

pub fn init() {
    Controller::Primary.select_drive(DriveNumber::First);
    Controller::Primary.select_drive(DriveNumber::Second);
    unsafe {
        DRIVES.init();
    }
}

#[derive(Debug)]
pub struct Drives {
    primary_first: Option<Drive>,
    primary_second: Option<Drive>,
    secondary_first: Option<Drive>,
    secondary_second: Option<Drive>,
}

#[derive(Debug)]
pub struct Drive {
    controller: Controller,
    number: DriveNumber,
    lba48_enabled: bool,
    identify_data: [u16; 256],
}

#[derive(Debug)]
pub enum DriveInitError {
    NoDrive(Controller, DriveNumber),
    DriveNotAta(Controller, DriveNumber),
    LbaNotSupported(Controller, DriveNumber),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DriveNumber {
    First = 0,
    Second = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Controller {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalBlockAddress(u64);

bitflags! {
    pub struct ErrorRegister: u8 {
        /// Address mark not found.
        const AMNF = 0x01;
        /// Track zero not found.
        const TKZNF = 0x02;
        /// Aborted command.
        const ABRT = 0x04;
        /// Media change request.
        const MCR = 0x08;
        /// ID not found.
        const IDNF = 0x10;
        /// Media changed.
        const MC = 0x20;
        /// Uncorrectable data error.
        const UNC = 0x40;
        /// Bad Block detected.
        const BBK = 0x80;
    }
}

bitflags! {
    pub struct StatusRegister: u8 {
        /// Indicates an error occurred.
        const ERR = 0x01;
        /// Index. Always set to zero.
        const IDX = 0x02;
        /// Corrected data. Always set to zero.
        const CORR = 0x04;
        /// Set when the drive has PIO data to transfer, or is ready to accept PIO data.
        const DRQ = 0x08;
        /// Overlapped Mode Service Request.
        const SRV = 0x10;
        /// Drive Write Fault Error (*does not set ERR*).
        const DWF = 0x20;
        /// Clear when drive is spun down, or after an error.
        const DRDY = 0x40;
        /// Indicates the drive is preparing to send/receive data.
        const BSY = 0x80;
    }
}

impl Drives {
    pub const fn new() -> Self {
        Self {
            primary_first: None,
            primary_second: None,
            secondary_first: None,
            secondary_second: None,
        }
    }

    pub fn init(&mut self) {
        if self.primary_first.is_none() {
            self.primary_first = Drive::init(Controller::Primary, DriveNumber::First).ok()
        }
        if self.primary_second.is_none() {
            self.primary_second = Drive::init(Controller::Primary, DriveNumber::Second).ok()
        }
        if self.secondary_first.is_none() {
            self.secondary_first = Drive::init(Controller::Secondary, DriveNumber::First).ok()
        }
        if self.secondary_second.is_none() {
            self.secondary_second = Drive::init(Controller::Secondary, DriveNumber::Second).ok()
        }
    }

    pub fn get(&self, controller: Controller, drive: DriveNumber) -> Option<&Drive> {
        match (controller, drive) {
            (Controller::Primary, DriveNumber::First) => self.primary_first.as_ref(),
            (Controller::Primary, DriveNumber::Second) => self.primary_second.as_ref(),
            (Controller::Secondary, DriveNumber::First) => self.secondary_first.as_ref(),
            (Controller::Secondary, DriveNumber::Second) => self.secondary_second.as_ref(),
        }
    }

    fn get_mut(&mut self, controller: Controller, drive: DriveNumber) -> &mut Option<Drive> {
        match (controller, drive) {
            (Controller::Primary, DriveNumber::First) => &mut self.primary_first,
            (Controller::Primary, DriveNumber::Second) => &mut self.primary_second,
            (Controller::Secondary, DriveNumber::First) => &mut self.secondary_first,
            (Controller::Secondary, DriveNumber::Second) => &mut self.secondary_second,
        }
    }
}

impl Drive {
    fn init(controller: Controller, drive: DriveNumber) -> Result<Self, DriveInitError> {
        controller.select_drive(drive);

        io::outb(controller.get_io_port_base() + PORT_SECTOR_NUMBER, 0);
        io::outb(controller.get_io_port_base() + PORT_CYLINDER_LOW, 0);
        io::outb(controller.get_io_port_base() + PORT_CYLINDER_HIGH, 0);

        // send the IDENTIFY command
        io::outb(controller.get_io_port_base() + PORT_COMMAND, 0xEC);

        let mut status = controller.read_status();
        if status == StatusRegister::empty() {
            return Err(DriveInitError::NoDrive(controller, drive));
        }

        while status.contains(StatusRegister::BSY) {
            status = controller.read_status();
        }

        if io::inb(controller.get_io_port_base() + PORT_CYLINDER_LOW) != 0
            || io::inb(controller.get_io_port_base() + PORT_CYLINDER_HIGH) != 0
        {
            return Err(DriveInitError::DriveNotAta(controller, drive));
        }

        // ERR is always cleared
        while !status.contains(StatusRegister::DRQ) {
            status = controller.read_status();
        }

        let mut result = Self {
            controller,
            number: drive,
            identify_data: [0; 256],
            lba48_enabled: false,
        };

        for entry in result.identify_data.iter_mut() {
            *entry = io::inw(controller.get_io_port_base() + PORT_DATA);
        }

        if result.identify_data[49] & (1 << 9) == 0 {
            return Err(DriveInitError::LbaNotSupported(controller, drive));
        }

        Ok(result)
    }
}

impl Controller {
    fn select_drive(&self, drive: DriveNumber) {
        if self.current_drive_number() != drive {
            let old = !(1 << 4) & io::inb(self.get_io_port_base() + PORT_DRIVE_HEAD);
            io::outb(
                self.get_io_port_base() + PORT_DRIVE_HEAD,
                old | (u8::from(drive) << 4),
            );
            self.set_current_drive_number(drive);
        }
    }

    pub fn read_status(&self) -> StatusRegister {
        unsafe {
            StatusRegister::from_bits_unchecked(io::inb(self.get_io_port_base() + PORT_STATUS))
        }
    }

    pub fn read_error(&self) -> ErrorRegister {
        unsafe { ErrorRegister::from_bits_unchecked(io::inb(self.get_io_port_base() + PORT_ERROR)) }
    }

    pub fn get_io_port_base(&self) -> io::IoPort {
        match self {
            Self::Primary => PORT_PRIMARY_IO,
            Self::Secondary => PORT_SECONDARY_IO,
        }
    }

    pub fn get_command_port_base(&self) -> io::IoPort {
        match self {
            Self::Primary => PORT_PRIMARY_IO,
            Self::Secondary => PORT_SECONDARY_IO,
        }
    }

    pub fn current_drive_number(&self) -> DriveNumber {
        unsafe {
            match self {
                Self::Primary => CURRENT_DRIVE_PRIMARY,
                Self::Secondary => CURRENT_DRIVE_SECONDARY,
            }
        }
    }

    fn set_current_drive_number(&self, drive: DriveNumber) {
        unsafe {
            match self {
                Self::Primary => CURRENT_DRIVE_PRIMARY = drive,
                Self::Secondary => CURRENT_DRIVE_SECONDARY = drive,
            }
        }
    }

    pub fn current_drive(&self) -> Option<&'static Drive> {
        self.drive(self.current_drive_number())
    }

    fn current_drive_mut(&self) -> &'static mut Option<Drive> {
        self.drive_mut(self.current_drive_number())
    }

    pub fn drive(&self, drive: DriveNumber) -> Option<&'static Drive> {
        unsafe { DRIVES.get(*self, drive) }
    }

    fn drive_mut(&self, drive: DriveNumber) -> &'static mut Option<Drive> {
        unsafe { DRIVES.get_mut(*self, drive) }
    }
}

impl LogicalBlockAddress {
    pub fn new24(low: u8, mid: u8, high: u8) -> Self {
        Self((low as u64) | ((mid as u64) << 8) | ((high as u64) << 16))
    }

    pub fn new48(low: u16, mid: u16, high: u16) -> Self {
        Self((low as u64) | ((mid as u64) << 16) | ((high as u64) << 32))
    }

    pub fn to24(&self) -> Option<(u8, u8, u8)> {
        if self.0 >= 1 << 24 {
            None
        } else {
            Some(self.to24_wrapped())
        }
    }

    pub fn to24_wrapped(&self) -> (u8, u8, u8) {
        (self.0 as u8, (self.0 >> 8) as u8, (self.0 >> 16) as u8)
    }

    pub fn to48(&self) -> Option<(u16, u16, u16)> {
        if self.0 >= 1 << 48 {
            None
        } else {
            Some((self.0 as u16, (self.0 >> 16) as u16, (self.0 >> 32) as u16))
        }
    }
}

impl Add for LogicalBlockAddress {
    type Output = LogicalBlockAddress;

    fn add(self, rhs: Self) -> Self::Output {
        // TODO make it fail if the sum is >= 24 or 48
        // probably redo the whole struct to take a const parameter u8 or 16
        Self(self.0 + rhs.0)
    }
}

impl TryFrom<u64> for LogicalBlockAddress {
    type Error = u64;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value >= 1 << 48 {
            Err(value)
        } else {
            Ok(Self(value))
        }
    }
}

impl From<LogicalBlockAddress> for u64 {
    fn from(value: LogicalBlockAddress) -> Self {
        value.0
    }
}
