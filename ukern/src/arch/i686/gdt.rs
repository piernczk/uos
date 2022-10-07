use crate::arch;

extern "C" {
    #[allow(improper_ctypes)]
    pub static mut GDTR: Ai686GDTRegister;
    pub static mut GDT: [Ai686GDTDescriptor; 6];

    fn load_gdtr();
}

pub unsafe fn init() {
    // null
    GDT[0].set_base(0);
    GDT[0].set_limit(0);
    GDT[0].set_access_byte(0);
    GDT[0].set_flags(0);

    // kernel code
    GDT[1].set_base(0);
    GDT[1].set_limit(0xFFFFF);
    GDT[1].set_access_byte(0x9A);
    GDT[1].set_flags(0xC);

    // kernel data
    GDT[2].set_base(0);
    GDT[2].set_limit(0xFFFFF);
    GDT[2].set_access_byte(0x92);
    GDT[2].set_flags(0xC);

    // user code
    GDT[3].set_base(0);
    GDT[3].set_limit(0xFFFFF);
    GDT[3].set_access_byte(0xFA);
    GDT[3].set_flags(0xC);

    // user data
    GDT[4].set_base(0);
    GDT[4].set_limit(0xFFFFF);
    GDT[4].set_access_byte(0xF2);
    GDT[4].set_flags(0xC);

    // task state segment
    GDT[5].set_base(&arch::tss::TSS as *const _ as u32);
    GDT[5].set_limit(core::mem::size_of::<arch::tss::Ai686TaskStateSegment>() as u32);
    GDT[5].set_access_byte(0x89);
    GDT[5].set_flags(0x0);

    GDTR.size = 0x2F;
    GDTR.offset = &GDT as *const _ as u32;

    load_gdtr();
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Ai686GDTDescriptor {
    limit0: u16,
    base0: u16,
    base1: u8,
    access: u8,
    limit1flags: u8,
    base2: u8,
}

impl Ai686GDTDescriptor {
    pub fn get_limit(&self) -> u32 {
        (self.limit0 as u32) | (((self.limit1flags & 0b1111) as u32) << 16)
    }

    pub fn set_limit(&mut self, limit: u32) {
        self.limit0 = limit as u16;
        self.limit1flags &= 0b1111_0000;
        self.limit1flags |= ((limit >> 16) & 0b1111) as u8;
    }

    pub fn get_base(&self) -> u32 {
        (self.base0 as u32) | ((self.base1 as u32) << 16) | ((self.base2 as u32) << 24)
    }

    pub fn set_base(&mut self, base: u32) {
        self.base0 = base as u16;
        self.base1 = (base >> 16) as u8;
        self.base2 = (base >> 24) as u8;
    }

    pub fn get_access_byte(&self) -> u8 {
        self.access
    }

    pub fn set_access_byte(&mut self, access_byte: u8) {
        self.access = access_byte;
    }

    pub fn get_flags(&self) -> u8 {
        self.limit1flags >> 4
    }

    pub fn set_flags(&mut self, flags: u8) {
        self.limit1flags &= 0b1111;
        self.limit1flags |= flags << 4;
    }
}

#[repr(packed)]
pub struct Ai686GDTRegister {
    pub size: u16,
    pub offset: u32,
}