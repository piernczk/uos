use crate::arch::generic::paging::*;

pub const AI686_PDE_MASK_ADDR: u32 = !0b1111_1111_1111;
pub const AI686_PDE_MASK_AVL: u32 = 0b1111_0100_0000;

pub const AI686_PDE_FLAG_P: u32 = 1 << 0;
pub const AI686_PDE_FLAG_RW: u32 = 1 << 1;
pub const AI686_PDE_FLAG_US: u32 = 1 << 2;
pub const AI686_PDE_FLAG_PWT: u32 = 1 << 3;
pub const AI686_PDE_FLAG_PCD: u32 = 1 << 4;
pub const AI686_PDE_FLAG_A: u32 = 1 << 5;
pub const AI686_PDE_FLAG_PS: u32 = 1 << 7;

pub const AI686_PTE_MASK_ADDR: u32 = !0b1111_1111_1111;
pub const AI686_PTE_MASK_AVL: u32 = 0b1110_0000_0000;

pub const AI686_PTE_FLAG_P: u32 = 1 << 0;
pub const AI686_PTE_FLAG_RW: u32 = 1 << 1;
pub const AI686_PTE_FLAG_US: u32 = 1 << 2;
pub const AI686_PTE_FLAG_PWT: u32 = 1 << 3;
pub const AI686_PTE_FLAG_PCD: u32 = 1 << 4;
pub const AI686_PTE_FLAG_A: u32 = 1 << 5;
pub const AI686_PTE_FLAG_D: u32 = 1 << 6;

extern "C" {
    static mut page_directory: Ai686PageDirectory;
    static mut page_tables: [Ai686PageTable; 1024];
}

pub type HighestTable = Ai686PageDirectory;
pub type HighestEntry = Ai686PageDirectoryEntry;

