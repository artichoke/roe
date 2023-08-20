use core::fmt;
use core::iter::FusedIterator;

use bstr::ByteSlice;

#[derive(Clone)]
#[must_use = "Uppercase is a Iterator and must be used"]
pub struct Uppercase<'a> {
    slice: &'a [u8],
}

impl<'a> fmt::Debug for Uppercase<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Uppercase")
            .field("slice", &self.slice.as_bstr())
            .finish()
    }
}

impl<'a> From<&'a [u8]> for Uppercase<'a> {
    fn from(slice: &'a [u8]) -> Self {
        Self::with_slice(slice)
    }
}

impl<'a> Uppercase<'a> {
    pub const fn with_slice(slice: &'a [u8]) -> Self {
        Self { slice }
    }
}

impl<'a> Iterator for Uppercase<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let (&byte, remainder) = self.slice.split_first()?;
        self.slice = remainder;
        Some(byte.to_ascii_uppercase())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.slice.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.slice.len()
    }
}

impl<'a> DoubleEndedIterator for Uppercase<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (&byte, remainder) = self.slice.split_last()?;
        self.slice = remainder;
        Some(byte.to_ascii_uppercase())
    }
}

impl<'a> ExactSizeIterator for Uppercase<'a> {}

impl<'a> FusedIterator for Uppercase<'a> {}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use bstr::ByteSlice;

    use super::Uppercase;

    #[test]
    fn empty() {
        let iter = Uppercase::from(&b""[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());
    }

    #[test]
    fn ascii() {
        let iter = Uppercase::from(&b"abc"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"ABC".as_bstr());

        let iter = Uppercase::from(&b"aBC"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"ABC".as_bstr());

        let iter = Uppercase::from(&b"ABC"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"ABC".as_bstr());

        let iter = Uppercase::from(&b"aBC, 123, ABC, baby you and me girl"[..]);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            b"ABC, 123, ABC, BABY YOU AND ME GIRL".as_bstr()
        );
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn utf8() {
        let s = "ß".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "ß".as_bytes().as_bstr());

        let s = "Αύριο".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "Αύριο".as_bytes().as_bstr()
        );

        let s = "Έτος".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "Έτος".as_bytes().as_bstr()
        );

        // two-byte characters
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L198-L200
        let s = "𐐜 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐙𐐊𐐡𐐝𐐓/𐐝𐐇𐐗𐐊𐐤𐐔 𐐒𐐋𐐗 𐐒𐐌 𐐜 𐐡𐐀𐐖𐐇𐐤𐐓𐐝 𐐱𐑂 𐑄 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐏𐐆𐐅𐐤𐐆𐐚𐐊𐐡𐐝𐐆𐐓𐐆".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "𐐜 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐙𐐊𐐡𐐝𐐓/𐐝𐐇𐐗𐐊𐐤𐐔 𐐒𐐋𐐗 𐐒𐐌 𐐜 𐐡𐐀𐐖𐐇𐐤𐐓𐐝 𐐱𐑂 𐑄 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐏𐐆𐐅𐐤𐐆𐐚𐐊𐐡𐐝𐐆𐐓𐐆"
                .as_bytes()
                .as_bstr()
        );

        // Change length when lowercased
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L226-L232
        let s = "zⱥⱦ".as_bytes();
        let uppercased = Uppercase::from(s).collect::<Vec<_>>();
        assert_eq!(uppercased.as_bstr(), "Zⱥⱦ".as_bytes().as_bstr());
        assert_eq!(s.len(), uppercased.len());
    }

    #[test]
    fn invalid_utf8() {
        let iter = Uppercase::from(&b"\xFF\xFE"[..]);
        assert_eq!(iter.collect::<Vec<u8>>().as_bstr(), b"\xFF\xFE".as_bstr());

        let iter = Uppercase::from(&b"ABC\xFF\xFEXYZ"[..]);
        assert_eq!(
            iter.collect::<Vec<u8>>().as_bstr(),
            b"ABC\xFF\xFEXYZ".as_bstr()
        );

        let iter = Uppercase::from(&b"abc\xFF\xFEXYZ"[..]);
        assert_eq!(
            iter.collect::<Vec<u8>>().as_bstr(),
            b"ABC\xFF\xFEXYZ".as_bstr()
        );

        // The bytes \xF0\x9F\x87 could lead to a valid UTF-8 sequence, but 3 of
        // them on their own are invalid. Only one replacement codepoint is
        // substituted, which demonstrates the "substitution of maximal
        // subparts" strategy.
        //
        // See: https://docs.rs/bstr/1.*/bstr/#handling-of-invalid-utf-8
        let iter = Uppercase::from(&b"aB\xF0\x9F\x87Yz"[..]);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            b"AB\xF0\x9F\x87YZ".as_bstr()
        );
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn unicode_replacement_character() {
        let s = "�".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "�".as_bytes().as_bstr());
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn dz_titlecase() {
        let s = "ǅ".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "ǅ".as_bytes().as_bstr());
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn latin_capital_i_with_dot_above() {
        let s = "İ".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "İ".as_bytes().as_bstr());
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn case_map_to_two_chars() {
        let s = "İ".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "İ".as_bytes().as_bstr());
    }

    #[test]
    fn size_hint() {
        assert_eq!(Uppercase::with_slice(b"").size_hint(), (0, Some(0)));
        assert_eq!(Uppercase::with_slice(b"abc, xyz").size_hint(), (8, Some(8)));
        assert_eq!(
            Uppercase::with_slice(b"abc, \xFF\xFE, xyz").size_hint(),
            (12, Some(12))
        );
        assert_eq!(
            Uppercase::with_slice("�".as_bytes()).size_hint(),
            (3, Some(3))
        );
        assert_eq!(
            Uppercase::with_slice("Έτος".as_bytes()).size_hint(),
            (8, Some(8))
        );
        assert_eq!(
            Uppercase::with_slice("ZȺȾ".as_bytes()).size_hint(),
            (5, Some(5))
        );

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Uppercase::with_slice(&utf8_with_invalid_bytes).size_hint(),
            (10, Some(10))
        );
    }

    #[test]
    fn count() {
        assert_eq!(Uppercase::with_slice(b"").count(), 0);
        assert_eq!(Uppercase::with_slice(b"abc, xyz").count(), 8);
        assert_eq!(Uppercase::with_slice(b"abc, \xFF\xFE, xyz").count(), 12);
        assert_eq!(Uppercase::with_slice("�".as_bytes()).count(), 3);
        assert_eq!(Uppercase::with_slice("Έτος".as_bytes()).count(), 8);
        assert_eq!(Uppercase::with_slice("ZȺȾ".as_bytes()).count(), 5);

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(Uppercase::with_slice(&utf8_with_invalid_bytes).count(), 10);
    }

    #[test]
    fn size_hint_covers_count() {
        let iter = Uppercase::with_slice(b"");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Uppercase::with_slice(b"abc, xyz");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Uppercase::with_slice(b"abc, \xFF\xFE, xyz");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Uppercase::with_slice("�".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Uppercase::with_slice("Έτος".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Uppercase::with_slice("ZȺȾ".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        let iter = Uppercase::with_slice(&utf8_with_invalid_bytes);
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());
    }
}
