//! The Task State Segment.
//!
//! The TSS is used for multitasking which is not yet implemented here.

use core::arch::asm;

/// The TSS stack.
pub static mut ESP0: [u8; 16 * 1024] = [0; 16 * 1024];

/// The Task State Segment.
///
/// There is usually one TSS per process (the old way, there are too few
/// segments in the [GDT](crate::arch::gdt)) or one per CPU.
pub static mut TSS: TaskStateSegment = TaskStateSegment {
    link: 0,
    esp0: 0,
    ss0: 0,
    res0: 0,
    esp1: 0,
    ss1: 0,
    res1: 0,
    esp2: 0,
    ss2: 0,
    res2: 0,
    cr3: 0,
    eip: 0,
    eflags: 0,
    eax: 0,
    ecx: 0,
    edx: 0,
    ebx: 0,
    esp: 0,
    ebp: 0,
    esi: 0,
    edi: 0,
    es: 0,
    res3: 0,
    cs: 0,
    res4: 0,
    ss: 0,
    res5: 0,
    ds: 0,
    res6: 0,
    fs: 0,
    res7: 0,
    gs: 0x13,
    res8: 0,
    ldtr: 0,
    res9: 0,
    res10: 0,
    iopb: core::mem::size_of::<TaskStateSegment>() as u16,
};

/// The Task State Segment structure.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct TaskStateSegment {
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

/// Initializes the TSS.
pub unsafe fn init() {
    asm!("ltr ax", in("ax") 0x28);
    TSS.cs = 0x0b;
    TSS.ss0 = 0x10;
    TSS.es = 0x13;
    TSS.ss = 0x13;
    TSS.ds = 0x13;
    TSS.fs = 0x13;
    TSS.gs = 0x13;
    TSS.esp0 = &ESP0 as *const _ as u32;
}