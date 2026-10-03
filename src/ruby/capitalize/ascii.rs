#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use core::fmt;
use core::iter::FusedIterator;

use bstr::ByteSlice;

#[derive(Clone)]
#[must_use = "Capitalize is an Iterator and must be used"]
pub struct Capitalize<'a> {
    slice: &'a [u8],
    head_yielded: bool,
}

impl fmt::Debug for Capitalize<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Capitalize")
            .field("slice", &self.slice.as_bstr())
            .finish()
    }
}

impl<'a> From<&'a [u8]> for Capitalize<'a> {
    fn from(slice: &'a [u8]) -> Self {
        Self::with_slice(slice)
    }
}

impl<'a> Capitalize<'a> {
    pub const fn with_slice(slice: &'a [u8]) -> Self {
        Self {
            slice,
            head_yielded: false,
        }
    }
}

impl Iterator for Capitalize<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let (&byte, remainder) = self.slice.split_first()?;
        self.slice = remainder;
        if self.head_yielded {
            Some(byte.to_ascii_lowercase())
        } else {
            self.head_yielded = true;
            Some(byte.to_ascii_uppercase())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.slice.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.slice.len()
    }
}

impl DoubleEndedIterator for Capitalize<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (&byte, remainder) = self.slice.split_last()?;
        self.slice = remainder;
        if remainder.is_empty() {
            if self.head_yielded {
                Some(byte.to_ascii_lowercase())
            } else {
                self.head_yielded = true;
                Some(byte.to_ascii_uppercase())
            }
        } else {
            Some(byte.to_ascii_lowercase())
        }
    }
}

impl ExactSizeIterator for Capitalize<'_> {}

impl FusedIterator for Capitalize<'_> {}

/// Converts the given slice to its ASCII capitalization equivalent in-place.
///
/// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z' in the first byte;
/// subsequent bytes with ASCII letters 'A' to 'Z' are mapped to 'a' to 'z';
/// non-ASCII letters are unchanged.
///
/// This function can be used to implement [`String#capitalize!`] for ASCII
/// strings in Ruby.
///
#[cfg_attr(
    feature = "alloc",
    doc = "To return a new capitalized value without modifying the existing one, use [`to_ascii_capitalize`]."
)]
///
/// # Examples
///
/// ```
/// # use roe::ruby::make_ascii_capitalize;
/// let mut buf = *b"ABCxyz";
/// make_ascii_capitalize(&mut buf);
/// assert_eq!(buf, *b"Abcxyz");
///
/// let mut buf = *b"1234%&*";
/// make_ascii_capitalize(&mut buf);
/// assert_eq!(buf, *b"1234%&*");
///
/// let mut buf = *b"ABC1234%&*";
/// make_ascii_capitalize(&mut buf);
/// assert_eq!(buf, *b"Abc1234%&*");
///
/// let mut buf = *b"1234%&*abcXYZ";
/// make_ascii_capitalize(&mut buf);
/// assert_eq!(buf, *b"1234%&*abcxyz");
///
/// let mut buf = *b"ABC, XYZ";
/// make_ascii_capitalize(&mut buf);
/// assert_eq!(buf, *b"Abc, xyz");
/// ```
///
/// [`String#capitalize!`]: https://docs.ruby-lang.org/en/4.0/String.html#method-i-capitalize-21
#[inline]
#[allow(clippy::module_name_repetitions)]
pub fn make_ascii_capitalize<T: AsMut<[u8]>>(slice: &mut T) {
    let slice = slice.as_mut();
    if let Some((head, tail)) = slice.split_first_mut() {
        head.make_ascii_uppercase();
        tail.make_ascii_lowercase();
    }
}

