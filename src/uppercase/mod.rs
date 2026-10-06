use core::convert::{TryFrom, TryInto};
use core::iter::FusedIterator;
use core::str::FromStr;

#[cfg(feature = "alloc")]
use alloc::borrow::Cow;
#[cfg(feature = "alloc")]
use alloc::collections::TryReserveError;

use crate::InvalidCaseMappingMode;
use crate::unicode::mapping::Mode;

mod ascii;
mod full;

pub use ascii::make_ascii_uppercase;
#[cfg(feature = "alloc")]
pub use ascii::to_ascii_uppercase;

#[derive(Debug, Clone)]
#[allow(variant_size_differences)]
enum Inner<'a> {
    Empty,
    Full(full::Uppercase<'a>),
    Ascii(ascii::Uppercase<'a>),
}

/// An iterator that yields the uppercase equivalent of a conventionally UTF-8
/// byte string.
///
/// This iterator yields [bytes].
///
/// This struct is created by the [`uppercase`] function. See its documentation
/// for more.
///
/// [bytes]: u8
/// [`uppercase`]: crate::uppercase()
#[derive(Debug, Clone)]
#[must_use = "Uppercase is a Iterator and must be used"]
pub struct Uppercase<'a> {
    iter: Inner<'a>,
}

impl Default for Uppercase<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Uppercase<'a> {
    /// Create a new, empty uppercase iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::Uppercase;
    /// let mut uppercase = Uppercase::new();
    /// assert_eq!(uppercase.next(), None);
    /// ```
    pub const fn new() -> Self {
        Self { iter: Inner::Empty }
    }

    /// Create a new uppercase iterator with the given byte slice using full
    /// Unicode case mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::Uppercase;
    /// let mut uppercase = Uppercase::with_slice(b"abcXYZ");
    /// assert_eq!(uppercase.next(), Some(b'A'));
    /// assert_eq!(uppercase.next(), Some(b'B'));
    /// assert_eq!(uppercase.next(), Some(b'C'));
    /// assert_eq!(uppercase.next(), Some(b'X'));
    /// assert_eq!(uppercase.next(), Some(b'Y'));
    /// assert_eq!(uppercase.next(), Some(b'Z'));
    /// assert_eq!(uppercase.next(), None);
    /// ```
    ///
    /// Non-ASCII characters are case mapped:
    ///
    /// ```
    /// # use roe::Uppercase;
    /// let uppercase = Uppercase::with_slice("Αύριο".as_bytes());
    /// assert_eq!(uppercase.collect::<Vec<_>>(), "ΑΎΡΙΟ".as_bytes());
    /// ```
    ///
    /// Invalid UTF-8 bytes are yielded as is without impacting Unicode
    /// characters:
    ///
    /// ```
    /// # use roe::Uppercase;
    /// let mut s = "Αύριο".to_string().into_bytes();
    /// s.extend(b"\xFF\xFE");
    /// let uppercase = Uppercase::with_slice(s.as_slice());
    ///
    /// let mut expected = "ΑΎΡΙΟ".to_string().into_bytes();
    /// expected.extend(b"\xFF\xFE");
    /// assert_eq!(uppercase.collect::<Vec<_>>(), expected);
    /// ```
    pub const fn with_slice(slice: &'a [u8]) -> Self {
        // Full Unicode and ASCII mappings agree for an entirely ASCII input.
        if slice.is_ascii() {
            return Self::with_ascii_slice(slice);
        }
        Self {
            iter: Inner::Full(full::Uppercase::with_slice(slice)),
        }
    }

    /// Create a new uppercase iterator with the given byte slice using ASCII
    /// case mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::Uppercase;
    /// let mut uppercase = Uppercase::with_ascii_slice(b"abcXYZ");
    /// assert_eq!(uppercase.next(), Some(b'A'));
    /// assert_eq!(uppercase.next(), Some(b'B'));
    /// assert_eq!(uppercase.next(), Some(b'C'));
    /// assert_eq!(uppercase.next(), Some(b'X'));
    /// assert_eq!(uppercase.next(), Some(b'Y'));
    /// assert_eq!(uppercase.next(), Some(b'Z'));
    /// assert_eq!(uppercase.next(), None);
    /// ```
    ///
    /// Non-ASCII characters are ignored:
    ///
    /// ```
    /// # use roe::Uppercase;
    /// let uppercase = Uppercase::with_ascii_slice("Αύριο".as_bytes());
    /// assert_eq!(uppercase.collect::<Vec<_>>(), "Αύριο".as_bytes());
    /// ```
    ///
    /// Invalid UTF-8 bytes are yielded as is without impacting ASCII bytes:
    ///
    /// ```
    /// # use roe::Uppercase;
    /// let uppercase = Uppercase::with_ascii_slice(b"abc\xFF\xFEXYZ");
    /// assert_eq!(uppercase.collect::<Vec<_>>(), b"ABC\xFF\xFEXYZ");
    /// ```
    pub const fn with_ascii_slice(slice: &'a [u8]) -> Self {
        Self {
            iter: Inner::Ascii(ascii::Uppercase::with_slice(slice)),
        }
    }

    pub(crate) const fn with_mode(slice: &'a [u8], mode: Mode) -> Self {
        if !mode.is_turkic() && slice.is_ascii() {
            return Self::with_ascii_slice(slice);
        }
        Self {
            iter: Inner::Full(full::Uppercase::with_mode(slice, mode)),
        }
    }
}

