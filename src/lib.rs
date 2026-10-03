#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![cfg_attr(test, allow(clippy::non_ascii_literal))]
#![cfg_attr(test, allow(clippy::shadow_unrelated))]
#![warn(clippy::cargo)]
#![allow(unknown_lints)]
#![allow(clippy::struct_field_names)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
#![forbid(unsafe_code)]
// Enable feature callouts in generated documentation:
// https://doc.rust-lang.org/beta/unstable-book/language-features/doc-cfg.html
//
// This approach is borrowed from tokio.
#![cfg_attr(docsrs, feature(doc_cfg))]

//! This crate provides [Unicode case mapping] routines and iterators for
//! [conventionally UTF-8 binary strings].
//!
//! Unicode case mapping or case conversion can be used to transform the
//! characters in a string. To quote the Unicode FAQ:
//!
//! > Case mapping or case conversion is a process whereby strings are converted
//! > to a particular form—uppercase, lowercase, or titlecase—possibly for
//! > display to the user.
//!
//! Roe supports full Unicode, Turkic, and ASCII lowercase, uppercase, and
//! capitalization and swapcase mappings. Full Unicode case folding is available through
//! [`LowercaseMode::Fold`]. Invalid UTF-8 is preserved.
//!
//! Mappings use bundled Unicode 18.0.0 tables, independent of the Rust compiler.
//! Like MRI Ruby, mappings are context-independent and Lithuanian mode is an
//! alias for full Unicode mapping. The API is still evolving ahead of 1.0.
//!
//! # Usage
//!
//! You can convert case like:
//!
//! ```
//! # use roe::{LowercaseMode, UppercaseMode, TitlecaseMode};
//! assert_eq!(
//!     roe::lowercase(b"Artichoke Ruby", LowercaseMode::Ascii).collect::<Vec<_>>(),
//!     b"artichoke ruby"
//! );
//! assert_eq!(
//!     roe::uppercase("Αύριο".as_bytes(), UppercaseMode::Full).collect::<Vec<_>>(),
//!     "ΑΎΡΙΟ".as_bytes()
//! );
//! assert_eq!(
//!     roe::titlecase("ﬃ".as_bytes(), TitlecaseMode::Full).collect::<Vec<_>>(),
//!     "Ffi".as_bytes()
//! );
//! ```
//!
//!
//! Roe provides fast path routines that assume the byte slice is ASCII-only.
//!
//! # Crate Features
//!
//! Roe is `no_std` compatible with an optional dependency on the [`alloc`]
//! crate.
//!
//! The **alloc** feature is enabled by default and provides APIs that allocate
//! [`String`] or [`Vec`]. Disable default features to use Roe without allocation.
//!
#![cfg_attr(
    not(feature = "alloc"),
    doc = "[`alloc`]: https://doc.rust-lang.org/alloc/index.html"
)]
#![cfg_attr(feature = "alloc", doc = "[`String`]: alloc::string::String")]
#![cfg_attr(
    not(feature = "alloc"),
    doc = "[`String`]: https://doc.rust-lang.org/alloc/string/struct.String.html"
)]
#![cfg_attr(feature = "alloc", doc = "[`Vec`]: alloc::vec::Vec")]
#![cfg_attr(
    not(feature = "alloc"),
    doc = "[`Vec`]: https://doc.rust-lang.org/alloc/vec/struct.Vec.html"
)]
//! [Unicode case mapping]: https://unicode.org/faq/casemap_charprop.html#casemap
//! [conventionally UTF-8 binary strings]: https://docs.rs/bstr/1.*/bstr/#when-should-i-use-byte-strings

#![no_std]
#![doc(html_root_url = "https://docs.rs/roe/0.0.8")]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

use core::convert::{TryFrom, TryInto};
use core::fmt;
use core::str::FromStr;

mod ascii;
mod lowercase;
pub mod ruby;
mod swapcase;
mod titlecase;
mod unicode;
mod uppercase;

/// Roe is derived from Unicode Data Files and is subject to Unicode License v3.
///
/// See <https://www.unicode.org/terms_of_use.html>.
///
/// # Unicode License v3
///
/// ```txt
#[doc = include_str!("../LICENSE-UNICODE")]
/// ```
#[cfg(doc)]
#[cfg_attr(docsrs, doc(cfg(doc)))]
pub mod unicode_terms {}

