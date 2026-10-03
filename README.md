# roe

[![GitHub Actions](https://github.com/artichoke/roe/workflows/CI/badge.svg)](https://github.com/artichoke/roe/actions)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/roe.svg)](https://crates.io/crates/roe)
[![API](https://docs.rs/roe/badge.svg)](https://docs.rs/roe)

Implements [Unicode case mapping] for [conventionally UTF-8 binary strings].

[unicode case mapping]: https://unicode.org/faq/casemap_charprop.html#casemap
[conventionally utf-8 binary strings]:
  https://docs.rs/bstr/1.*/bstr/#when-should-i-use-byte-strings

> Case mapping or case conversion is a process whereby strings are converted to
> a particular form—uppercase, lowercase, or titlecase—possibly for display to
> the user.

`roe` can convert conventionally UTF-8 binary strings to capitalized, lowercase,
and uppercase forms. This crate is used to implement [`String#capitalize`],
[`Symbol#capitalize`], [`String#downcase`], [`Symbol#downcase`],
[`String#upcase`], [`Symbol#upcase`], [`String#swapcase`], and
[`Symbol#swapcase`] in [Artichoke Ruby].

[`string#capitalize`]:
  https://docs.ruby-lang.org/en/4.0/String.html#method-i-capitalize
[`symbol#capitalize`]:
  https://docs.ruby-lang.org/en/4.0/Symbol.html#method-i-capitalize
[`string#downcase`]:
  https://docs.ruby-lang.org/en/4.0/String.html#method-i-downcase
[`symbol#downcase`]:
  https://docs.ruby-lang.org/en/4.0/Symbol.html#method-i-downcase
[`string#upcase`]: https://docs.ruby-lang.org/en/4.0/String.html#method-i-upcase
[`symbol#upcase`]: https://docs.ruby-lang.org/en/4.0/Symbol.html#method-i-upcase
[`string#swapcase`]:
  https://docs.ruby-lang.org/en/4.0/String.html#method-i-swapcase
[`symbol#swapcase`]:
  https://docs.ruby-lang.org/en/4.0/Symbol.html#method-i-swapcase
[artichoke ruby]: https://github.com/artichoke/artichoke

This crate depends on [`bstr`].

[`bstr`]: https://crates.io/crates/bstr

## Implementation

Roe generates conversion tables from Unicode Data Files. Roe implements case
mapping as defined in the [Unicode standard][casemap] (see [`PropList.txt`],
[`SpecialCasing.txt`], [`UnicodeData.txt`], [`CaseFolding.txt`]).

[casemap]: https://unicode.org/faq/casemap_charprop.html#casemap
[`proplist.txt`]: generated/ucd/PropList.txt
[`specialcasing.txt`]: generated/ucd/SpecialCasing.txt
[`unicodedata.txt`]: generated/ucd/UnicodeData.txt
[`casefolding.txt`]: generated/ucd/CaseFolding.txt

## Status

Roe supports full Unicode, Turkic, and ASCII lowercase, uppercase, and
capitalization and swapcase mappings for conventionally UTF-8 byte slices. Full
Unicode case folding is available through `LowercaseMode::Fold`. Invalid UTF-8
is preserved.

Like MRI Ruby, mappings are context-independent and Lithuanian mode currently
uses the full Unicode mappings. All Unicode mappings use bundled tables rather
than the Rust compiler's Unicode version.

The API is still evolving ahead of a 1.0 release.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
roe = "0.0.9"
```

Then convert case like:

```rust
use roe::ruby::{CapitalizeMode, capitalize};
use roe::{LowercaseMode, UppercaseMode};

assert_eq!(
    roe::lowercase(b"Artichoke Ruby", LowercaseMode::Ascii).collect::<Vec<_>>(),
    b"artichoke ruby"
);
assert_eq!(
    roe::uppercase("Αύριο".as_bytes(), UppercaseMode::Full).collect::<Vec<_>>(),
    "ΑΎΡΙΟ".as_bytes()
);
assert_eq!(
    capitalize("ﬃ".as_bytes(), CapitalizeMode::Full).collect::<Vec<_>>(),
    "Ffi".as_bytes()
);
```

## Crate Features

`roe` is `no_std` compatible with an optional dependency on the [`alloc`] crate.

The **alloc** feature is enabled by default and provides APIs that allocate
[`String`] or [`Vec`]. Disable default features to use Roe without allocation.

[`alloc`]: https://doc.rust-lang.org/alloc/index.html
[`string`]: https://doc.rust-lang.org/stable/alloc/string/struct.String.html
[`vec`]: https://doc.rust-lang.org/stable/alloc/vec/struct.Vec.html

### Minimum Supported Rust Version

This crate requires at least Rust 1.85.0. This version can be bumped in minor
releases.

## Unicode Version

Roe implements Unicode case mapping with the Unicode 18.0.0 case mapping
ruleset.

Each new release of Unicode may bring updates to the Data Files which are the
source for the case mappings in this crate. Updates to the case mapping rules
will be accompanied with a minor version bump.

## License

`roe` is licensed under the [MIT License](LICENSE) (c) Ryan Lopopolo.

`roe` includes Unicode Data Files which are subject to the [Unicode Terms of
Use] and [Unicode License v3](LICENSE-UNICODE) (c) Unicode, Inc.

[unicode terms of use]: https://www.unicode.org/copyright.html
