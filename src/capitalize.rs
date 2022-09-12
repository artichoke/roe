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

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use bstr::ByteSlice;

    use super::Captialize;

    #[test]
    fn capitalize_utf8_string_empty() {
        let iter = Captialize::from(&b""[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());
    }

    #[test]
    fn capitalize_utf8_string_ascii() {
        let iter = Captialize::from(&b"abc"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"abc".as_bstr());

        let iter = Captialize::from(&b"aBC"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"abc".as_bstr());

        let iter = Captialize::from(&b"ABC"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"abc".as_bstr());

        let iter = Captialize::from(&b"aBC, 123, ABC, baby you and me girl"[..]);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            b"abc, 123, abc, baby you and me girl".as_bstr()
        );
    }

    #[test]
    fn capitalize_utf8_string_utf8() {
        let s = "ß".as_bytes();
        let iter = Captialize::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "ß".as_bytes().as_bstr());

        let s = "Αύριο".as_bytes();
        let iter = Captialize::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "αύριο".as_bytes().as_bstr()
        );

        let s = "Έτος".as_bytes();
        let iter = Captialize::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "έτος".as_bytes().as_bstr()
        );

        // two-byte characters
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L198-L200
        let s = "𐐜 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐙𐐊𐐡𐐝𐐓/𐐝𐐇𐐗𐐊𐐤𐐔 𐐒𐐋𐐗 𐐒𐐌 𐐜 𐐡𐐀𐐖𐐇𐐤𐐓𐐝 𐐱𐑂 𐑄 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐏𐐆𐐅𐐤𐐆𐐚𐐊𐐡𐐝𐐆𐐓𐐆".as_bytes();
        let iter = Captialize::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "𐑄 𐐼𐐯𐑅𐐨𐑉𐐯𐐻 𐑁𐐲𐑉𐑅𐐻/𐑅𐐯𐐿𐐲𐑌𐐼 𐐺𐐳𐐿 𐐺𐐴 𐑄 𐑉𐐨𐐾𐐯𐑌𐐻𐑅 𐐱𐑂 𐑄 𐐼𐐯𐑅𐐨𐑉𐐯𐐻 𐐷𐐮𐐭𐑌𐐮𐑂𐐲𐑉𐑅𐐮𐐻𐐮"
                .as_bytes()
                .as_bstr()
        );

        // Change length when capitalized
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L226-L232
        let s = "ZȺȾ".as_bytes();
        let iter = Captialize::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "zⱥⱦ".as_bytes().as_bstr()
        );
    }

    #[test]
    fn capitalize_utf8_string_invalid_utf8() {
        let iter = Captialize::from(&b"\xFF\xFE"[..]);
        assert_eq!(iter.collect::<Vec<u8>>().as_bstr(), b"\xFF\xFE".as_bstr());

        let iter = Captialize::from(&b"ABC\xFF\xFEXYZ"[..]);
        assert_eq!(
            iter.collect::<Vec<u8>>().as_bstr(),
            b"abc\xFF\xFExyz".as_bstr()
        );

        let iter = Captialize::from(&b"abc\xFF\xFEXYZ"[..]);
        assert_eq!(
            iter.collect::<Vec<u8>>().as_bstr(),
            b"abc\xFF\xFExyz".as_bstr()
        );
    }

    #[test]
    fn capitalize_utf8_string_unicode_replacement_character() {
        let s = "�".as_bytes();
        let iter = Captialize::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "�".as_bytes().as_bstr());
    }
}
