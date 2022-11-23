use core::marker::PhantomData;
use core::mem::size_of;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ChunkSize(pub usize);

pub struct FreeChunkPtr<Chunk: ?Sized, const OFFSET: usize>(*const Self, PhantomData<Chunk>);

impl<Chunk, const OFFSET: usize> FreeChunkPtr<Chunk, OFFSET> {
    #[inline]
    pub fn get_chunk(self) -> *const Chunk {
        unsafe {
            self.0.byte_sub(OFFSET) as *const Chunk
        }
    }
}

pub type SmallFreeChunkFwd = FreeChunkPtr<SmallFreeChunk, { size_of::<ChunkSize>() }>;
pub type SmallFreeChunkBck = FreeChunkPtr<SmallFreeChunk, { size_of::<ChunkSize>() + size_of::<SmallFreeChunkFwd>() }>;
pub type LargeFreeChunkFwd = FreeChunkPtr<LargeFreeChunk, { size_of::<ChunkSize>() }>;
pub type LargeFreeChunkBck = FreeChunkPtr<LargeFreeChunk, { size_of::<ChunkSize>() + size_of::<LargeFreeChunkFwd>() }>;

#[repr(C)]
pub struct SmallFreeChunkHeader {
    size: ChunkSize,
    pub fwd: SmallFreeChunkFwd,
    pub bck: SmallFreeChunkBck,
}

#[repr(C)]
pub struct SmallFreeChunk {
    pub header: SmallFreeChunkHeader,
    pub payload: [u8],
}

impl SmallFreeChunk {
    pub fn set_size(&mut self, size: ChunkSize) {
        self.header.size = size;
        unsafe {
            // set the last word, prev_size
            *(self as *mut _ as *mut usize).byte_add(size.get_size()).sub(1) = size.get_size();
        }
    }

    #[inline]
    pub fn get_size(&self) -> ChunkSize {
        self.header.size
    }
}

#[repr(C)]
pub struct LargeFreeChunkHeader {
    pub size: ChunkSize,
    pub fwd: LargeFreeChunkFwd,
    pub bck: LargeFreeChunkBck,
    pub fd_nextsize: *const ChunkSize,
    pub bk_nextsize: *const ChunkSize,
}

#[repr(C)]
pub struct LargeFreeChunk {
    pub header: LargeFreeChunkHeader,
    pub payload: [u8],
}

impl LargeFreeChunk {
    pub fn set_size(&mut self, size: ChunkSize) {
        self.header.size = size;
        unsafe {
            // set the last word, prev_size
            *(self as *mut _ as *mut usize).byte_add(size.get_size()).sub(1) = size.get_size();
        }
    }

    #[inline]
    pub fn get_size(&self) -> ChunkSize {
        self.header.size
    }
}

impl ChunkSize {
    #[inline]
    pub const fn new(size: usize, arena_allocated: bool, chunk_obtained: bool, previous_used: bool) -> Self {
        Self(
            (size & !7) |
            ((arena_allocated as usize) << 2) |
            ((chunk_obtained as usize) << 1) |
            (previous_used as usize)
        )
    }

    #[inline]
    pub fn get_size(&self) -> usize {
        self.0 & !7
    }

    #[inline]
    pub fn set_size(&mut self, size: usize) {
        self.0 = (size & !7) | (self.0 & 7);
    }

    #[inline]
    pub fn is_arena_allocated(&self) -> bool {
        self.0 & 4 != 0
    }

    #[inline]
    pub fn set_arena_allocated(&mut self, arena_allocated: bool) {
        if arena_allocated {
            self.0 |= 4;
        } else {
            self.0 &= !4;
        }
    }

    #[inline]
    pub fn is_chunk_obtained(&self) -> bool {
        self.0 & 2 != 0
    }

    #[inline]
    pub fn set_chunk_obtained(&mut self, chunk_obtained: bool) {
        if chunk_obtained {
            self.0 |= 2;
        } else {
            self.0 &= !2;
        }
    }

    #[inline]
    pub fn is_previous_used(&self) -> bool {
        self.0 & 1 != 0
    }

    #[inline]
    pub fn set_previous_used(&mut self, previous_used: bool) {
        if previous_used {
            self.0 |= 1;
        } else {
            self.0 &= !1;
        }
    }
}
