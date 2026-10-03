use core::iter::FusedIterator;

use crate::unicode::mapping::Mode;

mod ascii;
mod full;

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

#[cfg(test)]
mod tests {
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
}
