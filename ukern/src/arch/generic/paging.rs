pub trait PageMap<Entry> {
    const SIZE: usize;
}

pub trait PageMapEntry: Sized {
    type Header;
    type Lower;
    type Points: PageMap<Self::Lower>;

    const AVL_SIZE: u8;
    const LOWEST: bool;

    fn init(&mut self);
    unsafe fn from_header(header: &mut Self::Header) -> &mut Self {
        let pme = &mut *(header as *mut Self::Header as *mut Self);
        pme.init();
        pme
    }

    fn get_pointer(&self) -> *mut Self::Points;
    fn set_pointer(&mut self, addr: *mut Self::Points);

    fn is_present(&self) -> bool;
    fn set_present(&mut self, p: bool);

    fn is_writable(&self) -> bool;
    fn set_writable(&mut self, rw: bool);

    fn is_supervisor_only(&self) -> bool;
    fn set_supervisor_only(&mut self, us: bool);

    fn is_write_through_enabled(&self) -> bool;
    fn set_write_through_enabled(&mut self, pwt: bool);
    fn is_cache_disabled(&self) -> bool;
    fn set_cache_disabled(&mut self, pcd: bool);

    fn was_accessed(&self) -> bool;
    fn clear_accessed(&mut self);

    fn get_avl(&self) -> u16;
    fn set_avl(&mut self, avl: u16);
}
