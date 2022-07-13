pub mod generic;
pub use generic::*;

#[cfg(target_arch = "x86")] pub mod i686;
#[cfg(target_arch = "x86")] pub use i686::paging;
