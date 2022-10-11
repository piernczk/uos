//! 8259 PIC, IRQs, exceptions, the Interrupt Descriptor Table and the like.

use crate::io;
use crate::terminal;
use crate::keyboard;
use crate::arch::{pit, util};

extern "C" {
    /// The Interrupt Descriptor Table Register.
    #[allow(improper_ctypes)]
    static mut IDTR: IDTRegister;
    /// The Interrupt Descriptor Table.
    static mut IDT: [GateDescriptor; 256];

    /// The IRQ wrappers.
    #[allow(improper_ctypes)]
    static IRQS: [*const fn(); 16];
    /// The exception wrappers.
    #[allow(improper_ctypes)]
    static EXCEPTIONS: [*const fn(); 32];
}

/// The "base" I/O port of the master Programmable Interrupt Controller.
///
/// The PIC chip, after receiving a hardware interrupt request (IRQ), invokes an
/// interrupt assigned to the IRQ by the kernel. It supports up to 8 unique
/// IRQs per chip. IRQ0..15 map to INT32..47.
///
/// `PIC1` is the master PIC, it defines the following IRQs:
/// - IRQ0 - [PIT](arch::i686::pit)
/// - IRQ1 - PS/2 keyboard
/// - IRQ2 - reserved, here assigned to the slave PIC ([`PIC2`])
/// - IRQ3 - COM2 (serial communication port)
/// - IRQ4 - COM1
/// - IRQ5 - LPT2 (parallel port)
/// - IRQ6 - floppy disk
/// - IRQ7 - LPT1, may be spurious
///
/// See [`PIC1_COMMAND`] and [`PIC1_DATA`].
pub const PIC1: io::IOPort = 0x20;
/// The command I/O port of the master PIC.
pub const PIC1_COMMAND: io::IOPort = PIC1;
/// The data I/O port of the master PIC.
pub const PIC1_DATA: io::IOPort = PIC1 + 1;

/// The "base" I/O port of the slave Programmable Interrupt Controller.
///
/// `PIC2` is the slave PIC and its IRQs always go through IRQ2 of the master
/// PIC. It defines the following IRQs:
/// - IRQ8 - CMOS RTC
/// - IRQ9 - free / legacy SCSI / NIC
/// - IRQ10 - free / SCSI / NIC
/// - IRQ11 - free / SCSI / NIC
/// - IRQ12 - PS/2 mouse
/// - IRQ13 - FPU / co-processor / inter-processor interrupt
/// - IRQ14 - primary ATA HD
/// - IRQ15 - secondary ATA HD
///
/// See [`PIC2_COMMAND`] and [`PIC2_DATA`].
pub const PIC2: io::IOPort = 0xA0;
/// The command I/O port of the slave PIC.
pub const PIC2_COMMAND: io::IOPort = PIC2;
/// The data I/O port of the slave PIC.
pub const PIC2_DATA: io::IOPort = PIC2 + 1;

/// End-of-interrupt signal.
///
/// It is sent to [`PIC1_COMMAND`] or [`PIC2_COMMAND`] (or both) as an
/// acknowlegment after handling the interrupt.
pub const EOI: u8 = 0x20;

pub const ICW1_INIT: u8 = 0x10;
pub const ICW1_ICW4: u8 = 0x01;
pub const ICW4_8086: u8 = 0x01;

/// Whether or not the last keyboard IRQ was a 'release'.
///
/// For some reason, after the key is released the keyboard sends an IRQ without
/// sending any scancode. When this variable is true, the IRQ is just ignored
/// and it's set to false again.
pub static mut LAST_RELEASED: bool = false;

/// Initializes the PICs and the IDT.
pub unsafe fn init() {
    remap_pic(0x20, 0x28);

    for i in 0..32 {
        IDT[i].set_offset(EXCEPTIONS[i] as u32);
        IDT[i].set_gate_type(GateType::Interrupt32);
        IDT[i].set_present(true);
        IDT[i].set_segment(0x08);
    }

    for i in 32..48 {
        IDT[i].set_offset(IRQS[i - 32] as u32);
        IDT[i].set_gate_type(GateType::Interrupt32);
        IDT[i].set_present(true);
        IDT[i].set_segment(0x08);
    }

    IDTR.size = core::mem::size_of::<[GateDescriptor; 256]>() as u16;
    IDTR.offset = &IDT as *const _ as u32;
    core::arch::asm!("lidt [IDTR]");
    util::sti();
}

/// The whole stack generated when an interrupt is raised.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct InterruptArgs {
    pub ds: u32,

    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,

    // TODO make these 2 u8s, add private res fields
    pub intno: u32,
    pub errno: u32,

    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub useresp: u32,
    pub ss: u32,
}

/// The IRQ handler.
#[no_mangle]
pub extern "C" fn irq_handler(args: InterruptArgs) {
    if args.intno == 33 {
        // IRQ1, the keyboard controller IRQ
        if unsafe { LAST_RELEASED } {
            io::inb(crate::ps2::DATA_PORT);
            unsafe {
                LAST_RELEASED = false;
            }
        } else {
            let key_action = keyboard::key_action();
            if let Ok(b) = key_action.map() {
                terminal::put_byte(b);
            }

            if let keyboard::KeyAction::Released(_) = key_action {
                unsafe {
                    LAST_RELEASED = true;
                }
            }
        }
    } else if args.intno == 32 {
        // IRQ0, the PIT IRQ
        unsafe {
            pit::tick();
            io::inb(crate::ps2::DATA_PORT);
        }
    }

    irq_ack(args.intno as u8);
}

