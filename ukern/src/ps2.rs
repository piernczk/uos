//! A basic 8042 controller driver.
//!
//! 8042 is primarily used to control PS/2 devices, but it's also responsible
//! for things like A20 Adress Line and system reset, for some reason.

use crate::io;

/// The number of the I/O bus port used for sending data to the controller
/// and devices.
pub const DATA_PORT: io::IOPort = 0x60;
/// The number of the I/O bus port used by the controller to report its status.
///
/// Status register consists of several flags, in the order:
/// - output buffer status (0 - empty, 1 - full),
/// - input buffer status (0 - empty, 1 - full),
/// - system flag (cleared on reset),
/// - command/data (either data written to the input buffer is data sent to the
///   device or the controller),
/// - unknown, may be keyboard lock,
/// - unknown,
/// - time-out error occurred,
/// - parity error occurred,
pub const STATUS_PORT: io::IOPort = 0x64;
/// The number of the I/O bus port used to send commands to the controller.
pub const COMMAND_PORT: io::IOPort = 0x64;

/// Whether the controller has two ports or not.
pub static mut DUAL_CHANNEL: bool = false;

/// An error that occurred during the initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitError {
    /// An unknown error that occurred during self-test phase.
    SelfTest,
    /// The first port doesn't work.
    ///
    /// The controller is perfectly usable even with this error, there's no
    /// access to this port though. See [`Self::AllPorts`] for the meaning of
    /// the return code.
    FirstPort(u8),
    /// The second port doesn't work, despite the controller having it.
    ///
    /// The controller is perfectly usable even with this error, there's no
    /// access to this port though. See [`Self::AllPorts`] for the meaning of
    /// the return code.
    SecondPort(u8),
    /// None of the present ports work.
    ///
    /// The first value is the returncode of the first port, and the second one
    /// is the return code of the second port. There are four possible values
    /// for each of them:
    /// - 1 - clock line stuck low
    /// - 2 - clock line stuck high
    /// - 3 - data line stuck low
    /// - 4 - data line stuck high
    AllPorts(u8, u8),
}

/// Initializes the 8042 controller.
///
/// Currently it's borked, but everything works without it (at least in QEMU).
/// For that reason it's private and not invoked anywhere. I left the code just
/// in case it magically starts working tomorrow.
///
/// nvm it works now.
pub fn init() -> Result<(), InitError> {
    send_command(0xAD); // disable the first port
    send_command(0xA7); // disable the second port
    io::inb(DATA_PORT); // flush

    // controller configuration byte
    send_command(0x20);
    let ccb = read_byte();
    let new_ccb = ccb & 0b0011_1101;

    let not_dual_channel = ccb & (1 << 5) == 0;

    send_command(0x60);
    send_byte(new_ccb); // disable all IRQs and translation

    // self test
    send_command(0xAA);
    let test = read_byte();
    if test != 0x55 {
        return Err(InitError::SelfTest); // should be 0xFC
    }

    // could restart
    send_command(0x60);
    send_byte(new_ccb);

    if !not_dual_channel {
        send_command(0xAE); // enable the second port
        send_command(0x20);
        unsafe {
            DUAL_CHANNEL = read_byte() & 1 << 5 == 0;
        }
        send_command(0xA7); // disable it again
    } else {
        unsafe {
            DUAL_CHANNEL = false;
        }
    }

    // test the first port
    send_command(0xAB);
    let mut err = (read_byte(), 0);

    if unsafe { DUAL_CHANNEL } {
        // test the second port
        send_command(0xA9);
        err.1 = read_byte();

        if err.0 != 0 && err.1 != 0 {
            return Err(InitError::AllPorts(err.0, err.1));
        }
    } else if err.0 != 0 {
        return Err(InitError::AllPorts(err.0, 0));
    }

    // enable the first port
    if err.0 == 0 {
        send_command(0xAE);
    }

    // enable the second port
    if err.1 == 0 && unsafe { DUAL_CHANNEL } {
        send_command(0xA8);
    }

    if err.0 != 0 {
        Err(InitError::FirstPort(err.0))
    } else if err.1 != 0 {
        Err(InitError::SecondPort(err.1))
    } else {
        Ok(())
    }
}

/// Sends a command to the controller's [`COMMAND_PORT`].
#[inline]
pub fn send_command(command: u8) {
    wait_write();
    io::outb(COMMAND_PORT, command);
}

/// Waits until one is able to read the [`DATA_PORT`].
pub fn wait_read() {
   let mut status = 0;
    while status & 1 == 0 {
        status = io::inb(STATUS_PORT);
    }
}

/// Waits until one is able to write to the [`DATA_PORT`] and [`COMMAND_PORT`].
pub fn wait_write() {
    let mut status = !0;
    while status & 2 != 0 {
        status = io::inb(STATUS_PORT);
    }
}

/// Reads a byte from the controller's [`DATA_PORT`].
#[inline]
pub fn read_byte() -> u8 {
    wait_read();
    io::inb(DATA_PORT)
}

/// Sends a byte to the controller's [`DATA_PORT`].
#[inline]
pub fn send_byte(data: u8) {
    wait_write();
    io::outb(DATA_PORT, data);
}
