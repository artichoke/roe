use core::char::ToUppercase;
use core::fmt;
use core::iter::FusedIterator;
use core::ops::Range;

use bstr::ByteSlice;

#[derive(Clone)]
#[must_use = "Uppercase is a Iterator and must be used"]
pub struct Uppercase<'a> {
    slice: &'a [u8],
    next_bytes: [u8; 4],
    next_range: Range<usize>,
    uppercase: Option<ToUppercase>,
}

impl<'a> fmt::Debug for Uppercase<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Uppercase")
            .field("slice", &self.slice.as_bstr())
            .field("next_bytes", &self.next_bytes)
            .field("next_range", &self.next_range)
            .field("uppercase", &self.uppercase)
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
        Self {
            slice,
            next_bytes: [0; 4],
            next_range: 0..0,
            uppercase: None,
        }
    }
}

impl<'a> Iterator for Uppercase<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.next_range.next() {
            debug_assert!(self.next_bytes.get(idx).is_some());

            return Some(self.next_bytes[idx]);
        }

        if let Some(ch) = self.uppercase.as_mut().and_then(Iterator::next) {
            let enc = ch.encode_utf8(&mut self.next_bytes);

            self.next_range = 1..enc.len();
            debug_assert!(self.next_bytes.get(self.next_range.clone()).is_some());

            return Some(self.next_bytes[0]);
        }

        self.uppercase = None;

        match bstr::decode_utf8(self.slice) {
            (_, 0) => None,
            (Some(ch), size) => {
                self.slice = &self.slice[size..];
                let mut uppercase = ch.to_uppercase();
                let ch = uppercase
                    .next()
                    .expect("ToUppercase yields at least one char");
                let enc = ch.encode_utf8(&mut self.next_bytes);

                self.next_range = 1..enc.len();
                debug_assert!(self.next_bytes.get(self.next_range.clone()).is_some());

                self.uppercase = Some(uppercase);
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
        const TO_UPPER_EXPAND: usize = 3;
        const UTF_8_CHAR_MAX_BYTES: usize = 4;
        if self.slice.is_empty() {
            (0, Some(0))
        } else if self.slice.is_ascii() {
            let len = self.slice.len();
            (len, Some(len))
        } else {
            let len = self.slice.len();
            (len, Some(len * TO_UPPER_EXPAND * UTF_8_CHAR_MAX_BYTES))
        }
    }

    fn count(self) -> usize {
        if self.slice.is_empty() {
            0
        } else if self.slice.is_ascii() {
            self.slice.len()
        } else {
            self.fold(0, |acc, _| acc + 1)
        }
    }
}

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

    #[test]
    fn utf8() {
        let s = "ß".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "SS".as_bytes().as_bstr()
        );

        let s = "Αύριο".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "ΑΎΡΙΟ".as_bytes().as_bstr()
        );

        let s = "Έτος".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "ΈΤΟΣ".as_bytes().as_bstr()
        );

        // two-byte characters
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L198-L200
        let s = "𐑄 𐐼𐐯𐑅𐐨𐑉𐐯𐐻 𐑁𐐲𐑉𐑅𐐻/𐑅𐐯𐐿𐐲𐑌𐐼 𐐺𐐳𐐿 𐐺𐐴 𐑄 𐑉𐐨𐐾𐐯𐑌𐐻𐑅 𐐱𐑂 𐑄 𐐼𐐯𐑅𐐨𐑉𐐯𐐻 𐐷𐐮𐐭𐑌𐐮𐑂𐐲𐑉𐑅𐐮𐐻𐐮".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "𐐜 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐙𐐊𐐡𐐝𐐓/𐐝𐐇𐐗𐐊𐐤𐐔 𐐒𐐋𐐗 𐐒𐐌 𐐜 𐐡𐐀𐐖𐐇𐐤𐐓𐐝 𐐉𐐚 𐐜 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐏𐐆𐐅𐐤𐐆𐐚𐐊𐐡𐐝𐐆𐐓𐐆"
                .as_bytes()
                .as_bstr()
        );

        // Change length when uppercased
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/894882e7/blns.txt#L226-L232
        let s = "zⱥⱦ".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "ZȺȾ".as_bytes().as_bstr()
        );
    }

    #[test]
    fn invalid_utf8() {
        let iter = Uppercase::from(&b"\xFF\xFE"[..]);
        assert_eq!(iter.collect::<Vec<u8>>().as_bstr(), b"\xFF\xFE".as_bstr());

        let iter = Uppercase::from(&b"abc\xFF\xFExyz"[..]);
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
        // See: https://docs.rs/bstr/0.2.*/bstr/#handling-of-invalid-utf-8
        let iter = Uppercase::from(&b"aB\xF0\x9F\x87Yz"[..]);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            b"AB\xF0\x9F\x87YZ".as_bstr()
        );
    }

    #[test]
    fn unicode_replacement_character() {
        let s = "�".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "�".as_bytes().as_bstr());
    }

    #[test]
    fn dz_titlecase() {
        let s = "ǅ".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), "Ǆ".as_bytes().as_bstr());
    }

    #[test]
    fn latin_small_i_with_dot_above() {
        let s = "i̇".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            [73_u8, 204, 135].as_bstr()
        );
    }

    #[test]
    fn case_map_to_two_chars() {
        let s = "և".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "ԵՒ".as_bytes().as_bstr()
        );

        let s = "ẙ".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "Y\u{30a}".as_bytes().as_bstr()
        );

        let s = "ᾂ".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "ἊΙ".as_bytes().as_bstr()
        );

        let s = "ﬗ".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "ՄԽ".as_bytes().as_bstr()
        );
    }

    #[test]
    fn case_map_to_three_chars() {
        let s = "ﬃ".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(iter.collect::<Vec<_>>().as_bstr(), b"FFI".as_bstr());

        let s = "ὖ".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "Υ\u{313}\u{342}".as_bytes().as_bstr()
        );

        let s = "ῷ".as_bytes();
        let iter = Uppercase::from(s);
        assert_eq!(
            iter.collect::<Vec<_>>().as_bstr(),
            "Ω\u{342}Ι".as_bytes().as_bstr()
        );
    }

    #[test]
    fn size_hint() {
        assert_eq!(Uppercase::with_slice(b"").size_hint(), (0, Some(0)));
        assert_eq!(Uppercase::with_slice(b"abc, xyz").size_hint(), (8, Some(8)));
        assert_eq!(
            Uppercase::with_slice(b"abc, \xFF\xFE, xyz").size_hint(),
            (12, Some(144))
        );
        assert_eq!(
            Uppercase::with_slice("�".as_bytes()).size_hint(),
            (3, Some(36))
        );
        assert_eq!(
            Uppercase::with_slice("Έτος".as_bytes()).size_hint(),
            (8, Some(96))
        );
        assert_eq!(
            Uppercase::with_slice("ZȺȾ".as_bytes()).size_hint(),
            (5, Some(60))
        );

        let mut utf8_with_invalid_bytes = b"\xFF\xFE".to_vec();
        utf8_with_invalid_bytes.extend_from_slice("Έτος".as_bytes());
        assert_eq!(
            Uppercase::with_slice(&utf8_with_invalid_bytes).size_hint(),
            (10, Some(120))
        );
    }

    #[test]
    fn count() {
        assert_eq!(Uppercase::with_slice(b"").count(), 0);
        assert_eq!(Uppercase::with_slice(b"abc, xyz").count(), 8);
        assert_eq!(Uppercase::with_slice(b"abc, \xFF\xFE, xyz").count(), 12);
        assert_eq!(Uppercase::with_slice("�".as_bytes()).count(), 3);
        assert_eq!(Uppercase::with_slice("Έτος".as_bytes()).count(), 8);
        assert_eq!(Uppercase::with_slice("zⱥⱦ".as_bytes()).count(), 5);

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
