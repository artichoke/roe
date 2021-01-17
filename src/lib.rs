#![no_std]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod lowercase;

pub use lowercase::Lowercase;

// Returns an iterator that yields a copy of the bytes in the given slice with
// all uppercase letters replaced with their lowercase counterparts.
//
// This function treats the given slice as a conventionally UTF-8 string. UTF-8
// byte sequences are converted to their Unicode lowercase equivalents. Invalid
// UTF-8 byte sequences are yielded as is.
pub fn lowercase(slice: &[u8]) -> Lowercase<'_> {
    Lowercase::from(slice)
}
