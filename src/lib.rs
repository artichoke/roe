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
//! # use roe::{LowercaseMode, UppercaseMode};
//! # use roe::ruby::{capitalize, CapitalizeMode};
//! assert_eq!(
//!     roe::lowercase(b"Artichoke Ruby", LowercaseMode::Ascii).collect::<Vec<_>>(),
//!     b"artichoke ruby"
//! );
//! assert_eq!(
//!     roe::uppercase("Αύριο".as_bytes(), UppercaseMode::Full).collect::<Vec<_>>(),
//!     "ΑΎΡΙΟ".as_bytes()
//! );
//! assert_eq!(
//!     capitalize("ﬃ".as_bytes(), CapitalizeMode::Full).collect::<Vec<_>>(),
//!     "Ffi".as_bytes()
//! );
//! ```
//!
//!
//! Roe provides fast path routines that assume the byte slice is ASCII-only.
//!
//! To detect a change without allocating, compare the mapped byte iterator with
//! the original bytes:
//!
//! ```
//! # use roe::{LowercaseMode, lowercase};
//! let input = b"artichoke";
//! assert!(lowercase(input, LowercaseMode::Full).eq(input.iter().copied()));
//! ```
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
#![doc(html_root_url = "https://docs.rs/roe/0.0.10")]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

use core::fmt;

#[cfg(feature = "alloc")]
mod collect;
mod lowercase;
pub mod ruby;
mod swapcase;
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

pub use lowercase::{Lowercase, LowercaseMode, lowercase, make_ascii_lowercase};
#[cfg(feature = "alloc")]
pub use lowercase::{to_ascii_lowercase, try_to_lowercase};
pub use swapcase::{Swapcase, SwapcaseMode, make_ascii_swapcase, swapcase};
#[cfg(feature = "alloc")]
pub use swapcase::{to_ascii_swapcase, try_to_swapcase};
pub use unicode::UNICODE_VERSION;
pub use unicode::to_titlecase;
pub use uppercase::{Uppercase, UppercaseMode, make_ascii_uppercase, uppercase};
#[cfg(feature = "alloc")]
pub use uppercase::{to_ascii_uppercase, try_to_uppercase};

/// Error that indicates a failure to parse a [`LowercaseMode`],
/// [`UppercaseMode`], [`SwapcaseMode`], or [`ruby::CapitalizeMode`].
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
    use alloc::format;

    use crate::InvalidCaseMappingMode;

    #[test]
    fn test_invalid_case_mapping_mode_fmt() {
        let err = InvalidCaseMappingMode::new();
        assert_eq!(format!("{err}"), "invalid option");
        let error: &dyn core::error::Error = &err;
        assert!(error.source().is_none());
    }
}
