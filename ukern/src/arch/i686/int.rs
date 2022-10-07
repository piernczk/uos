use crate::io;
use crate::terminal;
use crate::keyboard;
use crate::arch::{pit, util};

extern "C" {
    #[allow(improper_ctypes)]
    static mut IDTR: IDTRegister;
    static mut IDT: [GateDescriptor; 256];
    #[allow(improper_ctypes)]
    static IRQS: [*const fn(); 16];
    #[allow(improper_ctypes)]
    static EXCEPTIONS: [*const fn(); 32];
}

pub const PIC1: io::IOPort = 0x20;
pub const PIC1_COMMAND: io::IOPort = PIC1;
pub const PIC1_DATA: io::IOPort = PIC1 + 1;

pub const PIC2: io::IOPort = 0xA0;
pub const PIC2_COMMAND: io::IOPort = PIC2;
pub const PIC2_DATA: io::IOPort = PIC2 + 1;

pub const EOI: u8 = 0x20;

pub const ICW1_INIT: u8 = 0x10;
pub const ICW1_ICW4: u8 = 0x01;
pub const ICW4_8086: u8 = 0x01;

pub static mut LAST_RELEASED: bool = false;

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

    pub intno: u32,
    pub errno: u32,

    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub useresp: u32,
    pub ss: u32,
}

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

#[no_mangle]
pub extern "C" fn exception_handler(args: InterruptArgs) {
    if args.intno == 8 {
        // #DF, Double Fault
        terminal::init();
        terminal::print("double fault\n");
        util::cease();
    }
}

pub fn irq_ack(irq: u8) {
    if irq >= 8 {
        // send ACK to the slave PIC too
        io::outb(PIC2_COMMAND, EOI);
    }

    io::outb(PIC1_COMMAND, EOI);
}

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

#[repr(packed)]
pub struct IDTRegister {
    pub size: u16,
    pub offset: u32,
}

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
    pub fn get_offset(&self) -> u32 {
        (self.offset0 as u32) | ((self.offset1 as u32) << 16)
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.offset0 = offset as u16;
        self.offset1 = (offset >> 16) as u16;
    }

    pub fn get_segment(&self) -> u16 {
        self.segment & !7
    }

    pub fn set_segment(&mut self, segment: u16) {
        self.segment &= 7;
        self.segment |= segment & !7;
    }

    pub fn is_segment_ring3(&self) -> bool {
        self.segment & 3 == 3
    }

    pub fn set_segment_ring0(&mut self) {
        self.segment &= !3;
    }

    pub fn set_segment_ring3(&mut self) {
        self.segment |= 3;
    }

    pub fn get_gate_type(&self) -> GateType {
        GateType::try_from(self.flags & 15).unwrap()
    }

    pub fn set_gate_type(&mut self, gate: GateType) {
        self.flags &= !15;
        self.flags |= gate as u8;
        if gate == GateType::Task {
            self.offset0 = 0;
            self.offset1 = 0;
        }
    }

    pub fn is_ring3(&self) -> bool {
        self.flags & 0b0110_0000 == 0b0110_0000
    }

    pub fn set_ring0(&mut self) {
        self.flags &= !0b0110_0000;
    }

    pub fn set_ring3(&mut self) {
        self.flags |= 0b0110_0000;
    }

    pub fn is_present(&self) -> bool {
        self.flags & 128 == 128
    }

    pub fn set_present(&mut self, p: bool) {
        if p {
            self.flags |= 128;
        } else {
            self.flags &= !128;
        }
    }
}
