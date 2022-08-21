#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![cfg_attr(test, allow(clippy::non_ascii_literal))]
#![cfg_attr(test, allow(clippy::shadow_unrelated))]
#![warn(clippy::cargo)]
#![allow(unknown_lints)]
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
#![cfg_attr(docsrs, feature(doc_alias))]

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
//! This crate is currently a *work in progress*. When the API is complete, Roe
//! will support lowercase, uppercase, titlecase, and case folding iterators for
//! conventionally UTF-8 byte slices.
//!
//! Roe will implement support for full, Turkic, ASCII, and case folding
//! transforms.
//!
//! # Usage
//!
//! You can convert case like:
//!
//! ```
//! # use roe::{LowercaseMode, UppercaseMode};
//! assert_eq!(
//!     roe::lowercase(b"Artichoke Ruby", LowercaseMode::Ascii).collect::<Vec<_>>(),
//!     b"artichoke ruby"
//! );
//! assert_eq!(
//!     roe::uppercase("Αύριο".as_bytes(), UppercaseMode::Full).collect::<Vec<_>>(),
//!     "ΑΎΡΙΟ".as_bytes()
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
//! Roe has several Cargo features, all of which are enabled by default:
//!
//! - **std** - Adds a dependency on [`std`], the Rust Standard Library. This
//!   feature enables [`std::error::Error`] implementations on error types in
//!   this crate. Enabling the **std** feature also enables the **alloc**
//!   feature.
//! - **alloc** - Adds a dependency on [`alloc`], the Rust allocation and
//!   collections library. This feature enables APIs that allocate [`String`] or
//!   [`Vec`].
//!
#![cfg_attr(
    not(feature = "std"),
    doc = "[`std`]: https://doc.rust-lang.org/std/index.html"
)]
#![cfg_attr(
    not(feature = "std"),
    doc = "[`std::error::Error`]: https://doc.rust-lang.org/std/error/trait.Error.html"
)]
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
//! [conventionally UTF-8 binary strings]: https://docs.rs/bstr/0.2.*/bstr/#when-should-i-use-byte-strings

#![no_std]
#![doc(html_root_url = "https://docs.rs/roe/0.0.3")]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use core::convert::{TryFrom, TryInto};
use core::fmt;
use core::str::FromStr;

mod ascii;
mod lowercase;
mod uppercase;

pub use ascii::{make_ascii_lowercase, make_ascii_titlecase, make_ascii_uppercase};
#[cfg(feature = "alloc")]
pub use ascii::{to_ascii_lowercase, to_ascii_titlecase, to_ascii_uppercase};
pub use lowercase::Lowercase;
pub use uppercase::Uppercase;

/// Error that indicates a failure to parse a [`LowercaseMode`] or
/// [`UppercaseMode`].
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

#[cfg(feature = "std")]
impl std::error::Error for InvalidCaseMappingMode {}

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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
    /// In the future, full Unicode case mapping adapted for Lithuanian (keeping
    /// the dot on the lower case i even if there is an accent on top).
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

impl Default for LowercaseMode {
    fn default() -> Self {
        Self::Full
    }
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
/// # Panics
///
/// Not all [`LowercaseMode`]s are currently implemented. This function will
/// panic if the caller supplies [Turkic] or [case folding] lowercasing mode.
///
/// [conventionally UTF-8 string]: https://docs.rs/bstr/0.2.*/bstr/#when-should-i-use-byte-strings
/// [Turkic]: LowercaseMode::Turkic
/// [case folding]: LowercaseMode::Fold
// TODO: make this const once we're no longer panicking.
pub fn lowercase(slice: &[u8], options: LowercaseMode) -> Lowercase<'_> {
    match options {
        LowercaseMode::Full | LowercaseMode::Lithuanian => Lowercase::with_slice(slice),
        LowercaseMode::Ascii => Lowercase::with_ascii_slice(slice),
        // TODO: implement `turkic` and `fold` modes.
        LowercaseMode::Turkic => panic!("lowercase Turkic mode is not yet implemented"),
        LowercaseMode::Fold => panic!("lowercase case folding mode is not yet implemented"),
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
    /// In the future, full Unicode case mapping adapted for Lithuanian (keeping
    /// the dot on the lower case i even if there is an accent on top).
    ///
    /// [full Unicode case mapping]: Self::Full
    Lithuanian,
}

impl Default for UppercaseMode {
    fn default() -> Self {
        Self::Full
    }
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
/// # Panics
///
/// Not all [`UppercaseMode`]s are currently implemented. This function will
/// panic if the caller supplies [Turkic] uppercasing mode.
///
/// [conventionally UTF-8 string]: https://docs.rs/bstr/0.2.*/bstr/#when-should-i-use-byte-strings
/// [Turkic]: LowercaseMode::Turkic
/// [case folding]: LowercaseMode::Fold
// TODO: make this const once we're no longer panicking.
pub fn uppercase(slice: &[u8], options: UppercaseMode) -> Uppercase<'_> {
    match options {
        UppercaseMode::Full | UppercaseMode::Lithuanian => Uppercase::with_slice(slice),
        UppercaseMode::Ascii => Uppercase::with_ascii_slice(slice),
        // TODO: implement `turkic` mode.
        UppercaseMode::Turkic => panic!("uppercase Turkic mode is not yet implemented"),
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
