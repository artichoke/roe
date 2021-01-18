#![no_std]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use core::convert::{TryFrom, TryInto};
use core::fmt;
use core::str::FromStr;

mod lowercase;
mod uppercase;

pub use lowercase::Lowercase;
pub use uppercase::Uppercase;

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidLowercaseMode {
    _private: (),
}

impl InvalidLowercaseMode {
    /// Construct a new `InvalidLowercaseMode` error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::InvalidLowercaseMode;
    /// const ERR: InvalidLowercaseMode = InvalidLowercaseMode::new();
    /// assert_eq!(ERR.message(), "invalid option");
    /// ```
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Retrieve the error message associated with this `InvalidLowercaseMode`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roe::InvalidLowercaseMode;
    /// const MESSAGE: &str = InvalidLowercaseMode::new().message();
    /// assert_eq!(MESSAGE, "invalid option");
    /// ```
    #[allow(clippy::clippy::unused_self)]
    pub const fn message(self) -> &'static str {
        "invalid option"
    }
}

impl fmt::Display for InvalidLowercaseMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const MESSAGE: &str = InvalidLowercaseMode::new().message();
        f.write_str(MESSAGE)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidLowercaseMode {}

/// Options to configure the behavior of [`lowercase`].
///
/// Which letters exactly are replaced, and by which other letters, depends on
/// the given options.
///
/// See individual variants for a description of the available behaviors.
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

impl TryFrom<Option<&str>> for LowercaseMode {
    type Error = InvalidLowercaseMode;

    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        value.map(str::as_bytes).try_into()
    }
}

impl TryFrom<Option<&[u8]>> for LowercaseMode {
    type Error = InvalidLowercaseMode;

    fn try_from(value: Option<&[u8]>) -> Result<Self, Self::Error> {
        match value {
            None => Ok(Self::Full),
            Some(b"ascii") => Ok(Self::Ascii),
            Some(b"turkic") => Ok(Self::Turkic),
            Some(b"lithuanian") => Ok(Self::Lithuanian),
            Some(b"fold") => Ok(Self::Fold),
            Some(_) => Err(InvalidLowercaseMode::new()),
        }
    }
}

impl FromStr for LowercaseMode {
    type Err = InvalidLowercaseMode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Some(s).try_into()
    }
}

// Returns an iterator that yields a copy of the bytes in the given slice with
// all uppercase letters replaced with their lowercase counterparts.
//
// This function treats the given slice as a conventionally UTF-8 string. UTF-8
// byte sequences are converted to their Unicode lowercase equivalents. Invalid
// UTF-8 byte sequences are yielded as is.
//
// The case mapping mode is determined by the given [`LowercaseMode`]. See its
// documentation for details on the available case mapping modes.
pub const fn lowercase(slice: &[u8], options: LowercaseMode) -> Lowercase<'_> {
    match options {
        LowercaseMode::Full | LowercaseMode::Lithuanian => Lowercase::with_slice(slice),
        LowercaseMode::Ascii => Lowercase::with_ascii_slice(slice),
        // TODO: implement `turkic` and `fold` modes.
        LowercaseMode::Turkic | LowercaseMode::Fold => Lowercase::new(),
    }
}

// Returns an iterator that yields a copy of the bytes in the given slice with
// all lowercase letters replaced with their uppercase counterparts.
//
// This function treats the given slice as a conventionally UTF-8 string. UTF-8
// byte sequences are converted to their Unicode uppercase equivalents. Invalid
// UTF-8 byte sequences are yielded as is.
pub const fn uppercase(slice: &[u8]) -> Uppercase<'_> {
    Uppercase::with_slice(slice)
}
