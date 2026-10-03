use core::fmt;
use core::iter::FusedIterator;
use core::ops::Range;

use bstr::ByteSlice;

use crate::unicode::mapping::{Mode, lookup};
use crate::unicode::std_case_mapping_iter::CaseMappingIter;

#[derive(Clone)]
#[must_use = "Lowercase is a Iterator and must be used"]
pub struct Lowercase<'a> {
    slice: &'a [u8],
    next_bytes: [u8; 4],
    next_range: Range<usize>,
    lowercase: Option<CaseMappingIter>,
    mode: Mode,
}

impl fmt::Debug for Lowercase<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lowercase")
            .field("slice", &self.slice.as_bstr())
            .field("next_bytes", &self.next_bytes)
            .field("next_range", &self.next_range)
            .field("lowercase", &self.lowercase)
            .field("mode", &self.mode)
            .finish()
    }
}

impl<'a> From<&'a [u8]> for Lowercase<'a> {
    fn from(slice: &'a [u8]) -> Self {
        Self::with_slice(slice)
    }
}

impl<'a> Lowercase<'a> {
    pub const fn with_slice(slice: &'a [u8]) -> Self {
        Self::with_mode(slice, Mode::Lower)
    }

    pub const fn with_mode(slice: &'a [u8], mode: Mode) -> Self {
        Self {
            slice,
            next_bytes: [0; 4],
            next_range: 0..0,
            lowercase: None,
            mode,
        }
    }

    fn buffered_len(&self) -> usize {
        let mapped = self
            .lowercase
            .clone()
            .map_or(0, |iter| iter.map(char::len_utf8).sum());
        self.next_range.len() + mapped
    }
}

impl Iterator for Lowercase<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.next_range.next() {
            debug_assert!(self.next_bytes.get(idx).is_some());

            return Some(self.next_bytes[idx]);
        }

        if let Some(ch) = self.lowercase.as_mut().and_then(Iterator::next) {
            let enc = ch.encode_utf8(&mut self.next_bytes);

            self.next_range = 1..enc.len();
            debug_assert!(self.next_bytes.get(self.next_range.clone()).is_some());

            return Some(self.next_bytes[0]);
        }

        self.lowercase = None;

        match bstr::decode_utf8(self.slice) {
            (_, 0) => None,
            (Some(ch), size) => {
                self.slice = &self.slice[size..];
                let mut lowercase = lookup(ch, self.mode);
                let ch = lowercase
                    .next()
                    .expect("case mapping yields at least one char");
                let enc = ch.encode_utf8(&mut self.next_bytes);

                self.next_range = 1..enc.len();
                debug_assert!(self.next_bytes.get(self.next_range.clone()).is_some());

                self.lowercase = Some(lowercase);
                Some(self.next_bytes[0])
            }
            (None, size) => {
                let (bytes, remainder) = self.slice.split_at(size);
                self.slice = remainder;

                // Invalid byte sequences are at most three bytes.
                debug_assert!(self.next_bytes.get(..bytes.len()).is_some());

                self.next_bytes[..bytes.len()].copy_from_slice(bytes);
                self.next_range = 1..bytes.len();
                Some(self.next_bytes[0])
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        const CASE_MAPPING_MAX_BYTES: usize = 3 * 4;
        let buffered = self.buffered_len();
        let len = self.slice.len();
        if self.slice.is_ascii() && !self.mode.is_turkic() {
            let len = buffered + len;
            (len, Some(len))
        } else {
            // A decoded character consumes at most four input bytes and yields
            // at least one output byte. Invalid UTF-8 is passed through. Input
            // byte length is not a lower bound: e.g. the Kelvin sign maps to k.
            let min = buffered.saturating_add(len.div_ceil(4));
            let max = len
                .checked_mul(CASE_MAPPING_MAX_BYTES)
                .and_then(|len| len.checked_add(buffered));
            (min, max)
        }
    }

    fn count(self) -> usize {
        if self.slice.is_ascii() && !self.mode.is_turkic() {
            self.buffered_len() + self.slice.len()
        } else {
            self.fold(0, |acc, _| acc + 1)
        }
    }
}

