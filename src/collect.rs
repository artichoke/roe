use alloc::borrow::Cow;
use alloc::collections::TryReserveError;
use alloc::vec::Vec;

/// Collect a mapping only after its output first differs from the input.
///
/// The matching prefix is copied directly; the iterator is never restarted.
/// Compare bytes rather than characters: expansions can share an initial byte
/// with the input, and the first difference need not be a UTF-8 boundary.
pub(crate) fn try_collect(
    slice: &[u8],
    mut iter: impl Iterator<Item = u8>,
) -> Result<Cow<'_, [u8]>, TryReserveError> {
    let mut matched = 0;
    for byte in iter.by_ref() {
        if slice.get(matched) == Some(&byte) {
            matched += 1;
            continue;
        }
        let mut bytes = Vec::new();
        bytes.try_reserve(slice.len().max(matched.saturating_add(1)))?;
        bytes.extend_from_slice(&slice[..matched]);
        bytes.push(byte);
        for byte in iter {
            if bytes.len() == bytes.capacity() {
                bytes.try_reserve(1)?;
            }
            bytes.push(byte);
        }
        return Ok(Cow::Owned(bytes));
    }
    if matched == slice.len() {
        return Ok(Cow::Borrowed(slice));
    }
    // A shorter output is also a change, even if every yielded byte matched.
    let mut bytes = Vec::new();
    bytes.try_reserve(matched)?;
    bytes.extend_from_slice(&slice[..matched]);
    Ok(Cow::Owned(bytes))
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;
    use alloc::vec::Vec;

    use crate::ruby::{CapitalizeMode as C, capitalize, try_to_capitalize};
    use crate::{LowercaseMode as L, SwapcaseMode as S, UppercaseMode as U};

    fn results(input: &[u8]) -> [Cow<'_, [u8]>; 17] {
        [
            crate::try_to_lowercase(input, L::Full).unwrap(),
            crate::try_to_lowercase(input, L::Ascii).unwrap(),
            crate::try_to_lowercase(input, L::Turkic).unwrap(),
            crate::try_to_lowercase(input, L::Lithuanian).unwrap(),
            crate::try_to_lowercase(input, L::Fold).unwrap(),
            crate::try_to_uppercase(input, U::Full).unwrap(),
            crate::try_to_uppercase(input, U::Ascii).unwrap(),
            crate::try_to_uppercase(input, U::Turkic).unwrap(),
            crate::try_to_uppercase(input, U::Lithuanian).unwrap(),
            try_to_capitalize(input, C::Full).unwrap(),
            try_to_capitalize(input, C::Ascii).unwrap(),
            try_to_capitalize(input, C::Turkic).unwrap(),
            try_to_capitalize(input, C::Lithuanian).unwrap(),
            crate::try_to_swapcase(input, S::Full).unwrap(),
            crate::try_to_swapcase(input, S::Ascii).unwrap(),
            crate::try_to_swapcase(input, S::Turkic).unwrap(),
            crate::try_to_swapcase(input, S::Lithuanian).unwrap(),
        ]
    }

    #[track_caller]
    #[allow(
        clippy::ptr_arg,
        reason = "The borrowed/owned variant is part of the contract under test."
    )]
    fn check_result(input: &[u8], actual: &Cow<'_, [u8]>, expected: &[u8]) {
        assert_eq!(actual.as_ref(), expected);
        match actual {
            Cow::Borrowed(bytes) => {
                assert_eq!(expected, input);
                assert_eq!(bytes.as_ptr(), input.as_ptr());
                assert_eq!(bytes.len(), input.len());
            }
            Cow::Owned(_) => assert_ne!(expected, input),
        }
    }

    fn check(input: &[u8]) {
        let expected = [
            crate::lowercase(input, L::Full).collect::<Vec<_>>(),
            crate::lowercase(input, L::Ascii).collect(),
            crate::lowercase(input, L::Turkic).collect(),
            crate::lowercase(input, L::Lithuanian).collect(),
            crate::lowercase(input, L::Fold).collect(),
            crate::uppercase(input, U::Full).collect(),
            crate::uppercase(input, U::Ascii).collect(),
            crate::uppercase(input, U::Turkic).collect(),
            crate::uppercase(input, U::Lithuanian).collect(),
            capitalize(input, C::Full).collect(),
            capitalize(input, C::Ascii).collect(),
            capitalize(input, C::Turkic).collect(),
            capitalize(input, C::Lithuanian).collect(),
            crate::swapcase(input, S::Full).collect(),
            crate::swapcase(input, S::Ascii).collect(),
            crate::swapcase(input, S::Turkic).collect(),
            crate::swapcase(input, S::Lithuanian).collect(),
        ];
        for (actual, expected) in results(input).into_iter().zip(expected) {
            check_result(input, &actual, &expected);
        }
    }

    #[test]
    fn representative_strings_and_malformed_boundaries() {
        let inputs: &[&[u8]] = &[
            b"",
            b"artichoke",
            b"ARTICHOKE",
            b"Artichoke",
            b"123!\0",
            "東京🦀".as_bytes(),
            "Straße Αύριο 東京".as_bytes(),
            "ßﬃİẞςᎠ".as_bytes(),
            "Ǆǅǆ".as_bytes(),
            "Iİiı".as_bytes(),
            "I\u{307}".as_bytes(),
            "ᲐᲑᲒ".as_bytes(),
            "ΐᾀ".as_bytes(),
            b"\xff",
            b"\xf0\x9f\x87",
            b"\xffabc",
            b"a\xffBc",
            b"abc\xff",
            b"\xff\xc3\x9fTEST",
            b"\xc3\x9f\xffTEST",
            b"\xc3\x9fTEST\xff",
            b"\xff\xef\xac\x83",
            b"\xef\xac\x83\xff",
            b"\xffiI\xfe",
            b"\xc4\xff\xb0i",
            b"\xed\xa0\x80ABC",
            b"\0\xff\xc3\x9fTEST",
        ];
        for input in inputs {
            check(input);
        }
        // Exercise late changes and prefixes copied from inside UTF-8 output.
        for prefix in [b"123!".as_slice(), "東京🦀".as_bytes(), b"\xff\xfe"] {
            for input in inputs {
                let mut bytes = prefix.repeat(32);
                bytes.extend_from_slice(input);
                check(&bytes);
            }
        }
    }

    #[test]
    fn every_unicode_scalar_and_mode_matches_iterators() {
        for ch in (0..=0x0010_ffff).filter_map(char::from_u32) {
            let mut encoded = [0; 4];
            check(ch.encode_utf8(&mut encoded).as_bytes());
        }
    }

    #[test]
    fn every_byte_pair_matches_iterators() {
        for left in 0..=u8::MAX {
            for right in 0..=u8::MAX {
                check(&[left, right]);
            }
        }
    }

    #[test]
    fn deterministic_binary_strings_match_iterators() {
        let mut state = 0x_1234_5678_u32;
        for length in 0..256 {
            let mut bytes = Vec::new();
            for _ in 0..length {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                bytes.push(state.to_le_bytes()[3]);
            }
            check(&bytes);
        }
    }

    #[test]
    fn results_match_mri_4_0_7_fixture() {
        fn decode_hex(value: &str) -> Vec<u8> {
            value
                .as_bytes()
                .chunks_exact(2)
                .map(|bytes| u8::from_str_radix(core::str::from_utf8(bytes).unwrap(), 16).unwrap())
                .collect()
        }
        // The fixture excludes Unicode 17/18 differences independently using
        // raw UCD data. MRI rejects malformed UTF-8, covered separately above.
        for line in include_str!("../tests/fixtures/mri-4.0.7-case-mapping.tsv").lines() {
            if line.starts_with('#') {
                continue;
            }
            let mut columns = line.split('\t');
            let input = decode_hex(columns.next().unwrap());
            let expected: Vec<_> = columns.map(decode_hex).collect();
            assert_eq!(expected.len(), 17);
            for (actual, expected) in results(&input).into_iter().zip(expected) {
                check_result(&input, &actual, &expected);
            }
        }
    }

    #[test]
    fn collector_detects_shorter_and_longer_matching_outputs() {
        check_result(
            b"abc",
            &super::try_collect(b"abc", b"ab".iter().copied()).unwrap(),
            b"ab",
        );
        check_result(
            b"ab",
            &super::try_collect(b"ab", b"abc".iter().copied()).unwrap(),
            b"abc",
        );
        check_result(
            b"a",
            &super::try_collect(b"a", core::iter::empty()).unwrap(),
            b"",
        );
    }
}
