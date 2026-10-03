use roe::ruby::{CapitalizeMode, capitalize};
use roe::{LowercaseMode, SwapcaseMode, UppercaseMode, lowercase, uppercase};

fn check_remaining(mut iter: impl Iterator<Item = u8> + Clone) {
    // Count by advancing explicitly so the oracle does not use `count` or
    // allocation routines that may themselves rely on `size_hint`.
    loop {
        let mut remaining = iter.clone();
        let mut actual = 0;
        while remaining.next().is_some() {
            actual += 1;
        }
        let (min, max) = iter.size_hint();
        assert!(min <= actual, "lower bound {min} exceeds {actual}");
        if let Some(max) = max {
            assert!(actual <= max, "upper bound {max} is below {actual}");
        }
        assert_eq!(iter.clone().count(), actual);
        if iter.next().is_none() {
            assert_eq!(iter.next(), None);
            assert_eq!(iter.size_hint(), (0, Some(0)));
            break;
        }
    }
}

#[test]
fn lowercase_remaining_output() {
    for input in inputs() {
        check_remaining(lowercase(input, LowercaseMode::Full));
        check_remaining(lowercase(input, LowercaseMode::Ascii));
        check_remaining(lowercase(input, LowercaseMode::Turkic));
        check_remaining(lowercase(input, LowercaseMode::Fold));
        check_remaining(lowercase(input, LowercaseMode::Lithuanian));
    }
}

#[test]
fn uppercase_remaining_output() {
    for input in inputs() {
        check_remaining(uppercase(input, UppercaseMode::Full));
        check_remaining(uppercase(input, UppercaseMode::Ascii));
        check_remaining(uppercase(input, UppercaseMode::Turkic));
        check_remaining(uppercase(input, UppercaseMode::Lithuanian));
    }
}

#[test]
fn capitalize_remaining_output() {
    for input in inputs() {
        check_remaining(capitalize(input, CapitalizeMode::Full));
        check_remaining(capitalize(input, CapitalizeMode::Ascii));
        check_remaining(capitalize(input, CapitalizeMode::Turkic));
        check_remaining(capitalize(input, CapitalizeMode::Lithuanian));
    }
}

#[test]
fn swapcase_remaining_output() {
    for input in inputs() {
        for mode in [
            SwapcaseMode::Full,
            SwapcaseMode::Ascii,
            SwapcaseMode::Turkic,
            SwapcaseMode::Lithuanian,
        ] {
            check_remaining(roe::swapcase(input, mode));
        }
    }
}

fn inputs() -> impl Iterator<Item = &'static [u8]> {
    [
        &b""[..],
        b"abcXYZ",
        b"iI",
        "ǅᾈ".as_bytes(),
        "ß".as_bytes(),
        "ßABC".as_bytes(),
        "İ".as_bytes(),
        "İABC".as_bytes(),
        "ﬃ".as_bytes(),
        "ﬃABC".as_bytes(),
        "K".as_bytes(),
        "KABC".as_bytes(),
        "AK".as_bytes(),
        "ı".as_bytes(),
        "ıABC".as_bytes(),
        "Ⱥ".as_bytes(),
        "ȺABC".as_bytes(),
        "𐐜".as_bytes(),
        "𐐜ABC".as_bytes(),
        "ᾂῷﬗ".as_bytes(),
        b"\xf0\x9f\x87",
        b"\xf0\x9f\x87ABC",
        b"A\xff\xfeZ",
        b"\0ABC",
    ]
    .into_iter()
}
