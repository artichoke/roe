//! Case mapping for Unicode scalars decoded by the caller.

use core::fmt;
use core::iter::FusedIterator;

use crate::unicode::mapping::{self, Mode};
use crate::unicode::std_case_mapping_iter::CaseMappingIter;
use crate::{LowercaseMode, SwapcaseMode, UppercaseMode};

/// The Unicode scalars produced by mapping one character.
///
/// This iterator owns its state, borrows no input, and does not allocate.
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct CaseMapping(CaseMappingIter);

impl Default for CaseMapping {
    fn default() -> Self {
        Self(CaseMappingIter::Zero)
    }
}

impl Iterator for CaseMapping {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn count(self) -> usize {
        self.len()
    }
}

impl DoubleEndedIterator for CaseMapping {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl ExactSizeIterator for CaseMapping {}
impl FusedIterator for CaseMapping {}

impl fmt::Display for CaseMapping {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Lowercase or fold a Unicode scalar without allocating or encoding it.
///
/// Uses the same mappings as [`crate::lowercase`]. The caller chooses how to
/// decode its input and encode the resulting scalars.
///
/// ```
/// use roe::{LowercaseMode, lowercase_char};
///
/// assert!(lowercase_char('ß', LowercaseMode::Fold).eq(['s', 's']));
/// assert!(lowercase_char('İ', LowercaseMode::Full).eq(['i', '\u{307}']));
/// assert!(lowercase_char('I', LowercaseMode::Turkic).eq(['ı']));
/// ```
pub fn lowercase_char(character: char, mode: LowercaseMode) -> CaseMapping {
    let mode = match mode {
        LowercaseMode::Ascii => {
            return CaseMapping(CaseMappingIter::One(character.to_ascii_lowercase()));
        }
        LowercaseMode::Full | LowercaseMode::Lithuanian => Mode::Lower,
        LowercaseMode::Turkic => Mode::TurkicLower,
        LowercaseMode::Fold => Mode::Fold,
    };
    CaseMapping(mapping::lookup(character, mode))
}

/// Uppercase a Unicode scalar without allocating or encoding it.
///
/// Uses the same mappings as [`crate::uppercase`].
///
/// ```
/// use roe::{UppercaseMode, uppercase_char};
///
/// assert!(uppercase_char('ß', UppercaseMode::Full).eq(['S', 'S']));
/// assert!(uppercase_char('i', UppercaseMode::Turkic).eq(['İ']));
/// ```
pub fn uppercase_char(character: char, mode: UppercaseMode) -> CaseMapping {
    let mode = match mode {
        UppercaseMode::Ascii => {
            return CaseMapping(CaseMappingIter::One(character.to_ascii_uppercase()));
        }
        UppercaseMode::Full | UppercaseMode::Lithuanian => Mode::Upper,
        UppercaseMode::Turkic => Mode::TurkicUpper,
    };
    CaseMapping(mapping::lookup(character, mode))
}

/// Swap a Unicode scalar's case without allocating or encoding it.
///
/// Uses the same mappings as [`crate::swapcase`].
///
/// ```
/// use roe::{SwapcaseMode, swapcase_char};
///
/// assert!(swapcase_char('ǅ', SwapcaseMode::Full).eq(['d', 'Ž']));
/// assert!(swapcase_char('ß', SwapcaseMode::Full).eq(['S', 'S']));
/// ```
pub fn swapcase_char(character: char, mode: SwapcaseMode) -> CaseMapping {
    let mode = match mode {
        SwapcaseMode::Ascii => {
            let mapped = if character.is_ascii_lowercase() {
                character.to_ascii_uppercase()
            } else {
                character.to_ascii_lowercase()
            };
            return CaseMapping(CaseMappingIter::One(mapped));
        }
        SwapcaseMode::Full | SwapcaseMode::Lithuanian => Mode::Swap,
        SwapcaseMode::Turkic => Mode::TurkicSwap,
    };
    CaseMapping(mapping::lookup(character, mode))
}
