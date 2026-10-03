mod lowercase;
mod swapcase;
mod uppercase;

pub use lowercase::make_ascii_lowercase;
pub use uppercase::make_ascii_uppercase;

#[cfg(feature = "alloc")]
pub use lowercase::to_ascii_lowercase;
#[cfg(feature = "alloc")]
pub use uppercase::to_ascii_uppercase;

pub use swapcase::make_ascii_swapcase;
pub(crate) use swapcase::swapcase_byte;
#[cfg(feature = "alloc")]
pub use swapcase::to_ascii_swapcase;
