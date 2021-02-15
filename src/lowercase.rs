use core::iter::FusedIterator;

mod ascii;
mod full;

#[derive(Debug, Clone)]
#[allow(variant_size_differences)]
enum Inner<'a> {
    Empty,
    Full(full::Lowercase<'a>),
    Ascii(ascii::Lowercase<'a>),
}

/// An iterator that yields the lowercase equivalent of a conventionally UTF-8
/// byte string.
///
/// This iterator yields [bytes].
///
/// This struct is created by the [`lowercase`] function. See its documentation
/// for more.
///
/// [bytes]: u8
/// [`lowercase`]: crate::lowercase()
#[derive(Debug, Clone)]
#[must_use = "Lowercase is a Iterator and must be used"]
pub struct Lowercase<'a> {
    iter: Inner<'a>,
}

impl<'a> Lowercase<'a> {
    /// Create a new, empty lowercase iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::Lowercase;
    /// let mut lowercase = Lowercase::new();
    /// assert_eq!(lowercase.next(), None);
    /// ```
    pub const fn new() -> Self {
        Self { iter: Inner::Empty }
    }

    /// Create a new lowercase iterator with the given byte slice using full
    /// Unicode case mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::Lowercase;
    /// let mut lowercase = Lowercase::with_slice(b"abcXYZ");
    /// assert_eq!(lowercase.next(), Some(b'a'));
    /// assert_eq!(lowercase.next(), Some(b'b'));
    /// assert_eq!(lowercase.next(), Some(b'c'));
    /// assert_eq!(lowercase.next(), Some(b'x'));
    /// assert_eq!(lowercase.next(), Some(b'y'));
    /// assert_eq!(lowercase.next(), Some(b'z'));
    /// assert_eq!(lowercase.next(), None);
    /// ```
    ///
    /// Non-ASCII characters are case mapped:
    ///
    /// ```
    /// # use roe::Lowercase;
    /// let lowercase = Lowercase::with_slice("Αύριο".as_bytes());
    /// assert_eq!(lowercase.collect::<Vec<_>>(), "αύριο".as_bytes());
    /// ```
    ///
    /// Invalid UTF-8 bytes are yielded as is without impacting Unicode
    /// characters:
    ///
    /// ```
    /// # use roe::Lowercase;
    /// let mut s = "Αύριο".to_string().into_bytes();
    /// s.extend(b"\xFF\xFE");
    /// let lowercase = Lowercase::with_slice(s.as_slice());
    ///
    /// let mut expected = "αύριο".to_string().into_bytes();
    /// expected.extend(b"\xFF\xFE");
    /// assert_eq!(lowercase.collect::<Vec<_>>(), expected);
    /// ```
    pub const fn with_slice(slice: &'a [u8]) -> Self {
        Self {
            iter: Inner::Full(full::Lowercase::with_slice(slice)),
        }
    }

    /// Create a new lowercase iterator with the given byte slice using ASCII
    /// case mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::Lowercase;
    /// let mut lowercase = Lowercase::with_ascii_slice(b"abcXYZ");
    /// assert_eq!(lowercase.next(), Some(b'a'));
    /// assert_eq!(lowercase.next(), Some(b'b'));
    /// assert_eq!(lowercase.next(), Some(b'c'));
    /// assert_eq!(lowercase.next(), Some(b'x'));
    /// assert_eq!(lowercase.next(), Some(b'y'));
    /// assert_eq!(lowercase.next(), Some(b'z'));
    /// assert_eq!(lowercase.next(), None);
    /// ```
    ///
    /// Non-ASCII characters are ignored:
    ///
    /// ```
    /// # use roe::Lowercase;
    /// let lowercase = Lowercase::with_ascii_slice("Αύριο".as_bytes());
    /// assert_eq!(lowercase.collect::<Vec<_>>(), "Αύριο".as_bytes());
    /// ```
    ///
    /// Invalid UTF-8 bytes are yielded as is without impacting ASCII bytes:
    ///
    /// ```
    /// # use roe::Lowercase;
    /// let lowercase = Lowercase::with_ascii_slice(b"abc\xFF\xFEXYZ");
    /// assert_eq!(lowercase.collect::<Vec<_>>(), b"abc\xFF\xFExyz");
    /// ```
    pub const fn with_ascii_slice(slice: &'a [u8]) -> Self {
        Self {
            iter: Inner::Ascii(ascii::Lowercase::with_slice(slice)),
        }
    }
}