/// Returns a vector containing a copy of the given slice where each byte is
/// mapped to its ASCII capitalization equivalent.
///
/// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z' in the first byte;
/// subsequent bytes with ASCII letters 'A' to 'Z' are mapped to 'a' to 'z';
/// non-ASCII letters are unchanged.
///
/// This function can be used to implement [`String#capitalize`] and
/// [`Symbol#capitalize`] for ASCII strings in Ruby.
///
/// To capitalize the value in-place, use [`make_ascii_capitalize`].
///
/// # Examples
///
/// ```
/// # use roe::ruby::to_ascii_capitalize;
/// assert_eq!(to_ascii_capitalize("ABCxyz"), &b"Abcxyz"[..]);
/// assert_eq!(to_ascii_capitalize("1234%&*"), &b"1234%&*"[..]);
/// assert_eq!(to_ascii_capitalize("ABC1234%&*"), &b"Abc1234%&*"[..]);
/// assert_eq!(to_ascii_capitalize("1234%&*abcXYZ"), &b"1234%&*abcxyz"[..]);
/// assert_eq!(to_ascii_capitalize("ABC, XYZ"), &b"Abc, xyz"[..]);
/// ```
///
/// [`String#capitalize`]: https://docs.ruby-lang.org/en/4.0/String.html#method-i-capitalize
/// [`Symbol#capitalize`]: https://docs.ruby-lang.org/en/4.0/Symbol.html#method-i-capitalize
#[inline]
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[allow(clippy::module_name_repetitions)]
pub fn to_ascii_capitalize<T: AsRef<[u8]>>(slice: T) -> Vec<u8> {
    let slice = slice.as_ref();
    let mut capitalized = slice.to_ascii_lowercase();
    if let Some(head) = capitalized.first_mut() {
        head.make_ascii_uppercase();
    }
    capitalized
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use bstr::ByteSlice;

    use super::Capitalize;

    #[test]
    fn empty() {
        let iter = Capitalize::from(&b""[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());
    }

    #[test]
    fn ascii() {
        let iter = Capitalize::from(&b"abc"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"Abc".as_bstr());

        let iter = Capitalize::from(&b"aBC"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"Abc".as_bstr());

        let iter = Capitalize::from(&b"ABC"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"Abc".as_bstr());

        let iter = Capitalize::from(&b"aBC, 123, ABC, baby you and me girl"[..]);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            b"Abc, 123, abc, baby you and me girl".as_bstr()
        );
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn utf8() {
        let s = "ß".as_bytes();
        let iter = Capitalize::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "ß".as_bytes().as_bstr());

        let s = "Αύριο".as_bytes();
        let iter = Capitalize::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "Αύριο".as_bytes().as_bstr()
        );

        let s = "Έτος".as_bytes();
        let iter = Capitalize::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "Έτος".as_bytes().as_bstr()
        );

        // two-byte characters
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L198-L200
        let s = "𐐜 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐙𐐊𐐡𐐝𐐓/𐐝𐐇𐐗𐐊𐐤𐐔 𐐒𐐋𐐗 𐐒𐐌 𐐜 𐐡𐐀𐐖𐐇𐐤𐐓𐐝 𐐱𐑂 𐑄 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐏𐐆𐐅𐐤𐐆𐐚𐐊𐐡𐐝𐐆𐐓𐐆".as_bytes();
        let iter = Capitalize::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "𐐜 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐙𐐊𐐡𐐝𐐓/𐐝𐐇𐐗𐐊𐐤𐐔 𐐒𐐋𐐗 𐐒𐐌 𐐜 𐐡𐐀𐐖𐐇𐐤𐐓𐐝 𐐱𐑂 𐑄 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐏𐐆𐐅𐐤𐐆𐐚𐐊𐐡𐐝𐐆𐐓𐐆"
                .as_bytes()
                .as_bstr()
        );

        // Change length when titlecased
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L226-L232
        let s = "ⱥȾȾZ".as_bytes();
        let titlecased = Capitalize::from(s).collect::<Vec<_>>();
        assert_eq!(titlecased.as_bstr(), "ⱥȾȾz".as_bytes().as_bstr());
        assert_eq!(s.len(), titlecased.len());
    }

    #[test]
    fn invalid_utf8() {
        let iter = Capitalize::from(&b"\xFF\xFE"[..]);
        assert_eq!(iter.collect::<Vec<u8>>().as_bstr(), b"\xFF\xFE".as_bstr());

        let iter = Capitalize::from(&b"ABC\xFF\xFEXYZ"[..]);
        assert_eq!(
            iter.collect::<Vec<u8>>().as_bstr(),
            b"Abc\xFF\xFExyz".as_bstr()
        );

        let iter = Capitalize::from(&b"abc\xFF\xFEXYZ"[..]);
        assert_eq!(
            iter.collect::<Vec<u8>>().as_bstr(),
            b"Abc\xFF\xFExyz".as_bstr()
        );

        // The bytes \xF0\x9F\x87 could lead to a valid UTF-8 sequence, but 3 of
        // them on their own are invalid. Only one replacement codepoint is
        // substituted, which demonstrates the "substitution of maximal
        // subparts" strategy.
        //
        // See: https://docs.rs/bstr/1.*/bstr/#handling-of-invalid-utf-8
        let iter = Capitalize::from(&b"aB\xF0\x9F\x87Yz"[..]);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            b"Ab\xF0\x9F\x87yz".as_bstr()
        );
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn unicode_replacement_character() {
        let s = "�".as_bytes();
        let iter = Capitalize::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "�".as_bytes().as_bstr());
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn dz_titlecase() {
        let s = "ǅ".as_bytes();
        let iter = Capitalize::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "ǅ".as_bytes().as_bstr());
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn latin_capital_i_with_dot_above() {
        let s = "İ".as_bytes();
        let iter = Capitalize::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "İ".as_bytes().as_bstr());
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn case_map_to_two_chars() {
        let s = "İ".as_bytes();
        let iter = Capitalize::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "İ".as_bytes().as_bstr());
    }

    #[test]
    fn size_hint() {
        assert_eq!(Capitalize::with_slice(b"").size_hint(), (0, Some(0)));
        assert_eq!(
            Capitalize::with_slice(b"abc, xyz").size_hint(),
            (8, Some(8))
        );
        assert_eq!(
            Capitalize::with_slice(b"abc, \xFF\xFE, xyz").size_hint(),
            (12, Some(12))
        );
        assert_eq!(
            Capitalize::with_slice("�".as_bytes()).size_hint(),
            (3, Some(3))
        );
        assert_eq!(
            Capitalize::with_slice("Έτος".as_bytes()).size_hint(),
            (8, Some(8))
        );
        assert_eq!(
            Capitalize::with_slice("ZȺȾ".as_bytes()).size_hint(),
            (5, Some(5))
        );

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Capitalize::with_slice(&utf8_with_invalid_bytes).size_hint(),
            (10, Some(10))
        );
    }

    #[test]
    fn count() {
        assert_eq!(Capitalize::with_slice(b"").count(), 0);
        assert_eq!(Capitalize::with_slice(b"abc, xyz").count(), 8);
        assert_eq!(Capitalize::with_slice(b"abc, \xFF\xFE, xyz").count(), 12);
        assert_eq!(Capitalize::with_slice("�".as_bytes()).count(), 3);
        assert_eq!(Capitalize::with_slice("Έτος".as_bytes()).count(), 8);
        assert_eq!(Capitalize::with_slice("ZȺȾ".as_bytes()).count(), 5);

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(Capitalize::with_slice(&utf8_with_invalid_bytes).count(), 10);
    }

    #[test]
    fn size_hint_covers_count() {
        let iter = Capitalize::with_slice(b"");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Capitalize::with_slice(b"abc, xyz");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Capitalize::with_slice(b"abc, \xFF\xFE, xyz");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Capitalize::with_slice("�".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Capitalize::with_slice("Έτος".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Capitalize::with_slice("ZȺȾ".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        let iter = Capitalize::with_slice(&utf8_with_invalid_bytes);
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());
    }

    #[test]
    fn double_ended_iterator() {
        let mut iter = Capitalize::with_slice(b"abc");
        assert_eq!(iter.next_back(), Some(b'c'));
        assert_eq!(iter.next_back(), Some(b'b'));
        assert_eq!(iter.next_back(), Some(b'A'));

        let mut iter = Capitalize::with_slice(b"abc");
        assert_eq!(iter.next(), Some(b'A'));
        assert_eq!(iter.next_back(), Some(b'c'));
        assert_eq!(iter.next_back(), Some(b'b'));
    }
    #[test]
    fn make_ascii_capitalize_empty() {
        let mut buf = *b"";
        super::make_ascii_capitalize(&mut buf);
        assert_eq!(buf, *b"");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn to_ascii_capitalize_empty() {
        assert_eq!(super::to_ascii_capitalize(""), b"");
    }
}
