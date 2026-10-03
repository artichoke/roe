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

mod capitalize;

#[cfg(feature = "alloc")]
pub use capitalize::to_ascii_capitalize;
pub use capitalize::{Capitalize, CapitalizeMode, capitalize, make_ascii_capitalize};