/// The exception handler.
#[no_mangle]
pub extern "C" fn exception_handler(args: InterruptArgs) {
    if args.intno == 8 {
        // #DF, Double Fault
        terminal::init();
        terminal::print("double fault\n");
        util::cease();
    }
}

/// Sends [`EOI`] to the PICs.
pub fn irq_ack(irq: u8) {
    if irq >= 8 {
        // send ACK to the slave PIC too
        io::outb(PIC2_COMMAND, EOI);
    }

    io::outb(PIC1_COMMAND, EOI);
}

/// Remaps the PICs to new offsets.
///
/// The IRQ0..15 are assigned to INT0..15 by default, which collides with
/// exceptions' interrupts (INT0..31). Beacuse of that, the PICs need to be
/// remapped somewhere else, usually to INT32..47.
///
/// offset1 and offset2 are the [`PIC1`] and [`PIC2`] new offsets respectively.
pub fn remap_pic(offset1: u8, offset2: u8) {
    // save masks
    let a1 = io::inb(PIC1_DATA);
    let a2 = io::inb(PIC2_DATA);

    // begin PIC init
    io::outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    io::wait();
    io::outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    io::wait();

    // set the offsets
    io::outb(PIC1_DATA, offset1);
    io::wait();
    io::outb(PIC2_DATA, offset2);
    io::wait();

    // tell the PICs the slave PIC is at IRQ2
    io::outb(PIC1_DATA, 4);
    io::wait();
    io::outb(PIC2_DATA, 2);
    io::wait();

    // stuff lol
    io::outb(PIC1_DATA, ICW4_8086);
    io::wait();
    io::outb(PIC2_DATA, ICW4_8086);
    io::wait();

    // restore masks
    io::outb(PIC1_DATA, a1);
    io::outb(PIC2_DATA, a2);
    io::wait();
}

/// The IDT register, loaded with [`lidt`].
#[repr(packed)]
pub struct IDTRegister {
    /// The size of the IDT.
    pub size: u16,
    /// The location of the IDT.
    pub offset: u32,
}

/// The gate type of an interrupt.
///
/// I have no idea what these mean but they are there for a reason i guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GateType {
    Task = 0x5,
    Interrupt16 = 0x6,
    Trap16 = 0x7,
    Interrupt32 = 0xE,
    Trap32 = 0xF,
}

impl TryFrom<u8> for GateType {
    type Error = ();
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        Ok(match byte & 15 {
            0x5 => Self::Task,
            0x6 => Self::Interrupt16,
            0x7 => Self::Trap16,
            0xE => Self::Interrupt32,
            0xF => Self::Trap32,
            _ => return Err(()),
        })
    }
}

/// An IDT entry.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct GateDescriptor {
    offset0: u16,
    segment: u16,
    res: u8,
    flags: u8,
    offset1: u16,
}

impl GateDescriptor {
    /// Returns the location of the interrupt handler.
    pub fn get_offset(&self) -> u32 {
        (self.offset0 as u32) | ((self.offset1 as u32) << 16)
    }

    /// Sets the location of the interrupt handler.
    pub fn set_offset(&mut self, offset: u32) {
        self.offset0 = offset as u16;
        self.offset1 = (offset >> 16) as u16;
    }

    /// Returns the code segment the interrupt handler runs in.
    pub fn get_segment(&self) -> u16 {
        self.segment & !7
    }

    /// Sets the code segment the interrupt handler runs in.
    pub fn set_segment(&mut self, segment: u16) {
        self.segment &= 7;
        self.segment |= segment & !7;
    }

    /// Returns whether or not the segment is the user code segment.
    pub fn is_segment_ring3(&self) -> bool {
        self.segment & 3 == 3
    }

    /// Sets the segment as ring0 (kernel-level).
    pub fn set_segment_ring0(&mut self) {
        self.segment &= !3;
    }

    /// Sets the segment as ring3 (user-level).
    pub fn set_segment_ring3(&mut self) {
        self.segment |= 3;
    }

    /// Returns the [`GateType`].
    pub fn get_gate_type(&self) -> GateType {
        GateType::try_from(self.flags & 15).unwrap()
    }

    /// Sets the [`GateType`].
    pub fn set_gate_type(&mut self, gate: GateType) {
        self.flags &= !15;
        self.flags |= gate as u8;
        if gate == GateType::Task {
            self.offset0 = 0;
            self.offset1 = 0;
        }
    }

    /// Returns whether or not the interrupt can be raised by a user process.
    pub fn is_ring3(&self) -> bool {
        self.flags & 0b0110_0000 == 0b0110_0000
    }

    /// Makes the interrupt callable only by the kernel and hardware.
    pub fn set_ring0(&mut self) {
        self.flags &= !0b0110_0000;
    }

    /// Makes the interrupt callable to every process.
    pub fn set_ring3(&mut self) {
        self.flags |= 0b0110_0000;
    }

    /// Returns whether or not the interrupt is callable.
    pub fn is_present(&self) -> bool {
        self.flags & 128 == 128
    }

    /// Makes the interrupt callable.
    pub fn set_present(&mut self, p: bool) {
        if p {
            self.flags |= 128;
        } else {
            self.flags &= !128;
        }
    }
}
