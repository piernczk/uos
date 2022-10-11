//! i686 paging.
//!
//! The highest [`PageMap`] on i686 is the [`PageDirectory`]. It has 1024
//! [`PageDirectoryEntry`]es, each pointing to a [`PageTable`]. They also have
//! 1024 entries ([`PageTagleEntry`]), but they point to 4 KiB pages. This means
//! that i686 supports up to `1024 * 1024 * 4 KiB = 4 GiB` virtual memory
//! (assuming the PAE is disabled, and it is for now).

use crate::arch::{self, generic::paging::*};

pub const PDE_MASK_ADDR: u32 = !0b1111_1111_1111;
pub const PDE_MASK_AVL: u32 = 0b1111_0100_0000;

pub const PDE_FLAG_P: u32 = 1 << 0;
pub const PDE_FLAG_RW: u32 = 1 << 1;
pub const PDE_FLAG_US: u32 = 1 << 2;
pub const PDE_FLAG_PWT: u32 = 1 << 3;
pub const PDE_FLAG_PCD: u32 = 1 << 4;
pub const PDE_FLAG_A: u32 = 1 << 5;
pub const PDE_FLAG_PS: u32 = 1 << 7;

pub const PTE_MASK_ADDR: u32 = !0b1111_1111_1111;
pub const PTE_MASK_AVL: u32 = 0b1110_0000_0000;

pub const PTE_FLAG_P: u32 = 1 << 0;
pub const PTE_FLAG_RW: u32 = 1 << 1;
pub const PTE_FLAG_US: u32 = 1 << 2;
pub const PTE_FLAG_PWT: u32 = 1 << 3;
pub const PTE_FLAG_PCD: u32 = 1 << 4;
pub const PTE_FLAG_A: u32 = 1 << 5;
pub const PTE_FLAG_D: u32 = 1 << 6;

/// How many page directories to allocate for the kernel.
///
/// A temporary thing.
pub const KERNEL_ALLOC: usize = 2;

extern "C" {
    static mut PAGE_DIRECTORY: PageDirectory;
    static mut PAGE_TABLES: [PageTable; 1023];

    fn enable_paging();
}

pub type HighestMap = PageDirectory;
pub type HighestEntry = PageDirectoryEntry;

unsafe fn init_paging() {
    // init paging
    for (i, pde) in (&mut PAGE_DIRECTORY.0).into_iter().take(1023).enumerate() {
        pde.init();
        pde.set_pointer(&mut PAGE_TABLES[i] as *mut _);

        for (j, pte) in (&mut PAGE_TABLES[i].0).into_iter().enumerate() {
            pte.init();
            pte.set_pointer(((i << 22) | (j << 12)) as *mut Page);
        }
    }

    // setup recursive paging
    PAGE_DIRECTORY.0[1023].init();
    PAGE_DIRECTORY.0[1023].set_pointer(&mut PAGE_DIRECTORY as *mut _ as *mut PageTable);

    // alloc first few MiB for the kernel
    // TODO auto detect the count of needed pages in the linker script
    for (i, pde) in (&mut PAGE_DIRECTORY.0).into_iter().take(KERNEL_ALLOC).enumerate() {
        pde.set_supervisor_only(true);
        pde.set_present(true);

        for pte in &mut PAGE_TABLES[i].0 {
            pte.set_supervisor_only(true);
            pte.set_present(true);
        }
    }

    enable_paging();
}

