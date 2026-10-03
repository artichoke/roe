use core::convert::{TryFrom, TryInto};
use core::iter::FusedIterator;
use core::str::FromStr;

use crate::InvalidCaseMappingMode;
use crate::unicode::mapping::Mode;

mod ascii;
mod full;
mod georgian;

pub use ascii::make_ascii_capitalize;
#[cfg(feature = "alloc")]
pub use ascii::to_ascii_capitalize;

#[derive(Debug, Clone)]
#[allow(variant_size_differences)]
enum Inner<'a> {
    Empty,
    Full(full::Capitalize<'a>),
    Ascii(ascii::Capitalize<'a>),
}

/// An iterator that yields the Ruby-style capitalized equivalent of a
/// conventionally UTF-8 byte string.
///
/// This iterator yields [bytes].
///
/// This struct is created by the [`capitalize`] function. See its documentation
/// for more.
///
/// [bytes]: u8
/// [`capitalize`]: crate::ruby::capitalize()
#[derive(Debug, Clone)]
#[must_use = "Capitalize is an Iterator and must be used"]
pub struct Capitalize<'a> {
    iter: Inner<'a>,
}

impl Default for Capitalize<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Capitalize<'a> {
    /// Create a new, empty capitalize iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::ruby::Capitalize;
    /// let mut capitalize = Capitalize::new();
    /// assert_eq!(capitalize.next(), None);
    /// ```
    pub const fn new() -> Self {
        Self { iter: Inner::Empty }
    }

    /// Create a new capitalize iterator with the given byte slice using full
    /// Unicode case mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::ruby::Capitalize;
    /// let mut capitalize = Capitalize::with_slice(b"abcXYZ");
    /// assert_eq!(capitalize.next(), Some(b'A'));
    /// assert_eq!(capitalize.next(), Some(b'b'));
    /// assert_eq!(capitalize.next(), Some(b'c'));
    /// assert_eq!(capitalize.next(), Some(b'x'));
    /// assert_eq!(capitalize.next(), Some(b'y'));
    /// assert_eq!(capitalize.next(), Some(b'z'));
    /// assert_eq!(capitalize.next(), None);
    /// ```
    ///
    /// Non-ASCII characters are case mapped:
    ///
    /// ```
    /// # use roe::ruby::Capitalize;
    /// let capitalize = Capitalize::with_slice("αύριο".as_bytes());
    /// assert_eq!(capitalize.collect::<Vec<_>>(), "Αύριο".as_bytes());
    ///
    ///
    /// let capitalize = Capitalize::with_slice("ﬃ".as_bytes());
    /// assert_eq!(capitalize.collect::<Vec<_>>(), "Ffi".as_bytes());
    /// ```
    ///
    /// Invalid UTF-8 bytes are yielded as is without impacting Unicode
    /// characters:
    ///
    /// ```
    /// # use roe::ruby::Capitalize;
    /// let mut s = "αύριο".to_string().into_bytes();
    /// s.extend(b"\xFF\xFE");
    /// let capitalize = Capitalize::with_slice(s.as_slice());
    ///
    /// let mut expected = "Αύριο".to_string().into_bytes();
    /// expected.extend(b"\xFF\xFE");
    /// assert_eq!(capitalize.collect::<Vec<_>>(), expected);
    /// ```
    pub const fn with_slice(slice: &'a [u8]) -> Self {
        // Full Unicode and ASCII mappings agree for an entirely ASCII input.
        if slice.is_ascii() {
            return Self::with_ascii_slice(slice);
        }
        Self {
            iter: Inner::Full(full::Capitalize::with_slice(slice)),
        }
    }

    /// Create a new capitalize iterator with the given byte slice using ASCII
    /// case mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::ruby::Capitalize;
    /// let mut capitalize = Capitalize::with_ascii_slice(b"abcXYZ");
    /// assert_eq!(capitalize.next(), Some(b'A'));
    /// assert_eq!(capitalize.next(), Some(b'b'));
    /// assert_eq!(capitalize.next(), Some(b'c'));
    /// assert_eq!(capitalize.next(), Some(b'x'));
    /// assert_eq!(capitalize.next(), Some(b'y'));
    /// assert_eq!(capitalize.next(), Some(b'z'));
    /// assert_eq!(capitalize.next(), None);
    /// ```
    ///
    /// Non-ASCII characters are ignored:
    ///
    /// ```
    /// # use roe::ruby::Capitalize;
    /// let capitalize = Capitalize::with_ascii_slice("αΎρΙο".as_bytes());
    /// assert_eq!(capitalize.collect::<Vec<_>>(), "αΎρΙο".as_bytes());
    /// ```
    ///
    /// Invalid UTF-8 bytes are yielded as is without impacting ASCII bytes:
    ///
    /// ```
    /// # use roe::ruby::Capitalize;
    /// let capitalize = Capitalize::with_ascii_slice(b"abc\xFF\xFEXYZ");
    /// assert_eq!(capitalize.collect::<Vec<_>>(), b"Abc\xFF\xFExyz");
    /// ```
    pub const fn with_ascii_slice(slice: &'a [u8]) -> Self {
        Self {
            iter: Inner::Ascii(ascii::Capitalize::with_slice(slice)),
        }
    }

    pub(crate) const fn with_mode(slice: &'a [u8], mode: Mode) -> Self {
        if !mode.is_turkic() && slice.is_ascii() {
            return Self::with_ascii_slice(slice);
        }
        Self {
            iter: Inner::Full(full::Capitalize::with_mode(slice, mode)),
        }
    }
}

