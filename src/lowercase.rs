use core::iter::FusedIterator;

mod ascii;
mod full;

#[derive(Debug, Clone)]
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
}
