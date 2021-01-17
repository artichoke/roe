use core::char::{ToLowercase, ToUppercase};
use core::fmt;
use core::iter::FusedIterator;
use core::ops::Range;

use bstr::ByteSlice;

#[derive(Debug, Clone)]
enum UpperLower {
    Upper(ToUppercase),
    Lower(ToLowercase),
}

impl Iterator for UpperLower {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self {
            Self::Upper(upper) => upper.next(),
            Self::Lower(lower) => lower.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Upper(upper) => upper.size_hint(),
            Self::Lower(lower) => lower.size_hint(),
        }
    }
}

impl FusedIterator for UpperLower {}

impl ExactSizeIterator for UpperLower {}

/// An iterator that yields the capitalize equivalent of a conventionally UTF-8
/// byte string.
///
/// This iterator yields [bytes].
///
/// This struct is created by the [`capitalize`] function. See its documentation
/// for more.
///
/// [bytes]: u8
/// [`capitalize`]: crate::capitalize()
#[derive(Clone)]
#[must_use = "Captialize is a Iterator and must be consumed"]
pub struct Captialize<'a> {
    slice: &'a [u8],
    next_bytes: [u8; 4],
    next_range: Range<usize>,
    upper_lower: Option<UpperLower>,
}

impl<'a> fmt::Debug for Captialize<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Captialize")
            .field("slice", &self.slice.as_bstr())
            .field("next_bytes", &self.next_bytes)
            .field("next_range", &self.next_range)
            .field("upper_lower", &self.upper_lower)
            .finish()
    }
}

impl<'a> Default for Captialize<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a [u8]> for Captialize<'a> {
    fn from(slice: &'a [u8]) -> Self {
        Self {
            slice,
            next_bytes: [0; 4],
            next_range: 0..0,
            upper_lower: None,
        }
    }
}

impl<'a> Captialize<'a> {
    /// Create a new, empty capitalize iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::Captialize;
    /// let mut capitalize = Captialize::new();
    /// assert_eq!(capitalize.next(), None);
    /// ```
    pub const fn new() -> Self {
        Self {
            slice: &[],
            next_bytes: [0; 4],
            next_range: 0..0,
            upper_lower: None,
        }
    }
}

impl<'a> Iterator for Captialize<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.next_range.next() {
            return Some(self.next_bytes[idx]);
        }

        if let Some(ch) = self.upper_lower.as_mut().and_then(Iterator::next) {
            let mut capitalize = ch.to_capitalize();
            let ch = capitalize
                .next()
                .expect("ToCaptialize yields at least one char");
            let enc = ch.encode_utf8(&mut self.next_bytes);
            self.next_range = 1..enc.len();
            self.capitalize = Some(capitalize);
            return Some(self.next_bytes[0]);
        }

        self.capitalize = None;

        match bstr::decode_utf8(self.slice) {
            (_, 0) => None,
            (Some(ch), size) => {
                self.slice = &self.slice[size..];
                let mut capitalize = ch.to_capitalize();
                let ch = capitalize
                    .next()
                    .expect("ToCaptialize yields at least one char");
                let enc = ch.encode_utf8(&mut self.next_bytes);
                self.next_range = 1..enc.len();
                self.capitalize = Some(capitalize);
                Some(self.next_bytes[0])
            }
            (None, size) => {
                let (bytes, remainder) = self.slice.split_at(size);
                self.slice = remainder;
                // Invalid byte sequences are at most three bytes.
                self.next_bytes[..bytes.len()].copy_from_slice(bytes);
                self.next_range = 1..bytes.len();
                Some(self.next_bytes[0])
            }
        }
    }
}

impl<'a> FusedIterator for Captialize<'a> {}
