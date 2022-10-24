//! Global Descriptor Table.
//!
//! GDT was a method of memory fragmentation back in the 16-bit computing era.
//! It got superseded by much superior [paging](arch::i686::paging). It's now
//! kept for compatibility and the [TSS](arch::i686::tss).

use crate::arch;

extern "C" {
    /// The Global Descriptor Table Register.
    #[allow(improper_ctypes)]
    pub static mut GDTR: GDTRegister;
    /// The Global Descriptor Table.
    ///
    /// It may consist of up to 8192 segments.  Six of them are used here, in
    /// this order:
    /// - the null segment (0x00)
    /// - the kernel code segment (0x08)
    /// - the kernel data segment (0x10)
    /// - the user code segment (0x18)
    /// - the user data segment (0x20)
    /// - the task state segment (0x28)
    /// - may add more TSSs in the future
    pub static mut GDT: [GDTDescriptor; 6];

    /// Loads the [`GDTR`] and sets the segment registers.
    fn load_gdtr();
}

/// Initializes the GDT.
pub unsafe fn init() {
    // map GDTR to the virtual address of the GDT
    // it was set to physical address before
    GDTR.offset = &GDT as *const _ as u32;

    // set the task state segment
    GDT[5].set_base(&arch::tss::TSS as *const _ as u32);
    GDT[5].set_limit(core::mem::size_of::<arch::tss::TaskStateSegment>() as u32);
    GDT[5].set_access_byte(0x89);
    GDT[5].set_flags(0x0);
    GDTR.size = 0x2F;

    load_gdtr();
}

/// A GDT entry.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct GDTDescriptor {
    limit0: u16,
    base0: u16,
    base1: u8,
    access: u8,
    limit1flags: u8,
    base2: u8,
}

impl GDTDescriptor {
    /// Returns the segment size.
    pub fn get_limit(&self) -> u32 {
        (self.limit0 as u32) | (((self.limit1flags & 0b1111) as u32) << 16)
    }

    /// Sets the segment size.
    pub fn set_limit(&mut self, limit: u32) {
        self.limit0 = limit as u16;
        self.limit1flags &= 0b1111_0000;
        self.limit1flags |= ((limit >> 16) & 0b1111) as u8;
    }

    /// Returns the segment location.
    pub fn get_base(&self) -> u32 {
        (self.base0 as u32) | ((self.base1 as u32) << 16) | ((self.base2 as u32) << 24)
    }

    /// Sets the segment location.
    pub fn set_base(&mut self, base: u32) {
        self.base0 = base as u16;
        self.base1 = (base >> 16) as u8;
        self.base2 = (base >> 24) as u8;
    }

    /// Returns the access byte.
    ///
    /// It specifies the permissions needed to use the segment.
    pub fn get_access_byte(&self) -> u8 {
        self.access
    }

    /// Sets the access byte.
    pub fn set_access_byte(&mut self, access_byte: u8) {
        self.access = access_byte;
    }

    /// Returns the flags.
    pub fn get_flags(&self) -> u8 {
        self.limit1flags >> 4
    }

    /// Sets the flags.
    pub fn set_flags(&mut self, flags: u8) {
        self.limit1flags &= 0b1111;
        self.limit1flags |= flags << 4;
    }
}

/// The GDT register, loaded with [`lgdt`].
#[repr(packed)]
pub struct GDTRegister {
    /// The size of the GDT.
    pub size: u16,
    /// The location of the GDT.
    pub offset: u32,
}