pub use ascii::{
    make_ascii_lowercase, make_ascii_swapcase, make_ascii_titlecase, make_ascii_uppercase,
};
#[cfg(feature = "alloc")]
pub use ascii::{to_ascii_lowercase, to_ascii_swapcase, to_ascii_titlecase, to_ascii_uppercase};
pub use lowercase::Lowercase;
pub use swapcase::Swapcase;
pub use titlecase::Titlecase;
pub use unicode::to_titlecase;
pub use unicode::UNICODE_VERSION;
pub use uppercase::Uppercase;

/// Error that indicates a failure to parse a [`LowercaseMode`],
/// [`UppercaseMode`], [`SwapcaseMode`], or [`TitlecaseMode`].
///
/// This error corresponds to the [Ruby `ArgumentError` Exception class].
///
/// # Examples
///
/// ```
/// # use core::convert::TryInto;
/// # use roe::{InvalidCaseMappingMode, LowercaseMode};
/// let err = InvalidCaseMappingMode::new();
/// assert_eq!(err.message(), "invalid option");
///
/// let mode: Result<LowercaseMode, InvalidCaseMappingMode> = "full".try_into();
/// ```
///
/// [Ruby `ArgumentError` Exception class]: https://ruby-doc.org/core-3.1.2/ArgumentError.html
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidCaseMappingMode {
    _private: (),
}

impl InvalidCaseMappingMode {
    /// Construct a new `InvalidCaseMappingMode` error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::InvalidCaseMappingMode;
    /// const ERR: InvalidCaseMappingMode = InvalidCaseMappingMode::new();
    /// assert_eq!(ERR.message(), "invalid option");
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Retrieve the error message associated with this `InvalidCaseMappingMode`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::InvalidCaseMappingMode;
    /// const MESSAGE: &str = InvalidCaseMappingMode::new().message();
    /// assert_eq!(MESSAGE, "invalid option");
    /// ```
    #[must_use]
    #[allow(clippy::unused_self)]
    pub const fn message(self) -> &'static str {
        "invalid option"
    }
}

impl fmt::Display for InvalidCaseMappingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const MESSAGE: &str = InvalidCaseMappingMode::new().message();
        f.write_str(MESSAGE)
    }
}

impl core::error::Error for InvalidCaseMappingMode {}

/// Options to configure the behavior of [`lowercase`].
///
/// Which letters exactly are replaced, and by which other letters, depends on
/// the given options.
///
/// See individual variants for a description of the available behaviors.
///
/// If you're not sure which mode to choose, [`LowercaseMode::Full`] is a a good
/// default.
///
/// [`lowercase`]: crate::lowercase()
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LowercaseMode {
    /// Full Unicode case mapping, suitable for most languages.
    ///
    /// See the [Turkic] and [Lithuanian] variants for exceptions.
    ///
    /// Context-dependent case mapping as described in Table 3-14 of the Unicode
    /// standard is currently not supported.
    ///
    /// [Turkic]: Self::Turkic
    /// [Lithuanian]: Self::Lithuanian
    #[default]
    Full,
    /// Only the ASCII region, i.e. the characters `'A'..='Z'` and `'a'..='z'`,
    /// are affected.
    ///
    /// This option cannot be combined with any other option.
    Ascii,
    /// Full Unicode case mapping, adapted for Turkic languages (Turkish,
    /// Azerbaijani, …).
    ///
    /// This means that upper case I is mapped to lower case dotless i, and so
    /// on.
    Turkic,
    /// An alias for [full Unicode case mapping].
    ///
    /// This matches MRI Ruby, which does not implement Lithuanian contextual
    /// case mapping.
    ///
    /// [full Unicode case mapping]: Self::Full
    Lithuanian,
    /// Unicode case **folding**, which is more far-reaching than Unicode case
    /// mapping.
    ///
    /// This option currently cannot be combined with any other option (i.e.
    /// there is currently no variant for turkic languages).
    Fold,
}

impl TryFrom<&str> for LowercaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}

impl TryFrom<Option<&str>> for LowercaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        value.map(str::as_bytes).try_into()
    }
}

impl TryFrom<&[u8]> for LowercaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"ascii" => Ok(Self::Ascii),
            b"turkic" => Ok(Self::Turkic),
            b"lithuanian" => Ok(Self::Lithuanian),
            b"fold" => Ok(Self::Fold),
            _ => Err(InvalidCaseMappingMode::new()),
        }
    }
}

