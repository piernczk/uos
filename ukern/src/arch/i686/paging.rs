//! i686 paging.
//!
//! The highest [`PageMap`] on i686 is the [`PageDirectory`]. It has 1024
//! [`PageDirectoryEntry`]es, each pointing to a [`PageTable`]. They also have
//! 1024 entries ([`PageTagleEntry`]), but they point to 4 KiB pages. This means
//! that i686 supports up to `1024 * 1024 * 4 KiB = 4 GiB` virtual memory
//! (assuming the PAE is disabled, and it is for now).

use crate::arch::{self, generic::paging::*};
use core::arch::asm;

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

extern "C" {
    #[allow(improper_ctypes)]
    static mut PAGE_DIRECTORY_PHYS: PhysicalSymbol;
    #[allow(improper_ctypes)]
    static mut PAGE_TABLES_PHYS: PhysicalSymbol;
    static mut PAGE_DIRECTORY: PageDirectory;
    static mut PAGE_TABLES: [PageTable; 1023];

    #[allow(improper_ctypes)]
    pub static mut KERNEL_START_PHYS: PhysicalSymbol;
    #[allow(improper_ctypes)]
    pub static mut KERNEL_END_PHYS: PhysicalSymbol;
    #[allow(improper_ctypes)]
    pub static mut KERNEL_INIT_ALLOC: PhysicalSymbol;
    #[allow(improper_ctypes)]
    pub static mut KERNEL_START: VirtualSymbol;
    #[allow(improper_ctypes)]
    pub static mut KERNEL_END: VirtualSymbol;
}

pub type HighestMap = PageDirectory;
pub type HighestEntry = PageDirectoryEntry;

impl<T> PhysicalAddress<T> {
    /// Returns the virtual address of this symbol.
    ///
    /// It assumes it's a preallocated, static kernel symbol.
    pub fn virtual_addr(&self) -> *const VirtualSymbol {
        (self.0 as usize + 0xC0000000) as *const VirtualSymbol
    }
}

unsafe fn init_paging() {
    // including the first MiB
    let mut pages = (KERNEL_INIT_ALLOC.physical_addr().0 as usize + (1 << 12) - 1) >> 12;
    let mut page_tables = (KERNEL_INIT_ALLOC.physical_addr().0 as usize + (1 << 22) - 1) >> 22;

    // both higher half and identity paging are set up before kernel_main is
    // even called
    // identity paging isn't needed now so it must be disabled
    for pde in PAGE_DIRECTORY.0.iter_mut().take(page_tables) {
        pde.0 = 0;
    }

    for table in PAGE_TABLES.iter_mut().take(page_tables) {
        for pte in table.0.iter_mut() {
            pte.0 = 0;
        }
    }

    for pte in PAGE_TABLES[page_tables - 1]
        .0
        .iter_mut()
        .take(pages - (page_tables - 1) * 1024)
    {
        pte.0 = 0;
    }

    let kernel_end_page = (KERNEL_END.virtual_addr() as usize + (1 << 12) - 1) >> 12;
    let kernel_end_page_tables = (KERNEL_END.virtual_addr() as usize + (1 << 22) - 1) >> 22;
    pages += 768 * 1024;
    page_tables += 768;
    for pde in PAGE_DIRECTORY
        .0
        .iter_mut()
        .skip(kernel_end_page)
        .take(pages - kernel_end_page)
    {
        pde.0 = 0;
    }

    for table in PAGE_TABLES
        .iter_mut()
        .skip(page_tables)
        .take(kernel_end_page_tables - page_tables)
    {
        for pte in table.0.iter_mut() {
            pte.0 = 0;
        }
    }

    for pte in PAGE_TABLES[page_tables - 1]
        .0
        .iter_mut()
        .skip(kernel_end_page - (kernel_end_page_tables - 1) * 1024)
        .take(pages - kernel_end_page + 1024 * ((kernel_end_page_tables - 1) - (page_tables - 1)))
    {
        pte.0 = 0;
    }

    // make the first page inaccessible so that the null pointer is not valid
    PAGE_DIRECTORY.0[0].init();
    PAGE_DIRECTORY.0[0].set_pointer(PAGE_TABLES_PHYS.physical_addr().typed_address());
    PAGE_DIRECTORY.0[0].set_present(false);
    PAGE_DIRECTORY.0[0].set_supervisor_only(true);
    PAGE_DIRECTORY.0[0].set_writable(false);
    PAGE_TABLES[0].0[0].init();
    PAGE_TABLES[0].0[0].set_pointer(PhysicalAddress(core::ptr::null()));
    PAGE_TABLES[0].0[0].set_present(false);
    PAGE_TABLES[0].0[0].set_supervisor_only(true);
    PAGE_TABLES[0].0[0].set_writable(false);

    // setup recursive paging
    PAGE_DIRECTORY.0[1023].init();
    PAGE_DIRECTORY.0[1023].set_pointer(PAGE_DIRECTORY_PHYS.physical_addr().typed_address());
    PAGE_TABLES[0].0[0].set_present(true);
    PAGE_TABLES[0].0[0].set_supervisor_only(true);
    PAGE_TABLES[0].0[0].set_writable(true);

    // TODO mark the first page and the last page directory as always used by
    // the kernel when implementing memory managment

    flush();
}

pub unsafe fn flush() {
    let addr = PAGE_DIRECTORY_PHYS.physical_addr().0 as usize;
    asm!("mov cr3, eax", in("eax") addr);
}

/// Initializes paging, the [GDT](arch::gdt) and the [TSS](arch::tss).
pub unsafe fn init() {
    arch::gdt::init();
    arch::tss::init();
    init_paging();
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
    fn get_pointer(&self) -> PhysicalAddress<PageTable> {
        PhysicalAddress((self.0 & PDE_MASK_ADDR) as *const _)
    }

    #[inline]
    fn set_pointer(&mut self, addr: PhysicalAddress<PageTable>) {
        self.0 &= !PDE_MASK_ADDR;
        self.0 |= (addr.0 as u32) & PDE_MASK_ADDR
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
        self.0 =
            (self.0 & !PDE_MASK_AVL) | (((avl as u32) & 1) << 6) | (((avl as u32) & 0b11110) << 8)
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
    fn get_pointer(&self) -> PhysicalAddress<Page> {
        PhysicalAddress((self.0 & PTE_MASK_ADDR) as *const _)
    }

    #[inline]
    fn set_pointer(&mut self, addr: PhysicalAddress<Page>) {
        self.0 &= !PTE_MASK_ADDR;
        self.0 |= (addr.0 as u32) & PTE_MASK_ADDR;
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
