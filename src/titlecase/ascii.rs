use core::fmt;
use core::iter::FusedIterator;

use bstr::ByteSlice;

#[derive(Clone)]
#[must_use = "Titlecase is a Iterator and must be used"]
pub struct Titlecase<'a> {
    slice: &'a [u8],
    first: bool,
}

impl<'a> fmt::Debug for Titlecase<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Titlecase")
            .field("slice", &self.slice.as_bstr())
            .finish()
    }
}

impl<'a> From<&'a [u8]> for Titlecase<'a> {
    fn from(slice: &'a [u8]) -> Self {
        Self::with_slice(slice)
    }
}

impl<'a> Titlecase<'a> {
    pub const fn with_slice(slice: &'a [u8]) -> Self {
        Self { slice, first: true }
    }
}

impl<'a> Iterator for Titlecase<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let (&byte, remainder) = self.slice.split_first()?;
        self.slice = remainder;
        if self.first {
            self.first = false;
            Some(byte.to_ascii_uppercase())
        } else {
            Some(byte.to_ascii_lowercase())
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

impl<'a> DoubleEndedIterator for Titlecase<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (&byte, remainder) = self.slice.split_last()?;
        self.slice = remainder;
        if remainder.is_empty() {
            Some(byte.to_ascii_uppercase())
        } else {
            Some(byte.to_ascii_lowercase())
        }
    }
}

impl<'a> ExactSizeIterator for Titlecase<'a> {}