impl TryFrom<Option<&[u8]>> for LowercaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&[u8]>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(Self::Full),
            Some(b"ascii") => Ok(Self::Ascii),
            Some(b"turkic") => Ok(Self::Turkic),
            Some(b"lithuanian") => Ok(Self::Lithuanian),
            Some(b"fold") => Ok(Self::Fold),
            Some(_) => Err(InvalidCaseMappingMode::new()),
        }
    }
}

impl FromStr for LowercaseMode {
    type Err = InvalidCaseMappingMode;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

/// Returns an iterator that yields a copy of the bytes in the given slice with
/// all uppercase letters replaced with their lowercase counterparts.
///
/// This function treats the given slice as a [conventionally UTF-8 string].
/// UTF-8 byte sequences are converted to their Unicode lowercase equivalents.
/// Invalid UTF-8 byte sequences are yielded as is.
///
/// The case mapping mode is determined by the given [`LowercaseMode`]. See its
/// documentation for details on the available case mapping modes.
///
/// [conventionally UTF-8 string]: https://docs.rs/bstr/1.*/bstr/#when-should-i-use-byte-strings
pub const fn lowercase(slice: &[u8], options: LowercaseMode) -> Lowercase<'_> {
    match options {
        LowercaseMode::Full | LowercaseMode::Lithuanian => Lowercase::with_slice(slice),
        LowercaseMode::Ascii => Lowercase::with_ascii_slice(slice),
        LowercaseMode::Turkic => Lowercase::with_mode(slice, unicode::mapping::Mode::TurkicLower),
        LowercaseMode::Fold => Lowercase::with_mode(slice, unicode::mapping::Mode::Fold),
    }
}

/// Options to configure the behavior of [`uppercase`].
///
/// Which letters exactly are replaced, and by which other letters, depends on
/// the given options.
///
/// See individual variants for a description of the available behaviors.
///
/// If you're not sure which mode to choose, [`UppercaseMode::Full`] is a a good
/// default.
///
/// [`uppercase`]: crate::uppercase()
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum UppercaseMode {
    /// Full Unicode case mapping, suitable for most languages.
    ///
    /// See the [Turkic] and [Lithuanian] variants for exceptions.
    ///
    /// Context-dependent case mapping as described in Table 3-14 of the Unicode
    /// standard is currently not supported.
    ///
    /// [Turkic]: Self::Turkic
    /// [Lithuanian]: Self::Lithuanian
    #[default]
    Full,
    /// Only the ASCII region, i.e. the characters `'A'..='Z'` and `'a'..='z'`,
    /// are affected.
    ///
    /// This option cannot be combined with any other option.
    Ascii,
    /// Full Unicode case mapping, adapted for Turkic languages (Turkish,
    /// Azerbaijani, …).
    ///
    /// This means that upper case I is mapped to lower case dotless i, and so
    /// on.
    Turkic,
    /// Currently, just [full Unicode case mapping].
    ///
    /// This matches MRI Ruby, which does not implement Lithuanian contextual
    /// case mapping.
    ///
    /// [full Unicode case mapping]: Self::Full
    Lithuanian,
}

impl TryFrom<&str> for UppercaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}

impl TryFrom<Option<&str>> for UppercaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        value.map(str::as_bytes).try_into()
    }
}

impl TryFrom<&[u8]> for UppercaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"ascii" => Ok(Self::Ascii),
            b"turkic" => Ok(Self::Turkic),
            b"lithuanian" => Ok(Self::Lithuanian),
            _ => Err(InvalidCaseMappingMode::new()),
        }
    }
}

impl TryFrom<Option<&[u8]>> for UppercaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&[u8]>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(Self::Full),
            Some(b"ascii") => Ok(Self::Ascii),
            Some(b"turkic") => Ok(Self::Turkic),
            Some(b"lithuanian") => Ok(Self::Lithuanian),
            Some(_) => Err(InvalidCaseMappingMode::new()),
        }
    }
}

impl FromStr for UppercaseMode {
    type Err = InvalidCaseMappingMode;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

/// Returns an iterator that yields a copy of the bytes in the given slice with
/// all lowercase letters replaced with their uppercase counterparts.
///
/// This function treats the given slice as a [conventionally UTF-8 string].
/// UTF-8 byte sequences are converted to their Unicode uppercase equivalents.
/// Invalid UTF-8 byte sequences are yielded as is.
///
/// The case mapping mode is determined by the given [`UppercaseMode`]. See its
/// documentation for details on the available case mapping modes.
///
/// [conventionally UTF-8 string]: https://docs.rs/bstr/1.*/bstr/#when-should-i-use-byte-strings
pub const fn uppercase(slice: &[u8], options: UppercaseMode) -> Uppercase<'_> {
    match options {
        UppercaseMode::Full | UppercaseMode::Lithuanian => Uppercase::with_slice(slice),
        UppercaseMode::Ascii => Uppercase::with_ascii_slice(slice),
        UppercaseMode::Turkic => Uppercase::with_mode(slice, unicode::mapping::Mode::TurkicUpper),
    }
}