impl Iterator for Uppercase<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter {
            Inner::Empty => None,
            Inner::Full(ref mut iter) => iter.next(),
            Inner::Ascii(ref mut iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.iter {
            Inner::Empty => (0, Some(0)),
            Inner::Full(ref iter) => iter.size_hint(),
            Inner::Ascii(ref iter) => iter.size_hint(),
        }
    }

    fn count(self) -> usize {
        match self.iter {
            Inner::Empty => 0,
            Inner::Full(iter) => iter.count(),
            Inner::Ascii(iter) => iter.count(),
        }
    }
}

impl FusedIterator for Uppercase<'_> {}

/// Options to configure the behavior of [`uppercase`].
///
/// Which letters exactly are replaced, and by which other letters, depends on
/// the given options.
///
/// See individual variants for a description of the available behaviors.
///
/// If you're not sure which mode to choose, [`UppercaseMode::Full`] is a a good
/// default.
///
/// [`uppercase`]: crate::uppercase()
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum UppercaseMode {
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

impl TryFrom<&str> for UppercaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}

impl TryFrom<Option<&str>> for UppercaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        value.map(str::as_bytes).try_into()
    }
}

impl TryFrom<&[u8]> for UppercaseMode {
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

impl TryFrom<Option<&[u8]>> for UppercaseMode {
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

impl FromStr for UppercaseMode {
    type Err = InvalidCaseMappingMode;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

/// Returns an iterator that yields a copy of the bytes in the given slice with
/// all lowercase letters replaced with their uppercase counterparts.
///
/// This function treats the given slice as a [conventionally UTF-8 string].
/// UTF-8 byte sequences are converted to their Unicode uppercase equivalents.
/// Invalid UTF-8 byte sequences are yielded as is.
///
/// The case mapping mode is determined by the given [`UppercaseMode`]. See its
/// documentation for details on the available case mapping modes.
///
/// [conventionally UTF-8 string]: https://docs.rs/bstr/1.*/bstr/#when-should-i-use-byte-strings
pub const fn uppercase(slice: &[u8], options: UppercaseMode) -> Uppercase<'_> {
    match options {
        UppercaseMode::Full | UppercaseMode::Lithuanian => Uppercase::with_slice(slice),
        UppercaseMode::Ascii => Uppercase::with_ascii_slice(slice),
        UppercaseMode::Turkic => Uppercase::with_mode(slice, Mode::TurkicUpper),
    }
}

/// Fallibly uppercase a byte string, borrowing it when no bytes change.
///
/// Uses the same mappings and malformed UTF-8 preservation as [`uppercase`].
/// Returns [`Cow::Borrowed`] with the original slice exactly when the output
/// bytes equal the input. Otherwise, returns [`Cow::Owned`] with the mapped bytes.
///
/// Allocation starts at the first differing output byte. The matching prefix
/// is copied without restarting the mapping iterator. No allocation or copying
/// occurs for unchanged input; determining this still scans the mapping output.
/// The input is never modified, including on allocation failure.
///
/// # Errors
///
/// Returns an error if reserving storage for changed output fails.
///
/// # Examples
///
/// ```
/// # use std::borrow::Cow;
/// # use roe::{UppercaseMode, try_to_uppercase};
/// let input = b"ARTICHOKE";
/// let result = try_to_uppercase(input, UppercaseMode::Full)?;
/// assert!(matches!(result, Cow::Borrowed(_)));
/// assert_eq!(result.as_ptr(), input.as_ptr());
/// # Ok::<(), std::collections::TryReserveError>(())
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[inline]
pub fn try_to_uppercase(
    slice: &[u8],
    options: UppercaseMode,
) -> Result<Cow<'_, [u8]>, TryReserveError> {
    crate::collect::try_collect(slice, uppercase(slice, options))
}

#[cfg(test)]
mod tests {
    use core::{convert::TryInto, str::FromStr};

    use super::UppercaseMode;
    use crate::InvalidCaseMappingMode;
    use alloc::vec::Vec;
    use bstr::ByteSlice;

    use super::Uppercase;

    #[test]
    fn empty() {
        let iter = Uppercase::new();
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());

