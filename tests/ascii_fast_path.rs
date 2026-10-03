use roe::ruby::{CapitalizeMode, capitalize};
use roe::{LowercaseMode, SwapcaseMode, UppercaseMode, lowercase, swapcase, uppercase};

#[test]
fn ascii_after_expansions_and_invalid_utf8() {
    assert_eq!(
        lowercase("İA".as_bytes(), LowercaseMode::Full).collect::<Vec<_>>(),
        "i\u{307}a".as_bytes()
    );
    assert_eq!(
        lowercase(b"\xef\xac\x83A\xc3\x9f\xffI", LowercaseMode::Fold).collect::<Vec<_>>(),
        b"ffiass\xffi"
    );
    assert_eq!(
        uppercase(b"\xef\xac\x83a\xc3\x9f\xffi", UppercaseMode::Full).collect::<Vec<_>>(),
        b"FFIASS\xffI"
    );
    assert_eq!(
        swapcase(b"\xef\xac\x83A\xc3\x9f\xffi", SwapcaseMode::Full).collect::<Vec<_>>(),
        b"FFIaSS\xffI"
    );
    assert_eq!(
        capitalize(b"\xef\xac\x83A\xc3\x9f\xffI", CapitalizeMode::Full).collect::<Vec<_>>(),
        b"Ffia\xc3\x9f\xffi"
    );
}

#[test]
fn turkic_i_does_not_take_the_single_byte_path() {
    assert_eq!(
        lowercase(b"Ia\xc4\xb0\xffI", LowercaseMode::Turkic).collect::<Vec<_>>(),
        b"\xc4\xb1ai\xff\xc4\xb1"
    );
    assert_eq!(
        uppercase(b"iA\xef\xac\x83\xffi", UppercaseMode::Turkic).collect::<Vec<_>>(),
        b"\xc4\xb0AFFI\xff\xc4\xb0"
    );
    assert_eq!(
        swapcase(b"Ii\xef\xac\x83\xffIi", SwapcaseMode::Turkic).collect::<Vec<_>>(),
        b"\xc4\xb1\xc4\xb0FFI\xff\xc4\xb1\xc4\xb0"
    );
    assert_eq!(
        capitalize(b"iI\xef\xac\x83\xffI", CapitalizeMode::Turkic).collect::<Vec<_>>(),
        b"\xc4\xb0\xc4\xb1\xef\xac\x83\xff\xc4\xb1"
    );
}