/// Options to configure the behavior of [`swapcase`].
///
/// Which letters exactly are replaced, and by which other letters, depends on
/// the given options.
///
/// See individual variants for a description of the available behaviors.
///
/// If you're not sure which mode to choose, [`SwapcaseMode::Full`] is a a good
/// default.
///
/// [`swapcase`]: crate::swapcase()
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SwapcaseMode {
    /// Full Unicode case mapping, suitable for most languages.
    ///
    /// See the [Turkic] and [Lithuanian] variants for exceptions.
    ///
    /// Context-dependent case mapping as described in Table 3-14 of the Unicode
    /// standard is currently not supported.
    ///
    /// [Turkic]: Self::Turkic
    /// [Lithuanian]: Self::Lithuanian
    #[default]
    Full,
    /// Only the ASCII region, i.e. the characters `'A'..='Z'` and `'a'..='z'`,
    /// are affected.
    ///
    /// This option cannot be combined with any other option.
    Ascii,
    /// Full Unicode case mapping, adapted for Turkic languages (Turkish,
    /// Azerbaijani, …).
    ///
    /// This means that upper case I is mapped to lower case dotless i, and so
    /// on.
    Turkic,
    /// Currently, just [full Unicode case mapping].
    ///
    /// This matches MRI Ruby, which does not implement Lithuanian contextual
    /// case mapping.
    ///
    /// [full Unicode case mapping]: Self::Full
    Lithuanian,
}

impl TryFrom<&str> for SwapcaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}

impl TryFrom<Option<&str>> for SwapcaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        value.map(str::as_bytes).try_into()
    }
}

impl TryFrom<&[u8]> for SwapcaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"ascii" => Ok(Self::Ascii),
            b"turkic" => Ok(Self::Turkic),
            b"lithuanian" => Ok(Self::Lithuanian),
            _ => Err(InvalidCaseMappingMode::new()),
        }
    }
}

impl TryFrom<Option<&[u8]>> for SwapcaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&[u8]>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(Self::Full),
            Some(b"ascii") => Ok(Self::Ascii),
            Some(b"turkic") => Ok(Self::Turkic),
            Some(b"lithuanian") => Ok(Self::Lithuanian),
            Some(_) => Err(InvalidCaseMappingMode::new()),
        }
    }
}

impl FromStr for SwapcaseMode {
    type Err = InvalidCaseMappingMode;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

/// Returns an iterator that yields a copy of the bytes in the given slice with
/// the case of each character swapped.
///
/// This function treats the given slice as a [conventionally UTF-8 string].
/// Uppercase characters are lowercased, lowercase characters are uppercased,
/// and titlecase letters have the case of their components swapped.
/// Invalid UTF-8 byte sequences are yielded as is.
///
/// The case mapping mode is determined by the given [`SwapcaseMode`]. See its
/// documentation for details on the available case mapping modes.
///
/// [conventionally UTF-8 string]: https://docs.rs/bstr/1.*/bstr/#when-should-i-use-byte-strings
pub const fn swapcase(slice: &[u8], options: SwapcaseMode) -> Swapcase<'_> {
    match options {
        SwapcaseMode::Full | SwapcaseMode::Lithuanian => Swapcase::with_slice(slice),
        SwapcaseMode::Ascii => Swapcase::with_ascii_slice(slice),
        SwapcaseMode::Turkic => Swapcase::with_mode(slice, unicode::mapping::Mode::TurkicSwap),
    }
}