        let iter = Uppercase::default();
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());

        let iter = Uppercase::with_slice(b"");
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());

        let iter = Uppercase::with_ascii_slice(b"");
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());
    }

    #[test]
    fn size_hint() {
        assert_eq!(Uppercase::new().size_hint(), (0, Some(0)));
        assert_eq!(Uppercase::default().size_hint(), (0, Some(0)));
        assert_eq!(Uppercase::with_slice(b"").size_hint(), (0, Some(0)));

        assert_eq!(Uppercase::with_slice(b"abc, xyz").size_hint(), (8, Some(8)));
        assert_eq!(
            Uppercase::with_slice(b"abc, \xFF\xFE, xyz").size_hint(),
            (3, Some(144))
        );
        assert_eq!(
            Uppercase::with_slice("�".as_bytes()).size_hint(),
            (1, Some(36))
        );
        assert_eq!(
            Uppercase::with_slice("Έτος".as_bytes()).size_hint(),
            (2, Some(96))
        );
        assert_eq!(
            Uppercase::with_slice("ZȺȾ".as_bytes()).size_hint(),
            (2, Some(60))
        );

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Uppercase::with_slice(&utf8_with_invalid_bytes).size_hint(),
            (3, Some(120))
        );

        assert_eq!(
            Uppercase::with_ascii_slice(b"abc, xyz").size_hint(),
            (8, Some(8))
        );
        assert_eq!(
            Uppercase::with_ascii_slice(b"abc, \xFF\xFE, xyz").size_hint(),
            (12, Some(12))
        );
        assert_eq!(
            Uppercase::with_ascii_slice("�".as_bytes()).size_hint(),
            (3, Some(3))
        );
        assert_eq!(
            Uppercase::with_ascii_slice("Έτος".as_bytes()).size_hint(),
            (8, Some(8))
        );
        assert_eq!(
            Uppercase::with_ascii_slice("ZȺȾ".as_bytes()).size_hint(),
            (5, Some(5))
        );

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Uppercase::with_ascii_slice(&utf8_with_invalid_bytes).size_hint(),
            (10, Some(10))
        );
    }

    #[test]
    fn count() {
        assert_eq!(Uppercase::new().count(), 0);
        assert_eq!(Uppercase::default().count(), 0);
        assert_eq!(Uppercase::with_slice(b"").count(), 0);

        assert_eq!(Uppercase::with_slice(b"abc, xyz").count(), 8);
        assert_eq!(Uppercase::with_slice(b"abc, \xFF\xFE, xyz").count(), 12);
        assert_eq!(Uppercase::with_slice("�".as_bytes()).count(), 3);
        assert_eq!(Uppercase::with_slice("Έτος".as_bytes()).count(), 8);
        assert_eq!(Uppercase::with_slice("zⱥⱦ".as_bytes()).count(), 5);

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(Uppercase::with_slice(&utf8_with_invalid_bytes).count(), 10);

        assert_eq!(Uppercase::with_ascii_slice(b"abc, xyz").count(), 8);
        assert_eq!(
            Uppercase::with_ascii_slice(b"abc, \xFF\xFE, xyz").count(),
            12
        );
        assert_eq!(Uppercase::with_ascii_slice("�".as_bytes()).count(), 3);
        assert_eq!(Uppercase::with_ascii_slice("Έτος".as_bytes()).count(), 8);
        assert_eq!(Uppercase::with_ascii_slice("ZȺȾ".as_bytes()).count(), 5);

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Uppercase::with_ascii_slice(&utf8_with_invalid_bytes).count(),
            10
        );
    }

    #[test]
    fn size_hint_covers_count() {
        let iter = Uppercase::new();
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());
    }
    #[test]
    fn test_uppercase_mode_parsing() {
        assert_eq!(UppercaseMode::from_str("ascii"), Ok(UppercaseMode::Ascii));
        assert_eq!(UppercaseMode::from_str("turkic"), Ok(UppercaseMode::Turkic));
        assert_eq!(
            UppercaseMode::from_str("lithuanian"),
            Ok(UppercaseMode::Lithuanian)
        );
        assert_eq!(
            UppercaseMode::from_str("full"),
            Err(InvalidCaseMappingMode::new())
        );
    }

    #[test]
    fn test_uppercase_mode_conversion() {
        let mut mode: UppercaseMode;
        mode = "turkic".try_into().unwrap();
        assert_eq!(mode, UppercaseMode::Turkic);

        mode = Some("turkic").try_into().unwrap();
        assert_eq!(mode, UppercaseMode::Turkic);

        mode = b"turkic"[..].try_into().unwrap();
        assert_eq!(mode, UppercaseMode::Turkic);

        mode = Some(&b"turkic"[..]).try_into().unwrap();
        assert_eq!(mode, UppercaseMode::Turkic);
    }
}
