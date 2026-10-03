use std::collections::BTreeMap;

use roe::{LowercaseMode, TitlecaseMode, UppercaseMode, lowercase, titlecase, uppercase};

type Mappings = BTreeMap<char, Vec<u8>>;

fn scalar(code: &str) -> char {
    char::from_u32(u32::from_str_radix(code, 16).unwrap()).unwrap()
}

fn mapping(codes: &str) -> Vec<u8> {
    codes
        .split_whitespace()
        .map(scalar)
        .collect::<String>()
        .into_bytes()
}

fn expected_mappings() -> [Mappings; 4] {
    let mut maps: [Mappings; 4] = std::array::from_fn(|_| BTreeMap::new());
    for line in include_str!("../generated/ucd/UnicodeData.txt").lines() {
        let fields: Vec<_> = line.split(';').collect();
        for (kind, column) in [(0, 13), (1, 12), (2, 14)] {
            let mut target = fields[column];
            if kind == 2 && target.is_empty() {
                target = fields[12];
            }
            if !target.is_empty() {
                maps[kind].insert(scalar(fields[0]), mapping(target));
            }
        }
    }
    for line in include_str!("../generated/ucd/SpecialCasing.txt").lines() {
        let data = line.split('#').next().unwrap().trim();
        if data.is_empty() {
            continue;
        }
        let fields: Vec<_> = data.split(';').map(str::trim).collect();
        if !fields[4].is_empty() {
            continue;
        }
        for (kind, column) in [(0, 1), (1, 3), (2, 2)] {
            maps[kind].insert(scalar(fields[0]), mapping(fields[column]));
        }
    }
    for line in include_str!("../generated/ucd/CaseFolding.txt").lines() {
        let data = line.split('#').next().unwrap().trim();
        if data.is_empty() {
            continue;
        }
        let fields: Vec<_> = data.split(';').map(str::trim).collect();
        if matches!(fields[1], "C" | "F") {
            maps[3].insert(scalar(fields[0]), mapping(fields[2]));
        }
    }
    maps
}

#[test]
fn all_unicode_scalars_match_ucd() {
    let maps = expected_mappings();
    for c in (0..=0x10ffff).filter_map(char::from_u32) {
        let mut encoded = [0; 4];
        let input = c.encode_utf8(&mut encoded).as_bytes();
        let actual = [
            lowercase(input, LowercaseMode::Full).collect::<Vec<_>>(),
            uppercase(input, UppercaseMode::Full).collect::<Vec<_>>(),
            titlecase(input, TitlecaseMode::Full).collect::<Vec<_>>(),
            lowercase(input, LowercaseMode::Fold).collect::<Vec<_>>(),
        ];
        for (kind, (actual, table)) in actual.iter().zip(&maps).enumerate() {
            // MRI capitalization lowercases Georgian Mtavruli capitals.
            let table = if kind == 2 && ('\u{1c90}'..='\u{1cbf}').contains(&c) {
                &maps[0]
            } else {
                table
            };
            let expected = table.get(&c).map_or(input, Vec::as_slice);
            assert_eq!(actual, expected, "mapping {kind}, U+{:04X}", u32::from(c));
        }
    }
}

#[test]
fn turkic_mapping_and_capitalization() {
    let input = "Iİiı STRAẞE ﬃ".as_bytes();
    assert_eq!(
        lowercase(input, LowercaseMode::Turkic).collect::<Vec<_>>(),
        "ıiiı straße ﬃ".as_bytes()
    );
    assert_eq!(
        uppercase(input, UppercaseMode::Turkic).collect::<Vec<_>>(),
        "IİİI STRAẞE FFI".as_bytes()
    );
    assert_eq!(
        titlecase("iIİı ABC".as_bytes(), TitlecaseMode::Turkic).collect::<Vec<_>>(),
        "İıiı abc".as_bytes()
    );
    assert_eq!(
        titlecase("ﬃ Iİ".as_bytes(), TitlecaseMode::Turkic).collect::<Vec<_>>(),
        "Ffi ıi".as_bytes()
    );
    // Like MRI, Turkic casing is not contextual: the combining dot is retained.
    assert_eq!(
        lowercase("I\u{307}".as_bytes(), LowercaseMode::Turkic).collect::<Vec<_>>(),
        "ı\u{307}".as_bytes()
    );
}

#[test]
fn folding_is_distinct_from_lowercase() {
    let input = "ßẞﬃΣςİᎠ".as_bytes();
    assert_eq!(
        lowercase(input, LowercaseMode::Fold).collect::<Vec<_>>(),
        "ssssffiσσi\u{307}Ꭰ".as_bytes()
    );
    assert_eq!(
        lowercase(input, LowercaseMode::Full).collect::<Vec<_>>(),
        "ßßﬃσςi\u{307}ꭰ".as_bytes()
    );
}

#[test]
fn invalid_utf8_is_preserved_in_new_modes() {
    let input = b"I\xff\xf0\x9f\x87i";
    assert_eq!(
        lowercase(input, LowercaseMode::Turkic).collect::<Vec<_>>(),
        b"\xc4\xb1\xff\xf0\x9f\x87i"
    );
    assert_eq!(
        uppercase(input, UppercaseMode::Turkic).collect::<Vec<_>>(),
        b"I\xff\xf0\x9f\x87\xc4\xb0"
    );
    assert_eq!(
        titlecase(input, TitlecaseMode::Turkic).collect::<Vec<_>>(),
        input
    );
    assert_eq!(
        lowercase(input, LowercaseMode::Fold).collect::<Vec<_>>(),
        b"i\xff\xf0\x9f\x87i"
    );
}

#[test]
fn lithuanian_remains_an_alias_for_full() {
    let input = "I\u{301}İß".as_bytes();
    assert_eq!(
        lowercase(input, LowercaseMode::Lithuanian).collect::<Vec<_>>(),
        lowercase(input, LowercaseMode::Full).collect::<Vec<_>>()
    );
    assert_eq!(
        uppercase(input, UppercaseMode::Lithuanian).collect::<Vec<_>>(),
        uppercase(input, UppercaseMode::Full).collect::<Vec<_>>()
    );
    assert_eq!(
        titlecase(input, TitlecaseMode::Lithuanian).collect::<Vec<_>>(),
        titlecase(input, TitlecaseMode::Full).collect::<Vec<_>>()
    );
}

#[test]
fn georgian_capitalization_differs_from_character_titlecase() {
    assert_eq!(roe::to_titlecase('Ა'), ['Ა', '\0', '\0']);
    assert_eq!(
        titlecase("ᲐᲑ".as_bytes(), TitlecaseMode::Full).collect::<Vec<_>>(),
        "აბ".as_bytes()
    );
}
