use core::fmt;
use core::iter::FusedIterator;
use core::ops::Range;

use bstr::ByteSlice;

use crate::unicode::mapping::{Mode, lookup};
use crate::unicode::std_case_mapping_iter::CaseMappingIter;

#[derive(Clone)]
#[must_use = "Swapcase is a Iterator and must be used"]
pub struct Swapcase<'a> {
    slice: &'a [u8],
    next_bytes: [u8; 4],
    next_range: Range<usize>,
    swapcase: Option<CaseMappingIter>,
    mode: Mode,
}

impl fmt::Debug for Swapcase<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Swapcase")
            .field("slice", &self.slice.as_bstr())
            .field("next_bytes", &self.next_bytes)
            .field("next_range", &self.next_range)
            .field("swapcase", &self.swapcase)
            .field("mode", &self.mode)
            .finish()
    }
}

impl<'a> From<&'a [u8]> for Swapcase<'a> {
    fn from(slice: &'a [u8]) -> Self {
        Self::with_slice(slice)
    }
}

impl<'a> Swapcase<'a> {
    pub const fn with_slice(slice: &'a [u8]) -> Self {
        Self::with_mode(slice, Mode::Swap)
    }

    pub const fn with_mode(slice: &'a [u8], mode: Mode) -> Self {
        Self {
            slice,
            next_bytes: [0; 4],
            next_range: 0..0,
            swapcase: None,
            mode,
        }
    }

    fn buffered_len(&self) -> usize {
        let mapped = self
            .swapcase
            .clone()
            .map_or(0, |iter| iter.map(char::len_utf8).sum());
        self.next_range.len() + mapped
    }
}

impl Iterator for Swapcase<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.next_range.next() {
            debug_assert!(self.next_bytes.get(idx).is_some());

            return Some(self.next_bytes[idx]);
        }

        if let Some(ch) = self.swapcase.as_mut().and_then(Iterator::next) {
            let enc = ch.encode_utf8(&mut self.next_bytes);

            self.next_range = 1..enc.len();
            debug_assert!(self.next_bytes.get(self.next_range.clone()).is_some());

            return Some(self.next_bytes[0]);
        }

        self.swapcase = None;

        match bstr::decode_utf8(self.slice) {
            (_, 0) => None,
            (Some(ch), size) => {
                self.slice = &self.slice[size..];
                let mut swapcase = lookup(ch, self.mode);
                let ch = swapcase
                    .next()
                    .expect("case mapping yields at least one char");
                let enc = ch.encode_utf8(&mut self.next_bytes);

                self.next_range = 1..enc.len();
                debug_assert!(self.next_bytes.get(self.next_range.clone()).is_some());

                self.swapcase = Some(swapcase);
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
            // byte length is not a lower bound: e.g. dotless i maps to ASCII I.
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

impl FusedIterator for Swapcase<'_> {}
