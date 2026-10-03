use core::convert::{TryFrom, TryInto};
use core::iter::FusedIterator;
use core::str::FromStr;

use crate::InvalidCaseMappingMode;
use crate::unicode::mapping::Mode;

mod ascii;
mod full;

use self::ascii::swapcase_byte;

pub use ascii::make_ascii_swapcase;
#[cfg(feature = "alloc")]
pub use ascii::to_ascii_swapcase;

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

/// Options to configure the behavior of [`swapcase`].
///
/// Which letters exactly are replaced, and by which other letters, depends on
/// the given options.
///
/// See individual variants for a description of the available behaviors.
///
/// If you're not sure which mode to choose, [`SwapcaseMode::Full`] is a a good
/// default.
///
/// [`swapcase`]: crate::swapcase()
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SwapcaseMode {
    /// Full Unicode case mapping, suitable for most languages.
    ///
    /// See the [Turkic] and [Lithuanian] variants for exceptions.
    ///
    /// Context-dependent case mapping as described in Table 3-14 of the Unicode
    /// standard is currently not supported.
    ///
    /// [Turkic]: Self::Turkic
    /// [Lithuanian]: Self::Lithuanian
    #[default]
    Full,
    /// Only the ASCII region, i.e. the characters `'A'..='Z'` and `'a'..='z'`,
    /// are affected.
    ///
    /// This option cannot be combined with any other option.
    Ascii,
    /// Full Unicode case mapping, adapted for Turkic languages (Turkish,
    /// Azerbaijani, …).
    ///
    /// This means that upper case I is mapped to lower case dotless i, and so
    /// on.
    Turkic,
    /// Currently, just [full Unicode case mapping].
    ///
    /// This matches MRI Ruby, which does not implement Lithuanian contextual
    /// case mapping.
    ///
    /// [full Unicode case mapping]: Self::Full
    Lithuanian,
}

impl TryFrom<&str> for SwapcaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}

impl TryFrom<Option<&str>> for SwapcaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        value.map(str::as_bytes).try_into()
    }
}

impl TryFrom<&[u8]> for SwapcaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"ascii" => Ok(Self::Ascii),
            b"turkic" => Ok(Self::Turkic),
            b"lithuanian" => Ok(Self::Lithuanian),
            _ => Err(InvalidCaseMappingMode::new()),
        }
    }
}

impl TryFrom<Option<&[u8]>> for SwapcaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&[u8]>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(Self::Full),
            Some(b"ascii") => Ok(Self::Ascii),
            Some(b"turkic") => Ok(Self::Turkic),
            Some(b"lithuanian") => Ok(Self::Lithuanian),
            Some(_) => Err(InvalidCaseMappingMode::new()),
        }
    }
}

impl FromStr for SwapcaseMode {
    type Err = InvalidCaseMappingMode;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

/// Returns an iterator that yields a copy of the bytes in the given slice with
/// the case of each character swapped.
///
/// This function treats the given slice as a [conventionally UTF-8 string].
/// Uppercase characters are lowercased, lowercase characters are uppercased,
/// and titlecase letters have the case of their components swapped.
/// Invalid UTF-8 byte sequences are yielded as is.
///
/// The case mapping mode is determined by the given [`SwapcaseMode`]. See its
/// documentation for details on the available case mapping modes.
///
/// [conventionally UTF-8 string]: https://docs.rs/bstr/1.*/bstr/#when-should-i-use-byte-strings
pub const fn swapcase(slice: &[u8], options: SwapcaseMode) -> Swapcase<'_> {
    match options {
        SwapcaseMode::Full | SwapcaseMode::Lithuanian => Swapcase::with_slice(slice),
        SwapcaseMode::Ascii => Swapcase::with_ascii_slice(slice),
        SwapcaseMode::Turkic => Swapcase::with_mode(slice, Mode::TurkicSwap),
    }
}
