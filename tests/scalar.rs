use core::array;
use core::iter::Take;

use roe::{
    LowercaseMode, SwapcaseMode, UppercaseMode, lowercase, lowercase_char, swapcase, swapcase_char,
    uppercase, uppercase_char,
};

fn utf8(character: char) -> Take<array::IntoIter<u8, 4>> {
    let mut bytes = [0; 4];
    let width = character.encode_utf8(&mut bytes).len();
    bytes.into_iter().take(width)
}

#[test]
fn scalar_mappings_match_byte_iterators_for_every_unicode_scalar() {
    for point in 0..=0x0010_FFFF {
        let Some(character) = char::from_u32(point) else {
            continue;
        };
        let mut bytes = [0; 4];
        let source = character.encode_utf8(&mut bytes).as_bytes();
        for mode in [
            LowercaseMode::Full,
            LowercaseMode::Ascii,
            LowercaseMode::Turkic,
            LowercaseMode::Lithuanian,
            LowercaseMode::Fold,
        ] {
            assert!(
                lowercase_char(character, mode)
                    .flat_map(utf8)
                    .eq(lowercase(source, mode)),
                "U+{point:04X}: {mode:?}"
            );
        }
        for mode in [
            UppercaseMode::Full,
            UppercaseMode::Ascii,
            UppercaseMode::Turkic,
            UppercaseMode::Lithuanian,
        ] {
            assert!(
                uppercase_char(character, mode)
                    .flat_map(utf8)
                    .eq(uppercase(source, mode)),
                "U+{point:04X}: {mode:?}"
            );
        }
        for mode in [
            SwapcaseMode::Full,
            SwapcaseMode::Ascii,
            SwapcaseMode::Turkic,
            SwapcaseMode::Lithuanian,
        ] {
            assert!(
                swapcase_char(character, mode)
                    .flat_map(utf8)
                    .eq(swapcase(source, mode)),
                "U+{point:04X}: {mode:?}"
            );
        }
    }
}

#[test]
fn expanded_iterators_account_for_mixed_front_and_back_consumption() {
    assert_eq!(roe::CaseMapping::default().len(), 0);
    let mut mapping = lowercase_char('ﬃ', LowercaseMode::Fold);
    assert_eq!(mapping.len(), 3);
    assert_eq!(mapping.to_string(), "ffi");
    assert_eq!(mapping.next(), Some('f'));
    assert_eq!(mapping.len(), 2);
    assert_eq!(mapping.next_back(), Some('i'));
    assert_eq!(mapping.size_hint(), (1, Some(1)));
    assert_eq!(mapping.clone().count(), 1);
    assert_eq!(mapping.next_back(), Some('f'));
    assert_eq!(mapping.len(), 0);
    assert_eq!(mapping.next(), None);
    assert_eq!(mapping.next_back(), None);
    assert_eq!(mapping.next(), None);
    assert!(lowercase_char('\0', LowercaseMode::Fold).eq(['\0']));
}

#[test]
fn callers_can_decode_utf16_without_an_intermediate_string() {
    let original = char::decode_utf16([0x004D, 0x0061, 0x00DF, 0x0065]).map(Result::unwrap);
    let folded = original.flat_map(|character| lowercase_char(character, LowercaseMode::Fold));
    assert!(folded.eq(['m', 'a', 's', 's', 'e']));
}
