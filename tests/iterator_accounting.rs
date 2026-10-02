use roe::{lowercase, titlecase, uppercase, LowercaseMode, TitlecaseMode, UppercaseMode};

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
    }
}

#[test]
fn uppercase_remaining_output() {
    for input in inputs() {
        check_remaining(uppercase(input, UppercaseMode::Full));
        check_remaining(uppercase(input, UppercaseMode::Ascii));
    }
}

#[test]
fn titlecase_remaining_output() {
    for input in inputs() {
        check_remaining(titlecase(input, TitlecaseMode::Full));
        check_remaining(titlecase(input, TitlecaseMode::Ascii));
    }
}

fn inputs() -> impl Iterator<Item = &'static [u8]> {
    [
        &b""[..],
        b"abcXYZ",
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
