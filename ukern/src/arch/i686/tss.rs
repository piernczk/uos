use core::arch::asm;

extern "C" {
    pub static mut TSS: Ai686TaskStateSegment;
    pub static mut ESP0: [u8; 16384];
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Ai686TaskStateSegment {
    pub link: u32,

    pub esp0: u32,
    pub ss0: u16,
    res0: u16,
    pub esp1: u32,
    pub ss1: u16,
    res1: u16,
    pub esp2: u32,
    pub ss2: u16,
    res2: u16,

    pub cr3: u32,
    pub eip: u32,
    pub eflags: u32,

    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebx: u32,
    pub esp: u32,
    pub ebp: u32,
    pub esi: u32,
    pub edi: u32,

    pub es: u16,
    res3: u16,
    pub cs: u16,
    res4: u16,
    pub ss: u16,
    res5: u16,
    pub ds: u16,
    res6: u16,
    pub fs: u16,
    res7: u16,
    pub gs: u16,
    res8: u16,
    pub ldtr: u16,
    res9: u16,
    res10: u16,
    pub iopb: u16,
}

pub unsafe fn init() {
    TSS = core::mem::zeroed::<Ai686TaskStateSegment>();
    asm!("ltr ax", in("ax") 0x28);

    TSS.ss0 = 0x10;
    TSS.esp0 = &ESP0 as *const _ as u32;
    TSS.cs = 0x0b;
    TSS.ds = 0x13;
    TSS.es = 0x13;
    TSS.fs = 0x13;
    TSS.gs = 0x13;
    TSS.ss = 0x13;
    TSS.iopb = core::mem::size_of::<Ai686TaskStateSegment>() as u16;
}