impl Iterator for Capitalize<'_> {
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

impl FusedIterator for Capitalize<'_> {}

/// Options to configure the behavior of [`capitalize`].
///
/// Which letters exactly are replaced, and by which other letters, depends on
/// the given options.
///
/// See individual variants for a description of the available behaviors.
///
/// If you're not sure which mode to choose, [`CapitalizeMode::Full`] is a good
/// default.
///
/// [`capitalize`]: crate::ruby::capitalize()
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapitalizeMode {
    /// Full Unicode case mapping with Ruby string capitalization semantics.
    ///
    /// Georgian Mtavruli capitals are lowercased to Mkhedruli, including the
    /// initial character: `ᲐᲑᲒ` becomes `აბგ`. Modern Georgian uses Mtavruli for
    /// all-caps emphasis, without initial-letter capitalization. This avoids
    /// producing mixed forms such as `Აბგ`.
    ///
    /// This differs from [`to_titlecase(char)`](crate::to_titlecase), which
    /// follows Unicode's character mapping and leaves Mtavruli unchanged.
    /// See [`capitalize`](crate::ruby::capitalize()) for an example and rationale.
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
    /// This means that upper case I is mapped to titlecase dotless i, and so
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

impl TryFrom<&str> for CapitalizeMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}

impl TryFrom<Option<&str>> for CapitalizeMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        value.map(str::as_bytes).try_into()
    }
}

impl TryFrom<&[u8]> for CapitalizeMode {
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

impl TryFrom<Option<&[u8]>> for CapitalizeMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&[u8]>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(Self::default()),
            Some(value) => value.try_into(),
        }
    }
}

impl FromStr for CapitalizeMode {
    type Err = InvalidCaseMappingMode;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

/// Returns an iterator that yields a copy of the bytes in the given slice with
/// the leading letter replaced with its titlecase counterpart and all remaining
/// letters replaced with their lowercase counterparts.
///
/// This function treats the given slice as a [conventionally UTF-8 string].
/// The first valid character is titlecased and the remainder is lowercased,
/// following Ruby capitalization semantics, including Georgian Mtavruli.
/// Invalid UTF-8 byte sequences are yielded as is.
///
/// The case mapping mode is determined by the given [`CapitalizeMode`]. See its
/// documentation for details on the available case mapping modes.
///
/// # Georgian capitalization
///
/// Modern Georgian uses Mkhedruli for ordinary text and Mtavruli for all-caps
/// emphasis, without capitalizing only the initial letter. Unicode's character
/// titlecase mappings leave both forms unchanged. Titlecasing an initial
/// Mtavruli character and lowercasing the rest would therefore produce an
/// inappropriate mixed form such as `Აბგ`.
///
/// Like MRI, this function instead lowercases the initial Mtavruli character,
/// producing `აბგ`. This applies to [`CapitalizeMode::Full`],
/// [`CapitalizeMode::Turkic`], and [`CapitalizeMode::Lithuanian`]. The character
/// mapping function [`crate::to_titlecase`] retains Unicode's unchanged Mtavruli
/// mapping.
/// See [Ruby issue #14839](https://bugs.ruby-lang.org/issues/14839) for the
/// rationale and feedback from Georgian speakers.
///
/// ```
/// use roe::ruby::{capitalize, CapitalizeMode};
/// use roe::to_titlecase;
///
/// let capitalized: Vec<u8> = capitalize("ᲐᲑᲒ".as_bytes(), CapitalizeMode::Full).collect();
/// assert_eq!(capitalized, "აბგ".as_bytes());
/// assert_eq!(to_titlecase('Ა'), ['Ა', '\0', '\0']);
/// ```
///
/// [conventionally UTF-8 string]: https://docs.rs/bstr/1.*/bstr/#when-should-i-use-byte-strings
pub const fn capitalize(slice: &[u8], options: CapitalizeMode) -> Capitalize<'_> {
    match options {
        CapitalizeMode::Full | CapitalizeMode::Lithuanian => Capitalize::with_slice(slice),
        CapitalizeMode::Ascii => Capitalize::with_ascii_slice(slice),
        CapitalizeMode::Turkic => Capitalize::with_mode(slice, Mode::TurkicTitle),
    }
}

#[cfg(test)]
mod tests {
    use core::{convert::TryInto, str::FromStr};

    use super::CapitalizeMode;
    use crate::InvalidCaseMappingMode;
    use alloc::vec::Vec;
    use bstr::ByteSlice;