/// Options to configure the behavior of [`titlecase`].
///
/// Which letters exactly are replaced, and by which other letters, depends on
/// the given options.
///
/// See individual variants for a description of the available behaviors.
///
/// If you're not sure which mode to choose, [`TitlecaseMode::Full`] is a good
/// default.
///
/// [`titlecase`]: crate::titlecase()
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TitlecaseMode {
    /// Full Unicode case mapping with Ruby string capitalization semantics.
    ///
    /// Georgian Mtavruli capitals are lowercased to Mkhedruli, including the
    /// initial character: `ᲐᲑᲒ` becomes `აბგ`. Modern Georgian uses Mtavruli for
    /// all-caps emphasis, without initial-letter capitalization. This avoids
    /// producing mixed forms such as `Აბგ`.
    ///
    /// This differs from [`to_titlecase(char)`](crate::to_titlecase), which
    /// follows Unicode's character mapping and leaves Mtavruli unchanged.
    /// See [`titlecase`](crate::titlecase()) for an example and rationale.
    ///
    /// See the [Turkic] and [Lithuanian] variants for exceptions.
    ///
    /// Context-dependent case mapping as described in Table 3-14 of the Unicode
    /// standard is currently not supported.
    ///
    /// [Turkic]: Self::Turkic
    /// [Lithuanian]: Self::Lithuanian
    #[default]
    Full,
    /// Only the ASCII region, i.e. the characters `'A'..='Z'` and `'a'..='z'`,
    /// are affected.
    ///
    /// This option cannot be combined with any other option.
    Ascii,
    /// Full Unicode case mapping, adapted for Turkic languages (Turkish,
    /// Azerbaijani, …).
    ///
    /// This means that upper case I is mapped to title case dotless i, and so
    /// on.
    Turkic,
    /// Currently, just [full Unicode case mapping].
    ///
    /// This matches MRI Ruby, which does not implement Lithuanian contextual
    /// case mapping.
    ///
    /// [full Unicode case mapping]: Self::Full
    Lithuanian,
}

impl TryFrom<&str> for TitlecaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}

impl TryFrom<Option<&str>> for TitlecaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        value.map(str::as_bytes).try_into()
    }
}

impl TryFrom<&[u8]> for TitlecaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"ascii" => Ok(Self::Ascii),
            b"turkic" => Ok(Self::Turkic),
            b"lithuanian" => Ok(Self::Lithuanian),
            _ => Err(InvalidCaseMappingMode::new()),
        }
    }
}

impl TryFrom<Option<&[u8]>> for TitlecaseMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&[u8]>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(Self::default()),
            Some(value) => value.try_into(),
        }
    }
}

impl FromStr for TitlecaseMode {
    type Err = InvalidCaseMappingMode;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

/// Returns an iterator that yields a copy of the bytes in the given slice with
/// the leading letter replaced with its titlecase counterpart and all remaining
/// letters replaced with their lowercase counterparts.
///
/// This function treats the given slice as a [conventionally UTF-8 string].
/// The first valid character is titlecased and the remainder is lowercased,
/// following Ruby capitalization semantics, including Georgian Mtavruli.
/// Invalid UTF-8 byte sequences are yielded as is.
///
/// The case mapping mode is determined by the given [`TitlecaseMode`]. See its
/// documentation for details on the available case mapping modes.
///
/// # Georgian capitalization
///
/// Modern Georgian uses Mkhedruli for ordinary text and Mtavruli for all-caps
/// emphasis, without capitalizing only the initial letter. Unicode's character
/// titlecase mappings leave both forms unchanged. Titlecasing an initial
/// Mtavruli character and lowercasing the rest would therefore produce an
/// inappropriate mixed form such as `Აბგ`.
///
/// Like MRI, this function instead lowercases the initial Mtavruli character,
/// producing `აბგ`. This applies to [`TitlecaseMode::Full`],
/// [`TitlecaseMode::Turkic`], and [`TitlecaseMode::Lithuanian`]. The character
/// mapping function [`to_titlecase`] retains Unicode's unchanged Mtavruli mapping.
/// See [Ruby issue #14839](https://bugs.ruby-lang.org/issues/14839) for the
/// rationale and feedback from Georgian speakers.
///
/// ```
/// use roe::{titlecase, to_titlecase, TitlecaseMode};
///
/// let capitalized: Vec<u8> = titlecase("ᲐᲑᲒ".as_bytes(), TitlecaseMode::Full).collect();
/// assert_eq!(capitalized, "აბგ".as_bytes());
/// assert_eq!(to_titlecase('Ა'), ['Ა', '\0', '\0']);
/// ```
///
/// [conventionally UTF-8 string]: https://docs.rs/bstr/1.*/bstr/#when-should-i-use-byte-strings
pub const fn titlecase(slice: &[u8], options: TitlecaseMode) -> Titlecase<'_> {
    match options {
        TitlecaseMode::Full | TitlecaseMode::Lithuanian => Titlecase::with_slice(slice),
        TitlecaseMode::Ascii => Titlecase::with_ascii_slice(slice),
        TitlecaseMode::Turkic => Titlecase::with_mode(slice, unicode::mapping::Mode::TurkicTitle),
    }
}

