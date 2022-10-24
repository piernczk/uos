//! Architecture-independent paging abstractions.
//!
//! Paging allows to divide the operating memory into so-called pages. They are
//! grouped into maps (tables), which themselves may be grouped into maps.
//!
//! Pages provide security in form of having exactly one owner (being a
//! process), which is the one having access to it. Some pages are reserved for
//! the kernel and so no process can access or allocate them.
//!
//! Despite complying with the traits declared here, every `paging` arch
//! submodule must provide:
//! - types:
//!   - ```HighestMap: PageMap<HighestEntry>```
//!   - ```HighestEntry: PageMapEntry```
//! - functions:
//!   - ```init() -> *mut HighestMap```
//! - methods:
//!   - ```PhysicalAddress::virtual_addr() -> *const VirtualSymbol```

/// A symbol with a virtual address.
///
/// Used when there was a virtual address without a type defined as a symbol,
/// like `KERNEL_END` in `i686/linker.ld`. Could just use `()` but it's more
/// convenient to distinct it from [`PhysicalSymbol`].
pub struct VirtualSymbol;

impl VirtualSymbol {
    /// Returns the address of this symbol.
    ///
    /// The same as [&symbol as *const _`].
    pub fn virtual_addr(&self) -> *const Self {
        self as *const Self
    }
}

/// A symbol with a physical address.
///
/// Used when there was a physical address defined as a symbol, like
/// `KERNEL_END_PHYS` in `i686/linker.ld`. Also used to safely convert its
/// physical address to a virtual one (assuming it's a static kernel symbol)
/// with [`PhysicalAddress::virtual_addr`], regardless of the architecture.
pub struct PhysicalSymbol;

impl PhysicalSymbol {
    /// Returns the physical address of this symbol.
    pub fn physical_addr(&self) -> PhysicalSymbolAddress {
        PhysicalAddress(self as *const Self)
    }

    /// Returns the virtual address of this symbol.
    ///
    /// See [`PhysicalAddress::virtual_addr`].
    pub fn virtual_addr(&self) -> *const VirtualSymbol {
        self.physical_addr().virtual_addr()
    }
}

/// The physical address of a symbol.
///
/// Used to differentiate physical addresses from virtual ones (simple pointers
/// like `*const _` and `*const _`).
#[repr(transparent)]
pub struct PhysicalAddress<T>(pub *const T);
pub type PhysicalSymbolAddress = PhysicalAddress<PhysicalSymbol>;

impl<T> PhysicalAddress<T> {
    pub fn symbol_address(self) -> PhysicalSymbolAddress {
        PhysicalAddress(self.0 as *const PhysicalSymbol)
    }
}

impl PhysicalSymbolAddress {
    pub fn typed_address<T>(self) -> PhysicalAddress<T> {
        PhysicalAddress(self.0 as *const T)
    }
}

/// A map of [`PageMapEntry`]s or a single page.
///
/// If it's a page, set `Entry` to `u8`.
pub trait PageMap<Entry> {
    /// How many entries the map has.
    ///
    /// If it's a page, it's its size in bytes.
    const SIZE: usize;
}

/// A structure that points to a page or a page map.
pub trait PageMapEntry: Sized {
    /// The low-level representation of this type.
    ///
    /// For example, if the entry fits in 4 bytes, its `Header` is `u32`.
    type Header;
    /// The type of entries of the map this entry points to.
    ///
    /// If this entry points to a page, it's `u8`. If it points to an actual
    /// page map, set it to the type of an entry of said map.
    type Sector;
    /// The type of the page / page map this entry points to.
    type Points: PageMap<Self::Sector>;

    /// How many bits of information about the entry you can store in it.
    ///
    /// Some architectures leave extra bits for the use of the kernel.
    const AVL_SIZE: u8;
    /// Whether or not this entry points to a page.
    const LOWEST_SECTOR: bool;

    /// Inits the entry.
    fn init(&mut self);
    /// Constructs an entry from its [`Self::Header`] and [`init`]s it.
    unsafe fn from_header(header: &mut Self::Header) -> &mut Self {
        let pme = &mut *(header as *mut Self::Header as *mut Self);
        pme.init();
        pme
    }

    /// Returns the page / page map it points to.
    fn get_pointer(&self) -> PhysicalAddress<Self::Points>;
    /// Makes the entry point to the given page / page map.
    fn set_pointer(&mut self, addr: PhysicalAddress<Self::Points>);

    /// Returns whether or not this page / page map is used at the moment.
    ///
    /// If this entry points to a page map whose at least one entry is used, the
    /// whole page map and this entry are used.
    fn is_present(&self) -> bool;
    /// Make the entry present.
    ///
    /// See [`Self::is_present`].
    fn set_present(&mut self, p: bool);

    /// Returns whether this page / page map is read / write or read-only.
    fn is_writable(&self) -> bool;
    /// Makes the page / page map either read / write or read-only.
    fn set_writable(&mut self, rw: bool);

    /// Returns whether or not this page / page map is allocated by the kernel.
    fn is_supervisor_only(&self) -> bool;
    /// Makes the page / page map allocated by the kernel or by a user process.
    fn set_supervisor_only(&mut self, us: bool);

    /// Caching stuff.
    fn is_write_through_enabled(&self) -> bool;
    /// Caching stuff.
    fn set_write_through_enabled(&mut self, pwt: bool);
    /// Returns `true` if the entry is not cached.
    fn is_cache_disabled(&self) -> bool;
    /// Disables / enables caching for this entry.
    fn set_cache_disabled(&mut self, pcd: bool);

    /// Returns `true` if the entry was read by the CPU.
    fn was_accessed(&self) -> bool;
    /// Clears the `accessed` flag.
    fn clear_accessed(&mut self);

    /// Returns the stored information about the entry.
    fn get_avl(&self) -> u16;
    /// Sets some info about the entry.
    ///
    /// Its size in bits mustn't be larger than [`Self::AVL_SIZE`].
    fn set_avl(&mut self, avl: u16);
}
