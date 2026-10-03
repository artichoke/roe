use roe::ruby::{Capitalize, CapitalizeMode, capitalize};

#[test]
fn capitalization_modes_match_existing_byte_iterators() {
    for mode in [
        CapitalizeMode::Full,
        CapitalizeMode::Ascii,
        CapitalizeMode::Turkic,
        CapitalizeMode::Lithuanian,
    ] {
        for input in [b"iI\xff".as_slice(), "ᲐᲑᲒ".as_bytes(), "ǅﬃ ABC".as_bytes()] {
            let iter: Capitalize<'_> = capitalize(input, mode);
            assert_eq!(
                iter.collect::<Vec<_>>(),
                roe::titlecase(input, mode).collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn public_unicode_version_matches_bundled_data() {
    let (major, minor, patch) = roe::UNICODE_VERSION;
    let marker = format!("final data files for version {major}.{minor}.{patch}");
    assert!(include_str!("../generated/ucd/ReadMe.txt").contains(&marker));
}
