//! Case mapping with Ruby string semantics.
//!
//! [`capitalize`] titlecases the first valid UTF-8 character and lowercases the
//! remainder, matching Ruby's `String#capitalize`. This is distinct from Unicode
//! character titlecase, provided by [`crate::to_titlecase`], and from titlecasing
//! each word in a string.
//!
//! These byte-stream APIs preserve malformed UTF-8. Ruby encoding validation,
//! argument handling, and bang-method return values belong to the interpreter.
//!
//! # Georgian capitalization
//!
//! Modern Georgian uses Mkhedruli for ordinary text and Mtavruli for all-caps
//! emphasis, without initial-letter capitalization. Unicode character titlecase
//! leaves both forms unchanged. Ruby capitalization lowercases initial Mtavruli
//! too, avoiding mixed forms such as `Აბგ` and producing `აბგ` instead.
//! See [Ruby issue #14839](https://bugs.ruby-lang.org/issues/14839) for the rationale.

pub(crate) mod georgian;

/// Byte iterator for Ruby-style string capitalization.
pub use crate::Titlecase as Capitalize;
/// Options for Ruby-style string capitalization.
pub use crate::TitlecaseMode as CapitalizeMode;

/// Capitalize the first valid UTF-8 character and lowercase the remainder.
///
/// Full, Turkic, and Lithuanian modes follow Ruby's Georgian capitalization
/// rule. ASCII mode only changes ASCII letters. Malformed UTF-8 is preserved.
///
/// ```
/// use roe::ruby::{capitalize, CapitalizeMode};
///
/// assert_eq!(capitalize(b"hELLO wORLD", CapitalizeMode::Full).collect::<Vec<_>>(), b"Hello world");
/// assert_eq!(capitalize("ᲐᲑᲒ".as_bytes(), CapitalizeMode::Full).collect::<Vec<_>>(), "აბგ".as_bytes());
/// assert_eq!(roe::to_titlecase('Ა'), ['Ა', '\0', '\0']);
/// ```
pub const fn capitalize(slice: &[u8], mode: CapitalizeMode) -> Capitalize<'_> {
    crate::titlecase(slice, mode)
}