impl<'a> Iterator for Lowercase<'a> {
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

impl<'a> FusedIterator for Lowercase<'a> {}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use bstr::ByteSlice;

    use super::Lowercase;

    #[test]
    fn empty() {
        let iter = Lowercase::new();
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());

        let iter = Lowercase::with_slice(b"");
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());

        let iter = Lowercase::with_ascii_slice(b"");
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());
    }

    #[test]
    fn size_hint() {
        assert_eq!(Lowercase::new().size_hint(), (0, Some(0)));

        assert_eq!(Lowercase::with_slice(b"abc, xyz").size_hint(), (8, Some(8)));
        assert_eq!(
            Lowercase::with_slice(b"abc, \xFF\xFE, xyz").size_hint(),
            (12, Some(144))
        );
        assert_eq!(
            Lowercase::with_slice("�".as_bytes()).size_hint(),
            (3, Some(36))
        );
        assert_eq!(
            Lowercase::with_slice("Έτος".as_bytes()).size_hint(),
            (8, Some(96))
        );
        assert_eq!(
            Lowercase::with_slice("ZȺȾ".as_bytes()).size_hint(),
            (5, Some(60))
        );

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Lowercase::with_slice(&utf8_with_invalid_bytes).size_hint(),
            (10, Some(120))
        );

        assert_eq!(
            Lowercase::with_ascii_slice(b"abc, xyz").size_hint(),
            (8, Some(8))
        );
        assert_eq!(
            Lowercase::with_ascii_slice(b"abc, \xFF\xFE, xyz").size_hint(),
            (12, Some(12))
        );
        assert_eq!(
            Lowercase::with_ascii_slice("�".as_bytes()).size_hint(),
            (3, Some(3))
        );
        assert_eq!(
            Lowercase::with_ascii_slice("Έτος".as_bytes()).size_hint(),
            (8, Some(8))
        );
        assert_eq!(
            Lowercase::with_ascii_slice("ZȺȾ".as_bytes()).size_hint(),
            (5, Some(5))
        );

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Lowercase::with_ascii_slice(&utf8_with_invalid_bytes).size_hint(),
            (10, Some(10))
        );
    }

    #[test]
    fn count() {
        assert_eq!(Lowercase::new().count(), 0);

        assert_eq!(Lowercase::with_slice(b"abc, xyz").count(), 8);
        assert_eq!(Lowercase::with_slice(b"abc, \xFF\xFE, xyz").count(), 12);
        assert_eq!(Lowercase::with_slice("�".as_bytes()).count(), 3);
        assert_eq!(Lowercase::with_slice("Έτος".as_bytes()).count(), 8);
        assert_eq!(Lowercase::with_slice("ZȺȾ".as_bytes()).count(), 7);

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(Lowercase::with_slice(&utf8_with_invalid_bytes).count(), 10);

        assert_eq!(Lowercase::with_ascii_slice(b"abc, xyz").count(), 8);
        assert_eq!(
            Lowercase::with_ascii_slice(b"abc, \xFF\xFE, xyz").count(),
            12
        );
        assert_eq!(Lowercase::with_ascii_slice("�".as_bytes()).count(), 3);
        assert_eq!(Lowercase::with_ascii_slice("Έτος".as_bytes()).count(), 8);
        assert_eq!(Lowercase::with_ascii_slice("ZȺȾ".as_bytes()).count(), 5);

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Lowercase::with_ascii_slice(&utf8_with_invalid_bytes).count(),
            10
        );
    }

    #[test]
    fn size_hint_covers_count() {
        let iter = Lowercase::new();
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());
    }
}