// Ensure code blocks in README.md compile
//
// This module and macro declaration should be kept at the end of the file, in
// order to not interfere with code coverage.
#[cfg(doctest)]
macro_rules! readme {
    ($x:expr) => {
        #[doc = $x]
        mod readme {}
    };
    () => {
        readme!(include_str!("../README.md"));
    };
}
#[cfg(doctest)]
readme!();

#[cfg(test)]
mod tests {
    use core::{convert::TryInto, str::FromStr};

    use alloc::format;

    use crate::{InvalidCaseMappingMode, LowercaseMode, TitlecaseMode, UppercaseMode};

    #[test]
    fn test_invalid_case_mapping_mode_fmt() {
        let err = InvalidCaseMappingMode::new();
        assert_eq!(format!("{err}"), "invalid option");
        let error: &dyn core::error::Error = &err;
        assert!(error.source().is_none());
    }

    #[test]
    fn test_lowercase_mode_parsing() {
        assert_eq!(LowercaseMode::from_str("ascii"), Ok(LowercaseMode::Ascii));
        assert_eq!(LowercaseMode::from_str("turkic"), Ok(LowercaseMode::Turkic));
        assert_eq!(
            LowercaseMode::from_str("lithuanian"),
            Ok(LowercaseMode::Lithuanian)
        );
        assert_eq!(LowercaseMode::from_str("fold"), Ok(LowercaseMode::Fold));
        assert_eq!(
            LowercaseMode::from_str("full"),
            Err(InvalidCaseMappingMode::new())
        );
    }

    #[test]
    fn test_lowercase_mode_conversion() {
        let mut mode: LowercaseMode;
        mode = "turkic".try_into().unwrap();
        assert_eq!(mode, LowercaseMode::Turkic);

        mode = Some("turkic").try_into().unwrap();
        assert_eq!(mode, LowercaseMode::Turkic);

        mode = b"turkic"[..].try_into().unwrap();
        assert_eq!(mode, LowercaseMode::Turkic);

        mode = Some(&b"turkic"[..]).try_into().unwrap();
        assert_eq!(mode, LowercaseMode::Turkic);
    }

    #[test]
    fn test_uppercase_mode_parsing() {
        assert_eq!(UppercaseMode::from_str("ascii"), Ok(UppercaseMode::Ascii));
        assert_eq!(UppercaseMode::from_str("turkic"), Ok(UppercaseMode::Turkic));
        assert_eq!(
            UppercaseMode::from_str("lithuanian"),
            Ok(UppercaseMode::Lithuanian)
        );
        assert_eq!(
            UppercaseMode::from_str("full"),
            Err(InvalidCaseMappingMode::new())
        );
    }

    #[test]
    fn test_uppercase_mode_conversion() {
        let mut mode: UppercaseMode;
        mode = "turkic".try_into().unwrap();
        assert_eq!(mode, UppercaseMode::Turkic);

        mode = Some("turkic").try_into().unwrap();
        assert_eq!(mode, UppercaseMode::Turkic);

        mode = b"turkic"[..].try_into().unwrap();
        assert_eq!(mode, UppercaseMode::Turkic);

        mode = Some(&b"turkic"[..]).try_into().unwrap();
        assert_eq!(mode, UppercaseMode::Turkic);
    }

    #[test]
    fn test_titlecase_mode_parsing() {
        assert_eq!(TitlecaseMode::from_str("ascii"), Ok(TitlecaseMode::Ascii));
        assert_eq!(TitlecaseMode::from_str("turkic"), Ok(TitlecaseMode::Turkic));
        assert_eq!(
            TitlecaseMode::from_str("lithuanian"),
            Ok(TitlecaseMode::Lithuanian)
        );
        assert_eq!(
            TitlecaseMode::from_str("full"),
            Err(InvalidCaseMappingMode::new())
        );
    }

    #[test]
    fn test_titlecase_mode_conversion() {
        let mut mode: TitlecaseMode;
        mode = "turkic".try_into().unwrap();
        assert_eq!(mode, TitlecaseMode::Turkic);

        mode = Some("turkic").try_into().unwrap();
        assert_eq!(mode, TitlecaseMode::Turkic);

        mode = b"turkic"[..].try_into().unwrap();
        assert_eq!(mode, TitlecaseMode::Turkic);

        mode = Some(&b"turkic"[..]).try_into().unwrap();
        assert_eq!(mode, TitlecaseMode::Turkic);
    }
}
