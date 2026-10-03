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

use core::convert::{TryFrom, TryInto};
use core::str::FromStr;

use crate::InvalidCaseMappingMode;
use crate::unicode::mapping::Mode;

mod ascii;
mod capitalize;
pub(crate) mod georgian;

pub use ascii::make_ascii_capitalize;
#[cfg(feature = "alloc")]
pub use ascii::to_ascii_capitalize;
pub use capitalize::Capitalize;

/// Options to configure the behavior of [`capitalize`].
///
/// Which letters exactly are replaced, and by which other letters, depends on
/// the given options.
///
/// See individual variants for a description of the available behaviors.
///
/// If you're not sure which mode to choose, [`CapitalizeMode::Full`] is a good
/// default.
///
/// [`capitalize`]: crate::ruby::capitalize()
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapitalizeMode {
    /// Full Unicode case mapping with Ruby string capitalization semantics.
    ///
    /// Georgian Mtavruli capitals are lowercased to Mkhedruli, including the
    /// initial character: `ᲐᲑᲒ` becomes `აბგ`. Modern Georgian uses Mtavruli for
    /// all-caps emphasis, without initial-letter capitalization. This avoids
    /// producing mixed forms such as `Აბგ`.
    ///
    /// This differs from [`to_titlecase(char)`](crate::to_titlecase), which
    /// follows Unicode's character mapping and leaves Mtavruli unchanged.
    /// See [`capitalize`](crate::ruby::capitalize()) for an example and rationale.
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
    /// This means that upper case I is mapped to titlecase dotless i, and so
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

impl TryFrom<&str> for CapitalizeMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}

impl TryFrom<Option<&str>> for CapitalizeMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        value.map(str::as_bytes).try_into()
    }
}

impl TryFrom<&[u8]> for CapitalizeMode {
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

impl TryFrom<Option<&[u8]>> for CapitalizeMode {
    type Error = InvalidCaseMappingMode;

    #[inline]
    fn try_from(value: Option<&[u8]>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(Self::default()),
            Some(value) => value.try_into(),
        }
    }
}

impl FromStr for CapitalizeMode {
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
/// The case mapping mode is determined by the given [`CapitalizeMode`]. See its
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
/// producing `აბგ`. This applies to [`CapitalizeMode::Full`],
/// [`CapitalizeMode::Turkic`], and [`CapitalizeMode::Lithuanian`]. The character
/// mapping function [`crate::to_titlecase`] retains Unicode's unchanged Mtavruli
/// mapping.
/// See [Ruby issue #14839](https://bugs.ruby-lang.org/issues/14839) for the
/// rationale and feedback from Georgian speakers.
///
/// ```
/// use roe::ruby::{capitalize, CapitalizeMode};
/// use roe::to_titlecase;
///
/// let capitalized: Vec<u8> = capitalize("ᲐᲑᲒ".as_bytes(), CapitalizeMode::Full).collect();
/// assert_eq!(capitalized, "აბგ".as_bytes());
/// assert_eq!(to_titlecase('Ა'), ['Ა', '\0', '\0']);
/// ```
///
/// [conventionally UTF-8 string]: https://docs.rs/bstr/1.*/bstr/#when-should-i-use-byte-strings
pub const fn capitalize(slice: &[u8], options: CapitalizeMode) -> Capitalize<'_> {
    match options {
        CapitalizeMode::Full | CapitalizeMode::Lithuanian => Capitalize::with_slice(slice),
        CapitalizeMode::Ascii => Capitalize::with_ascii_slice(slice),
        CapitalizeMode::Turkic => Capitalize::with_mode(slice, Mode::TurkicTitle),
    }
}

#[cfg(test)]
mod tests {
    use core::{convert::TryInto, str::FromStr};

    use super::CapitalizeMode;
    use crate::InvalidCaseMappingMode;

    #[test]
    fn test_capitalize_mode_parsing() {
        assert_eq!(CapitalizeMode::from_str("ascii"), Ok(CapitalizeMode::Ascii));
        assert_eq!(
            CapitalizeMode::from_str("turkic"),
            Ok(CapitalizeMode::Turkic)
        );
        assert_eq!(
            CapitalizeMode::from_str("lithuanian"),
            Ok(CapitalizeMode::Lithuanian)
        );
        assert_eq!(
            CapitalizeMode::from_str("full"),
            Err(InvalidCaseMappingMode::new())
        );
    }

    #[test]
    fn test_capitalize_mode_conversion() {
        let mut mode: CapitalizeMode;
        mode = "turkic".try_into().unwrap();
        assert_eq!(mode, CapitalizeMode::Turkic);

        mode = Some("turkic").try_into().unwrap();
        assert_eq!(mode, CapitalizeMode::Turkic);

        mode = b"turkic"[..].try_into().unwrap();
        assert_eq!(mode, CapitalizeMode::Turkic);

        mode = Some(&b"turkic"[..]).try_into().unwrap();
        assert_eq!(mode, CapitalizeMode::Turkic);
    }
}
