#![no_std]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod lowercase;
mod uppercase;

pub use lowercase::Lowercase;
pub use uppercase::Uppercase;

// Returns an iterator that yields a copy of the bytes in the given slice with
// all uppercase letters replaced with their lowercase counterparts.
//
// This function treats the given slice as a conventionally UTF-8 string. UTF-8
// byte sequences are converted to their Unicode lowercase equivalents. Invalid
// UTF-8 byte sequences are yielded as is.
pub fn lowercase(slice: &[u8]) -> Lowercase<'_> {
    Lowercase::from(slice)
}

// Returns an iterator that yields a copy of the bytes in the given slice with
// all lowercase letters replaced with their uppercase counterparts.
//
// This function treats the given slice as a conventionally UTF-8 string. UTF-8
// byte sequences are converted to their Unicode uppercase equivalents. Invalid
// UTF-8 byte sequences are yielded as is.
pub fn uppercase(slice: &[u8]) -> Uppercase<'_> {
    Uppercase::from(slice)
}
