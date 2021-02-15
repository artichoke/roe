# roe

[![GitHub Actions](https://github.com/artichoke/roe/workflows/CI/badge.svg)](https://github.com/artichoke/roe/actions)
[![Discord](https://img.shields.io/discord/607683947496734760)](https://discord.gg/QCe2tp2)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/roe.svg)](https://crates.io/crates/roe)
[![API](https://docs.rs/roe/badge.svg)](https://docs.rs/roe)
[![API trunk](https://img.shields.io/badge/docs-trunk-blue.svg)](https://artichoke.github.io/roe/roe/)

Implements [Unicode case mapping] for [conventionally UTF-8 binary strings].

> Case mapping or case conversion is a process whereby strings are converted to
> a particular form—uppercase, lowercase, or titlecase—possibly for display to
> the user.

`roe` can convert conventionally UTF-8 binary strings to capitalized, lowercase,
and uppercase forms. This crate is used to implement [`String#capitalize`],
[`Symbol#capitalize`], [`String#downcase`], [`Symbol#downcase`],
[`String#upcase`], and [`Symbol#upcase`] in [Artichoke Ruby].

This crate depends on [`bstr`].

## Status

This crate is currently a _work in progress_. When the API is complete, Roe will
support lowercase, uppercase, titlecase, and case folding iterators for
conventionally UTF-8 byte slices.

Roe will implement support for full, Turkic, ASCII, and case folding transforms.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
roe = "0.0.1"
```

Then convert case like:

```rust
use roe::{LowercaseMode, UppercaseMode};

assert_eq!(roe::lowercase(b"Artichoke Ruby", LowercaseMode::Ascii).collect::<Vec<_>>(), b"artichoke ruby");
assert_eq!(roe::uppercase("Αύριο".as_bytes(), UppercaseMode::Full).collect::<Vec<_>>(), "ΑΎΡΙΟ".as_bytes());
```

## Crate Features

`roe` is `no_std` compatible with an optional dependency on the [`alloc`] crate.

`roe` has several Cargo features, all of which are enabled by default:

- **std** - Adds a dependency on [`std`], the Rust Standard Library. This
  feature enables [`std::error::Error`] implementations on error types in this
  crate. Enabling the **std** feature also enables the **alloc** feature.
- **alloc** - Adds a dependency on [`alloc`], the Rust allocation and
  collections library. This feature enables APIs that allocate [`String`] or
  [`Vec`].

## License

`roe` is licensed under the [MIT License](LICENSE) (c) Ryan Lopopolo.

[unicode case mapping]: https://unicode.org/faq/casemap_charprop.html#casemap
[conventionally utf-8 binary strings]:
  https://docs.rs/bstr/0.2.*/bstr/#when-should-i-use-byte-strings
[`string#capitalize`]:
  https://ruby-doc.org/core-2.6.3/String.html#method-i-capitalize
[`symbol#capitalize`]:
  https://ruby-doc.org/core-2.6.3/Symbol.html#method-i-capitalize
[`string#downcase`]:
  https://ruby-doc.org/core-2.6.3/String.html#method-i-downcase
[`symbol#downcase`]:
  https://ruby-doc.org/core-2.6.3/Symbol.html#method-i-downcase
[`string#upcase`]: https://ruby-doc.org/core-2.6.3/String.html#method-i-upcase
[`symbol#upcase`]: https://ruby-doc.org/core-2.6.3/Symbol.html#method-i-upcase
[artichoke ruby]: https://github.com/artichoke/artichoke
[`bstr`]: https://crates.io/crates/bstr
[`alloc`]: https://doc.rust-lang.org/alloc/index.html
[`std`]: https://doc.rust-lang.org/std/index.html
[`std::error::error`]: https://doc.rust-lang.org/std/error/trait.Error.html
[`string`]: https://doc.rust-lang.org/stable/alloc/string/struct.String.html
[`vec`]: https://doc.rust-lang.org/stable/alloc/vec/struct.Vec.html