impl<'a> FusedIterator for Titlecase<'a> {}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use bstr::ByteSlice;

    use super::Titlecase;

    #[test]
    fn empty() {
        let iter = Titlecase::from(&b""[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());
    }

    #[test]
    fn ascii() {
        let iter = Titlecase::from(&b"abc"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"Abc".as_bstr());

        let iter = Titlecase::from(&b"aBC"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"Abc".as_bstr());

        let iter = Titlecase::from(&b"ABC"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"Abc".as_bstr());

        let iter = Titlecase::from(&b"aBC, 123, ABC, baby you and me girl"[..]);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            b"Abc, 123, abc, baby you and me girl".as_bstr()
        );
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn utf8() {
        let s = "ß".as_bytes();
        let iter = Titlecase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "ß".as_bytes().as_bstr());

        let s = "Αύριο".as_bytes();
        let iter = Titlecase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "Αύριο".as_bytes().as_bstr()
        );

        let s = "Έτος".as_bytes();
        let iter = Titlecase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "Έτος".as_bytes().as_bstr()
        );

        // two-byte characters
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L198-L200
        let s = "𐐜 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐙𐐊𐐡𐐝𐐓/𐐝𐐇𐐗𐐊𐐤𐐔 𐐒𐐋𐐗 𐐒𐐌 𐐜 𐐡𐐀𐐖𐐇𐐤𐐓𐐝 𐐱𐑂 𐑄 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐏𐐆𐐅𐐤𐐆𐐚𐐊𐐡𐐝𐐆𐐓𐐆".as_bytes();
        let iter = Titlecase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "𐐜 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐙𐐊𐐡𐐝𐐓/𐐝𐐇𐐗𐐊𐐤𐐔 𐐒𐐋𐐗 𐐒𐐌 𐐜 𐐡𐐀𐐖𐐇𐐤𐐓𐐝 𐐱𐑂 𐑄 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐏𐐆𐐅𐐤𐐆𐐚𐐊𐐡𐐝𐐆𐐓𐐆"
                .as_bytes()
                .as_bstr()
        );

        // Change length when titlecased
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L226-L232
        let s = "ⱥȾȾZ".as_bytes();
        let titlecased = Titlecase::from(s).collect::<Vec<_>>();
        assert_eq!(titlecased.as_bstr(), "ⱥȾȾz".as_bytes().as_bstr());
        assert_eq!(s.len(), titlecased.len());
    }

    #[test]
    fn invalid_utf8() {
        let iter = Titlecase::from(&b"\xFF\xFE"[..]);
        assert_eq!(iter.collect::<Vec<u8>>().as_bstr(), b"\xFF\xFE".as_bstr());

        let iter = Titlecase::from(&b"ABC\xFF\xFEXYZ"[..]);
        assert_eq!(
            iter.collect::<Vec<u8>>().as_bstr(),
            b"Abc\xFF\xFExyz".as_bstr()
        );

        let iter = Titlecase::from(&b"abc\xFF\xFEXYZ"[..]);
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
        let iter = Titlecase::from(&b"aB\xF0\x9F\x87Yz"[..]);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            b"Ab\xF0\x9F\x87yz".as_bstr()
        );
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn unicode_replacement_character() {
        let s = "�".as_bytes();
        let iter = Titlecase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "�".as_bytes().as_bstr());
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn dz_titlecase() {
        let s = "ǅ".as_bytes();
        let iter = Titlecase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "ǅ".as_bytes().as_bstr());
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn latin_capital_i_with_dot_above() {
        let s = "İ".as_bytes();
        let iter = Titlecase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "İ".as_bytes().as_bstr());
    }

    // ignore unicode for ASCII iterator
    #[test]
    fn case_map_to_two_chars() {
        let s = "İ".as_bytes();
        let iter = Titlecase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "İ".as_bytes().as_bstr());
    }

    #[test]
    fn size_hint() {
        assert_eq!(Titlecase::with_slice(b"").size_hint(), (0, Some(0)));
        assert_eq!(Titlecase::with_slice(b"abc, xyz").size_hint(), (8, Some(8)));
        assert_eq!(
            Titlecase::with_slice(b"abc, \xFF\xFE, xyz").size_hint(),
            (12, Some(12))
        );
        assert_eq!(
            Titlecase::with_slice("�".as_bytes()).size_hint(),
            (3, Some(3))
        );
        assert_eq!(
            Titlecase::with_slice("Έτος".as_bytes()).size_hint(),
            (8, Some(8))
        );
        assert_eq!(
            Titlecase::with_slice("ZȺȾ".as_bytes()).size_hint(),
            (5, Some(5))
        );

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Titlecase::with_slice(&utf8_with_invalid_bytes).size_hint(),
            (10, Some(10))
        );
    }

    #[test]
    fn count() {
        assert_eq!(Titlecase::with_slice(b"").count(), 0);
        assert_eq!(Titlecase::with_slice(b"abc, xyz").count(), 8);
        assert_eq!(Titlecase::with_slice(b"abc, \xFF\xFE, xyz").count(), 12);
        assert_eq!(Titlecase::with_slice("�".as_bytes()).count(), 3);
        assert_eq!(Titlecase::with_slice("Έτος".as_bytes()).count(), 8);
        assert_eq!(Titlecase::with_slice("ZȺȾ".as_bytes()).count(), 5);

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(Titlecase::with_slice(&utf8_with_invalid_bytes).count(), 10);
    }

    #[test]
    fn size_hint_covers_count() {
        let iter = Titlecase::with_slice(b"");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Titlecase::with_slice(b"abc, xyz");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Titlecase::with_slice(b"abc, \xFF\xFE, xyz");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Titlecase::with_slice("�".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Titlecase::with_slice("Έτος".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Titlecase::with_slice("ZȺȾ".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        let iter = Titlecase::with_slice(&utf8_with_invalid_bytes);
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());
    }

    #[test]
    fn double_ended_iterator() {
        let mut iter = Titlecase::with_slice(b"abc");
        assert_eq!(iter.next_back(), Some(b'c'));
        assert_eq!(iter.next_back(), Some(b'b'));
        assert_eq!(iter.next_back(), Some(b'A'));

        let mut iter = Titlecase::with_slice(b"abc");
        assert_eq!(iter.next(), Some(b'A'));
        assert_eq!(iter.next_back(), Some(b'c'));
        assert_eq!(iter.next_back(), Some(b'B')); // FIXME: Should be 'b'
    }
}