/// Initializes paging, the [GDT](arch::gdt) and the [TSS](arch::tss).
///
/// Returns the address of the page directory.
pub unsafe fn init() -> *mut PageDirectory {
    arch::gdt::init();
    init_paging();
    arch::tss::init();
    &mut PAGE_DIRECTORY as *mut _
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct PageDirectory([PageDirectoryEntry; 1024]);
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct PageDirectoryEntry(u32);

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct PageTable([PageTableEntry; 1024]);
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct PageTableEntry(u32);

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Page([u8; 4096]);

impl PageMap<PageDirectoryEntry> for PageDirectory {
    const SIZE: usize = 1024;
}

impl PageMap<PageTableEntry> for PageTable {
    const SIZE: usize = 1024;
}

impl PageMap<u8> for Page {
    const SIZE: usize = 4096;
}

impl PageMapEntry for PageDirectoryEntry {
    type Header = u32;
    type Sector = PageTableEntry;
    type Points = PageTable;

    const AVL_SIZE: u8 = 5;
    const LOWEST_SECTOR: bool = false;

    #[inline]
    fn init(&mut self) {
        self.0 &= !PDE_FLAG_PS;
    }

    #[inline]
    fn get_pointer(&self) -> *mut PageTable {
        (self.0 & PDE_MASK_ADDR) as *mut PageTable
    }

    #[inline]
    fn set_pointer(&mut self, addr: *mut PageTable) {
        self.0 &= !PDE_MASK_ADDR;
        self.0 |= (addr as u32) & PDE_MASK_ADDR
    }

    #[inline]
    fn is_present(&self) -> bool {
        self.0 & PDE_FLAG_P != 0
    }

    #[inline]
    fn set_present(&mut self, p: bool) {
        if p {
            self.0 |= PDE_FLAG_P;
        } else {
            self.0 &= !PDE_FLAG_P;
        }
    }

    #[inline]
    fn is_writable(&self) -> bool {
        self.0 & PDE_FLAG_RW != 0
    }

    #[inline]
    fn set_writable(&mut self, rw: bool) {
        if rw {
            self.0 |= PDE_FLAG_RW;
        } else {
            self.0 &= !PDE_FLAG_RW;
        }
    }

    #[inline]
    fn is_supervisor_only(&self) -> bool {
        self.0 & PDE_FLAG_US == 0
    }

    #[inline]
    fn set_supervisor_only(&mut self, us: bool) {
        if us {
            self.0 |= PDE_FLAG_US;
        } else {
            self.0 &= !PDE_FLAG_US;
        }
    }

    #[inline]
    fn is_write_through_enabled(&self) -> bool {
        self.0 & PDE_FLAG_PWT != 0
    }

    #[inline]
    fn set_write_through_enabled(&mut self, pwt: bool) {
        if pwt {
            self.0 |= PDE_FLAG_PWT;
        } else {
            self.0 &= !PDE_FLAG_PWT;
        }
    }

    #[inline]
    fn is_cache_disabled(&self) -> bool {
        self.0 & PDE_FLAG_PCD != 0
    }

    #[inline]
    fn set_cache_disabled(&mut self, pcd: bool) {
        if pcd {
            self.0 |= PDE_FLAG_PCD;
        } else {
            self.0 &= !PDE_FLAG_PCD;
        }
    }

    #[inline]
    fn was_accessed(&self) -> bool {
        self.0 & PDE_FLAG_A != 0
    }

    #[inline]
    fn clear_accessed(&mut self) {
        self.0 &= !PDE_FLAG_A
    }

    #[inline]
    fn get_avl(&self) -> u16 {
        (((self.0 & 64) >> 6) | ((self.0 & 0b1111_0000_0000) >> 7)) as u16
    }

    #[inline]
    fn set_avl(&mut self, avl: u16) {
        self.0 = (self.0 & !PDE_MASK_AVL) | (((avl as u32) & 1) << 6) | (((avl as u32) & 0b11110) << 8)
    }
}

impl PageMapEntry for PageTableEntry {
    type Header = u32;
    type Sector = u8;
    type Points = Page;

    const AVL_SIZE: u8 = 3;
    const LOWEST_SECTOR: bool = true;

    #[inline]
    fn init(&mut self) {
        self.0 &= !PTE_FLAG_D;
    }

    #[inline]
    fn get_pointer(&self) -> *mut Page {
        (self.0 & PTE_MASK_ADDR) as *mut Page
    }

    #[inline]
    fn set_pointer(&mut self, addr: *mut Page) {
        self.0 &= !PTE_MASK_ADDR;
        self.0 |= (addr as u32) & PTE_MASK_ADDR;
    }

    #[inline]
    fn is_present(&self) -> bool {
        self.0 & PTE_FLAG_P != 0
    }

    #[inline]
    fn set_present(&mut self, p: bool) {
        if p {
            self.0 |= PTE_FLAG_P;
        } else {
            self.0 &= !PTE_FLAG_P;
        }
    }

    #[inline]
    fn is_writable(&self) -> bool {
        self.0 & PTE_FLAG_RW != 0
    }

    #[inline]
    fn set_writable(&mut self, rw: bool) {
        if rw {
            self.0 |= PTE_FLAG_RW;
        } else {
            self.0 &= !PTE_FLAG_RW;
        }
    }

    #[inline]
    fn is_supervisor_only(&self) -> bool {
        self.0 & PTE_FLAG_US == 0
    }

    #[inline]
    fn set_supervisor_only(&mut self, us: bool) {
        if us {
            self.0 |= PTE_FLAG_US;
        } else {
            self.0 &= !PTE_FLAG_US;
        }
    }

    #[inline]
    fn is_write_through_enabled(&self) -> bool {
        self.0 & PTE_FLAG_PWT != 0
    }

    #[inline]
    fn set_write_through_enabled(&mut self, pwt: bool) {
        if pwt {
            self.0 |= PTE_FLAG_PWT;
        } else {
            self.0 &= !PTE_FLAG_PWT;
        }
    }

    #[inline]
    fn is_cache_disabled(&self) -> bool {
        self.0 & PTE_FLAG_PCD != 0
    }

    #[inline]
    fn set_cache_disabled(&mut self, pcd: bool) {
        if pcd {
            self.0 |= PTE_FLAG_PCD;
        } else {
            self.0 &= !PTE_FLAG_PCD;
        }
    }

    #[inline]
    fn was_accessed(&self) -> bool {
        self.0 & PTE_FLAG_A != 0
    }

    #[inline]
    fn clear_accessed(&mut self) {
        self.0 &= !PTE_FLAG_A;
    }

    #[inline]
    fn get_avl(&self) -> u16 {
        (self.0 & PTE_MASK_AVL >> 9) as u16
    }

    #[inline]
    fn set_avl(&mut self, avl: u16) {
        self.0 &= !PTE_MASK_AVL;
        self.0 |= (avl as u32) << 9;
    }
}