impl FusedIterator for Lowercase<'_> {}

#[cfg(test)]
mod tests {
    use alloc::{format, vec::Vec};
    use bstr::ByteSlice;

    use crate::unicode::mapping::{Mode, lookup};
    use core::char;

    use super::Lowercase;

    #[test]
    fn empty() {
        let iter = Lowercase::from(&b""[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"".as_bstr());
    }

    #[test]
    fn ascii() {
        let iter = Lowercase::from(&b"abc"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"abc".as_bstr());

        let iter = Lowercase::from(&b"aBC"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"abc".as_bstr());

        let iter = Lowercase::from(&b"ABC"[..]);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"abc".as_bstr());

        let iter = Lowercase::from(&b"aBC, 123, ABC, baby you and me girl"[..]);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            b"abc, 123, abc, baby you and me girl".as_bstr()
        );
    }

    #[test]
    fn utf8() {
        let s = "ß".as_bytes();
        let iter = Lowercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "ß".as_bytes().as_bstr());

        let s = "Αύριο".as_bytes();
        let iter = Lowercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "αύριο".as_bytes().as_bstr()
        );

        let s = "Έτος".as_bytes();
        let iter = Lowercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "έτος".as_bytes().as_bstr()
        );

        // two-byte characters
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L198-L200
        let s = "𐐜 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐙𐐊𐐡𐐝𐐓/𐐝𐐇𐐗𐐊𐐤𐐔 𐐒𐐋𐐗 𐐒𐐌 𐐜 𐐡𐐀𐐖𐐇𐐤𐐓𐐝 𐐱𐑂 𐑄 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐏𐐆𐐅𐐤𐐆𐐚𐐊𐐡𐐝𐐆𐐓𐐆".as_bytes();
        let iter = Lowercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "𐑄 𐐼𐐯𐑅𐐨𐑉𐐯𐐻 𐑁𐐲𐑉𐑅𐐻/𐑅𐐯𐐿𐐲𐑌𐐼 𐐺𐐳𐐿 𐐺𐐴 𐑄 𐑉𐐨𐐾𐐯𐑌𐐻𐑅 𐐱𐑂 𐑄 𐐼𐐯𐑅𐐨𐑉𐐯𐐻 𐐷𐐮𐐭𐑌𐐮𐑂𐐲𐑉𐑅𐐮𐐻𐐮"
                .as_bytes()
                .as_bstr()
        );

        // Change length when lowercased
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L226-L232
        let s = "ZȺȾ".as_bytes();
        let lowercased = Lowercase::from(s).collect::<Vec<_>>();
        assert_eq!(lowercased.as_bstr(), "zⱥⱦ".as_bytes().as_bstr());
        assert_ne!(s.len(), lowercased.len());
    }

    #[test]
    fn invalid_utf8() {
        let iter = Lowercase::from(&b"\xFF\xFE"[..]);
        assert_eq!(iter.collect::<Vec<u8>>().as_bstr(), b"\xFF\xFE".as_bstr());

        let iter = Lowercase::from(&b"ABC\xFF\xFEXYZ"[..]);
        assert_eq!(
            iter.collect::<Vec<u8>>().as_bstr(),
            b"abc\xFF\xFExyz".as_bstr()
        );

        let iter = Lowercase::from(&b"abc\xFF\xFEXYZ"[..]);
        assert_eq!(
            iter.collect::<Vec<u8>>().as_bstr(),
            b"abc\xFF\xFExyz".as_bstr()
        );

        // The bytes \xF0\x9F\x87 could lead to a valid UTF-8 sequence, but 3 of
        // them on their own are invalid. Only one replacement codepoint is
        // substituted, which demonstrates the "substitution of maximal
        // subparts" strategy.
        //
        // See: https://docs.rs/bstr/1.*/bstr/#handling-of-invalid-utf-8
        let iter = Lowercase::from(&b"aB\xF0\x9F\x87Yz"[..]);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            b"ab\xF0\x9F\x87yz".as_bstr()
        );
    }

    #[test]
    fn unicode_replacement_character() {
        let s = "�".as_bytes();
        let iter = Lowercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "�".as_bytes().as_bstr());
    }

    #[test]
    fn dz_to_lowercase() {
        let s = "ǅ".as_bytes();
        let iter = Lowercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "ǆ".as_bytes().as_bstr());

        let s = "Ǆ".as_bytes();
        let iter = Lowercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "ǆ".as_bytes().as_bstr());

        let s = "ǆ".as_bytes();
        let iter = Lowercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "ǆ".as_bytes().as_bstr());
    }

    #[test]
    fn latin_capital_i_with_dot_above() {
        let s = "İ".as_bytes();
        let iter = Lowercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            [105_u8, 204, 135].as_bstr()
        );
    }

    #[test]
    fn case_map_to_two_chars() {
        let s = "İ".as_bytes();
        let iter = Lowercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "i\u{307}".as_bytes().as_bstr()
        );
    }

    #[test]
    fn case_map_to_three_chars() {
        // there are no such characters
        for ch in '\0'..char::MAX {
            assert!(
                lookup(ch, Mode::Lower).count() < 3,
                "Expected no characters that downcase to three or more characters, found: '{}', which expands to: {:?}",
                ch,
                lookup(ch, Mode::Lower).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn size_hint() {
        assert_eq!(Lowercase::with_slice(b"").size_hint(), (0, Some(0)));
        assert_eq!(Lowercase::with_slice(b"abc, xyz").size_hint(), (8, Some(8)));
        assert_eq!(
            Lowercase::with_slice(b"abc, \xFF\xFE, xyz").size_hint(),
            (3, Some(144))
        );
        assert_eq!(
            Lowercase::with_slice("�".as_bytes()).size_hint(),
            (1, Some(36))
        );
        assert_eq!(
            Lowercase::with_slice("Έτος".as_bytes()).size_hint(),
            (2, Some(96))
        );
        assert_eq!(
            Lowercase::with_slice("ZȺȾ".as_bytes()).size_hint(),
            (2, Some(60))
        );

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Lowercase::with_slice(&utf8_with_invalid_bytes).size_hint(),
            (3, Some(120))
        );
    }

    #[test]
    fn count() {
        assert_eq!(Lowercase::with_slice(b"").count(), 0);
        assert_eq!(Lowercase::with_slice(b"abc, xyz").count(), 8);
        assert_eq!(Lowercase::with_slice(b"abc, \xFF\xFE, xyz").count(), 12);
        assert_eq!(Lowercase::with_slice("�".as_bytes()).count(), 3);
        assert_eq!(Lowercase::with_slice("Έτος".as_bytes()).count(), 8);
        assert_eq!(Lowercase::with_slice("ZȺȾ".as_bytes()).count(), 7);

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(Lowercase::with_slice(&utf8_with_invalid_bytes).count(), 10);
    }

    #[test]
    fn size_hint_covers_count() {
        let iter = Lowercase::with_slice(b"");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Lowercase::with_slice(b"abc, xyz");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Lowercase::with_slice(b"abc, \xFF\xFE, xyz");
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Lowercase::with_slice("�".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Lowercase::with_slice("Έτος".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let iter = Lowercase::with_slice("ZȺȾ".as_bytes());
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        let iter = Lowercase::with_slice(&utf8_with_invalid_bytes);
        let (min, max) = iter.size_hint();
        let count = iter.count();
        assert!(min <= count);
        assert!(count <= max.unwrap());
    }

    #[test]
    fn test_fmt() {
        let s = "Αύριο".as_bytes();
        let iter = Lowercase::from(s);
        assert_eq!(
            format!("{iter:?}"),
            "Lowercase { slice: \"Αύριο\", next_bytes: [0, 0, 0, 0], next_range: 0..0, lowercase: None, mode: Lower }"
        );
    }
}