    use super::Capitalize;

    #[test]
    fn empty() {
        let iter = Capitalize::new();
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());

        let iter = Capitalize::default();
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());

        let iter = Capitalize::with_slice(b"");
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());

        let iter = Capitalize::with_ascii_slice(b"");
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());
    }

    #[test]
    fn size_hint() {
        assert_eq!(Capitalize::new().size_hint(), (0, Some(0)));
        assert_eq!(Capitalize::default().size_hint(), (0, Some(0)));
        assert_eq!(Capitalize::with_slice(b"").size_hint(), (0, Some(0)));

        assert_eq!(
            Capitalize::with_slice(b"abc, xyz").size_hint(),
            (8, Some(8))
        );
        assert_eq!(
            Capitalize::with_slice(b"abc, \xFF\xFE, xyz").size_hint(),
            (3, Some(144))
        );
        assert_eq!(
            Capitalize::with_slice("�".as_bytes()).size_hint(),
            (1, Some(36))
        );
        assert_eq!(
            Capitalize::with_slice("Έτος".as_bytes()).size_hint(),
            (2, Some(96))
        );
        assert_eq!(
            Capitalize::with_slice("ZȺȾ".as_bytes()).size_hint(),
            (2, Some(60))
        );

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Capitalize::with_slice(&utf8_with_invalid_bytes).size_hint(),
            (3, Some(120))
        );

        assert_eq!(
            Capitalize::with_ascii_slice(b"abc, xyz").size_hint(),
            (8, Some(8))
        );
        assert_eq!(
            Capitalize::with_ascii_slice(b"abc, \xFF\xFE, xyz").size_hint(),
            (12, Some(12))
        );
        assert_eq!(
            Capitalize::with_ascii_slice("�".as_bytes()).size_hint(),
            (3, Some(3))
        );
        assert_eq!(
            Capitalize::with_ascii_slice("Έτος".as_bytes()).size_hint(),
            (8, Some(8))
        );
        assert_eq!(
            Capitalize::with_ascii_slice("ZȺȾ".as_bytes()).size_hint(),
            (5, Some(5))
        );

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Capitalize::with_ascii_slice(&utf8_with_invalid_bytes).size_hint(),
            (10, Some(10))
        );
    }

    #[test]
    fn count() {
        assert_eq!(Capitalize::new().count(), 0);
        assert_eq!(Capitalize::default().count(), 0);
        assert_eq!(Capitalize::with_slice(b"").count(), 0);

        assert_eq!(Capitalize::with_slice(b"abc, xyz").count(), 8);
        assert_eq!(Capitalize::with_slice(b"abc, \xFF\xFE, xyz").count(), 12);
        assert_eq!(Capitalize::with_slice("�".as_bytes()).count(), 3);
        assert_eq!(Capitalize::with_slice("Έτος".as_bytes()).count(), 8);
        assert_eq!(Capitalize::with_slice("ZȺȾ".as_bytes()).count(), 7);

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(Capitalize::with_slice(&utf8_with_invalid_bytes).count(), 10);

        assert_eq!(Capitalize::with_ascii_slice(b"abc, xyz").count(), 8);
        assert_eq!(
            Capitalize::with_ascii_slice(b"abc, \xFF\xFE, xyz").count(),
            12
        );
        assert_eq!(Capitalize::with_ascii_slice("�".as_bytes()).count(), 3);
        assert_eq!(Capitalize::with_ascii_slice("Έτος".as_bytes()).count(), 8);
        assert_eq!(Capitalize::with_ascii_slice("ZȺȾ".as_bytes()).count(), 5);

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Capitalize::with_ascii_slice(&utf8_with_invalid_bytes).count(),
            10
        );
    }

    #[test]
    fn size_hint_covers_count() {
        let iter = Capitalize::new();
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());
    }
    #[test]
    fn test_capitalize_mode_parsing() {
        assert_eq!(CapitalizeMode::from_str("ascii"), Ok(CapitalizeMode::Ascii));
        assert_eq!(
            CapitalizeMode::from_str("turkic"),
            Ok(CapitalizeMode::Turkic)
        );
        assert_eq!(
            CapitalizeMode::from_str("lithuanian"),
            Ok(CapitalizeMode::Lithuanian)
        );
        assert_eq!(
            CapitalizeMode::from_str("full"),
            Err(InvalidCaseMappingMode::new())
        );
    }

    #[test]
    fn test_capitalize_mode_conversion() {
        let mut mode: CapitalizeMode;
        mode = "turkic".try_into().unwrap();
        assert_eq!(mode, CapitalizeMode::Turkic);

        mode = Some("turkic").try_into().unwrap();
        assert_eq!(mode, CapitalizeMode::Turkic);

        mode = b"turkic"[..].try_into().unwrap();
        assert_eq!(mode, CapitalizeMode::Turkic);

        mode = Some(&b"turkic"[..]).try_into().unwrap();
        assert_eq!(mode, CapitalizeMode::Turkic);
    }
}
