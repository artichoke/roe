use roe::ruby::{CapitalizeMode as C, capitalize};
use roe::{LowercaseMode as L, SwapcaseMode as S, UppercaseMode as U};

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|bytes| u8::from_str_radix(core::str::from_utf8(bytes).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn mapping_corpus_matches_mri_4_0_7() {
    for (row, line) in include_str!("fixtures/mri-4.0.7-case-mapping.tsv")
        .lines()
        .enumerate()
    {
        if line.starts_with('#') {
            continue;
        }
        let columns: Vec<_> = line.split('\t').collect();
        assert_eq!(columns.len(), 18);
        let input = decode_hex(columns[0]);
        let mut actual = Vec::new();
        for mode in [L::Full, L::Ascii, L::Turkic, L::Lithuanian, L::Fold] {
            actual.push(roe::lowercase(&input, mode).collect::<Vec<_>>());
        }
        for mode in [U::Full, U::Ascii, U::Turkic, U::Lithuanian] {
            actual.push(roe::uppercase(&input, mode).collect::<Vec<_>>());
        }
        for mode in [C::Full, C::Ascii, C::Turkic, C::Lithuanian] {
            actual.push(capitalize(&input, mode).collect::<Vec<_>>());
        }
        for mode in [S::Full, S::Ascii, S::Turkic, S::Lithuanian] {
            actual.push(roe::swapcase(&input, mode).collect::<Vec<_>>());
        }
        for (operation, (actual, expected)) in actual.iter().zip(&columns[1..]).enumerate() {
            assert_eq!(
                *actual,
                decode_hex(expected),
                "fixture row {row}, operation {operation}, input {}",
                columns[0]
            );
        }
    }
}

#[test]
fn ascii_swapcase_preserves_binary_bytes() {
    let mut bytes: Vec<_> = (0..=u8::MAX).collect();
    let expected: Vec<_> = bytes
        .iter()
        .map(|&byte| {
            if byte.is_ascii_alphabetic() {
                byte ^ 0x20
            } else {
                byte
            }
        })
        .collect();
    assert_eq!(
        roe::swapcase(&bytes, S::Ascii).collect::<Vec<_>>(),
        expected
    );
    #[cfg(feature = "alloc")]
    assert_eq!(roe::to_ascii_swapcase(&bytes), expected);
    roe::make_ascii_swapcase(&mut bytes);
    assert_eq!(bytes, expected);
}

#[test]
fn swapcase_preserves_invalid_utf8() {
    assert_eq!(
        roe::swapcase(b"iI\xff\xf0\x9f\x87", S::Turkic).collect::<Vec<_>>(),
        b"\xc4\xb0\xc4\xb1\xff\xf0\x9f\x87"
    );
}

#[test]
fn swapcase_rejects_folding_mode() {
    assert!("fold".parse::<S>().is_err());
    assert_eq!(S::try_from(None::<&str>), Ok(S::Full));
    assert_eq!(S::try_from("turkic"), Ok(S::Turkic));
}
