use core::iter::FusedIterator;

use crate::ascii::swapcase_byte;
use crate::unicode::mapping::Mode;

mod full;

#[derive(Debug, Clone)]
enum Inner<'a> {
    Full(full::Swapcase<'a>),
    Ascii(&'a [u8]),
}

/// An iterator yielding bytes with the case of each character swapped.
///
/// Created by [`swapcase`](crate::swapcase()). Invalid UTF-8 bytes are preserved.
#[derive(Debug, Clone)]
#[must_use = "Swapcase is an iterator and must be used"]
pub struct Swapcase<'a> {
    iter: Inner<'a>,
}

impl Default for Swapcase<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Swapcase<'a> {
    /// Create an empty swapcase iterator.
    pub const fn new() -> Self {
        Self::with_slice(b"")
    }

    /// Create an iterator using full Unicode case swapping.
    pub const fn with_slice(slice: &'a [u8]) -> Self {
        Self {
            iter: Inner::Full(full::Swapcase::with_slice(slice)),
        }
    }

    /// Create an iterator that swaps ASCII letters and preserves all other bytes.
    pub const fn with_ascii_slice(slice: &'a [u8]) -> Self {
        Self {
            iter: Inner::Ascii(slice),
        }
    }

    pub(crate) const fn with_mode(slice: &'a [u8], mode: Mode) -> Self {
        Self {
            iter: Inner::Full(full::Swapcase::with_mode(slice, mode)),
        }
    }
}

impl Iterator for Swapcase<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iter {
            Inner::Full(iter) => iter.next(),
            Inner::Ascii(slice) => {
                let (&byte, remainder) = slice.split_first()?;
                *slice = remainder;
                Some(swapcase_byte(byte))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.iter {
            Inner::Full(iter) => iter.size_hint(),
            Inner::Ascii(slice) => (slice.len(), Some(slice.len())),
        }
    }

    fn count(self) -> usize {
        match self.iter {
            Inner::Full(iter) => iter.count(),
            Inner::Ascii(slice) => slice.len(),
        }
    }
}

impl FusedIterator for Swapcase<'_> {}