pub unsafe fn init() -> *mut Ai686PageDirectory {
    let mut i = 0;
    for pde in &mut page_directory.0 {
        pde.init();
        pde.set_pointer(&mut page_tables[i] as *mut _);

        let mut j = 0;
        for pte in &mut page_tables[i].0 {
            pte.init();
            pte.set_pointer((j * (10 << 12)) as *mut Ai686Page);
            j += 1;
        }

        i += 1;
    }

    &mut page_directory as *mut _
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Ai686PageDirectory([Ai686PageDirectoryEntry; 1024]);
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Ai686PageDirectoryEntry(u32);

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Ai686PageTable([Ai686PageTableEntry; 1024]);
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Ai686PageTableEntry(u32);

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Ai686Page([u8; 4096]);

impl PageMap<Ai686PageDirectoryEntry> for Ai686PageDirectory {
    const SIZE: usize = 1024;
}

impl PageMap<Ai686PageTableEntry> for Ai686PageTable {
    const SIZE: usize = 1024;
}

impl PageMap<u8> for Ai686Page {
    const SIZE: usize = 4096;
}

impl PageMapEntry for Ai686PageDirectoryEntry {
    type Header = u32;
    type Lower = Ai686PageTableEntry;
    type Points = Ai686PageTable;

    const AVL_SIZE: u8 = 5;
    const LOWEST: bool = false;

    #[inline]
    fn init(&mut self) {
        self.0 &= !AI686_PDE_FLAG_PS;
    }

    #[inline]
    fn get_pointer(&self) -> *mut Ai686PageTable {
        (self.0 & AI686_PDE_MASK_ADDR) as *mut Ai686PageTable
    }

    #[inline]
    fn set_pointer(&mut self, addr: *mut Ai686PageTable) {
        self.0 &= !AI686_PDE_MASK_ADDR;
        self.0 |= (addr as u32) & AI686_PDE_MASK_ADDR
    }

    #[inline]
    fn is_present(&self) -> bool {
        self.0 & AI686_PDE_FLAG_P != 0
    }

    #[inline]
    fn set_present(&mut self, p: bool) {
        if p {
            self.0 |= AI686_PDE_FLAG_P;
        } else {
            self.0 &= !AI686_PDE_FLAG_P;
        }
    }

    #[inline]
    fn is_writable(&self) -> bool {
        self.0 & AI686_PDE_FLAG_RW != 0
    }

    #[inline]
    fn set_writable(&mut self, rw: bool) {
        if rw {
            self.0 |= AI686_PDE_FLAG_RW;
        } else {
            self.0 &= !AI686_PDE_FLAG_RW;
        }
    }

    #[inline]
    fn is_supervisor_only(&self) -> bool {
        self.0 & AI686_PDE_FLAG_US == 0
    }

    #[inline]
    fn set_supervisor_only(&mut self, us: bool) {
        if us {
            self.0 |= AI686_PDE_FLAG_US;
        } else {
            self.0 &= !AI686_PDE_FLAG_US;
        }
    }

    #[inline]
    fn is_write_through_enabled(&self) -> bool {
        self.0 & AI686_PDE_FLAG_PWT != 0
    }

    #[inline]
    fn set_write_through_enabled(&mut self, pwt: bool) {
        if pwt {
            self.0 |= AI686_PDE_FLAG_PWT;
        } else {
            self.0 &= !AI686_PDE_FLAG_PWT;
        }
    }

    #[inline]
    fn is_cache_disabled(&self) -> bool {
        self.0 & AI686_PDE_FLAG_PCD != 0
    }

    #[inline]
    fn set_cache_disabled(&mut self, pcd: bool) {
        if pcd {
            self.0 |= AI686_PDE_FLAG_PCD;
        } else {
            self.0 &= !AI686_PDE_FLAG_PCD;
        }
    }

    #[inline]
    fn was_accessed(&self) -> bool {
        self.0 & AI686_PDE_FLAG_A != 0
    }

    #[inline]
    fn clear_accessed(&mut self) {
        self.0 &= !AI686_PDE_FLAG_A
    }

    #[inline]
    fn get_avl(&self) -> u16 {
        (((self.0 & 64) >> 6) | ((self.0 & 0b1111_0000_0000) >> 7)) as u16
    }

    #[inline]
    fn set_avl(&mut self, avl: u16) {
        self.0 = (self.0 & !AI686_PDE_MASK_AVL) | (((avl as u32) & 1) << 6) | (((avl as u32) & 0b11110) << 8)
    }
}

impl PageMapEntry for Ai686PageTableEntry {
    type Header = u32;
    type Lower = u8;
    type Points = Ai686Page;

    const AVL_SIZE: u8 = 3;
    const LOWEST: bool = true;

    #[inline]
    fn init(&mut self) {
        self.0 &= !AI686_PTE_FLAG_D;
    }

    #[inline]
    fn get_pointer(&self) -> *mut Ai686Page {
        (self.0 & AI686_PTE_MASK_ADDR) as *mut Ai686Page
    }

    #[inline]
    fn set_pointer(&mut self, addr: *mut Ai686Page) {
        self.0 &= !AI686_PTE_MASK_ADDR;
        self.0 |= (addr as u32) & AI686_PTE_MASK_ADDR;
    }

    #[inline]
    fn is_present(&self) -> bool {
        self.0 & AI686_PTE_FLAG_P != 0
    }

    #[inline]
    fn set_present(&mut self, p: bool) {
        if p {
            self.0 |= AI686_PTE_FLAG_P;
        } else {
            self.0 &= !AI686_PTE_FLAG_P;
        }
    }

    #[inline]
    fn is_writable(&self) -> bool {
        self.0 & AI686_PTE_FLAG_RW != 0
    }

    #[inline]
    fn set_writable(&mut self, rw: bool) {
        if rw {
            self.0 |= AI686_PTE_FLAG_RW;
        } else {
            self.0 &= !AI686_PTE_FLAG_RW;
        }
    }

    #[inline]
    fn is_supervisor_only(&self) -> bool {
        self.0 & AI686_PTE_FLAG_US == 0
    }

    #[inline]
    fn set_supervisor_only(&mut self, us: bool) {
        if us {
            self.0 |= AI686_PTE_FLAG_US;
        } else {
            self.0 &= !AI686_PTE_FLAG_US;
        }
    }

    #[inline]
    fn is_write_through_enabled(&self) -> bool {
        self.0 & AI686_PTE_FLAG_PWT != 0
    }

    #[inline]
    fn set_write_through_enabled(&mut self, pwt: bool) {
        if pwt {
            self.0 |= AI686_PTE_FLAG_PWT;
        } else {
            self.0 &= !AI686_PTE_FLAG_PWT;
        }
    }

    #[inline]
    fn is_cache_disabled(&self) -> bool {
        self.0 & AI686_PTE_FLAG_PCD != 0
    }

    #[inline]
    fn set_cache_disabled(&mut self, pcd: bool) {
        if pcd {
            self.0 |= AI686_PTE_FLAG_PCD;
        } else {
            self.0 &= !AI686_PTE_FLAG_PCD;
        }
    }

    #[inline]
    fn was_accessed(&self) -> bool {
        self.0 & AI686_PTE_FLAG_A != 0
    }

    #[inline]
    fn clear_accessed(&mut self) {
        self.0 &= !AI686_PTE_FLAG_A;
    }

    #[inline]
    fn get_avl(&self) -> u16 {
        (self.0 & AI686_PTE_MASK_AVL >> 9) as u16
    }

    #[inline]
    fn set_avl(&mut self, avl: u16) {
        self.0 &= !AI686_PTE_MASK_AVL;
        self.0 |= (avl as u32) << 9;
    }
}
