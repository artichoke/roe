mod lowercase;
mod titlecase;
mod uppercase;

pub use lowercase::make_ascii_lowercase;
pub use titlecase::make_ascii_titlecase;
pub use uppercase::make_ascii_uppercase;

#[cfg(feature = "alloc")]
pub use lowercase::to_ascii_lowercase;
#[cfg(feature = "alloc")]
pub use titlecase::to_ascii_titlecase;
#[cfg(feature = "alloc")]
pub use uppercase::to_ascii_uppercase;
