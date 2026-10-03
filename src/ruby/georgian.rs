//! Georgian casing rules for Ruby-style string capitalization.
//!
//! Modern Georgian uses Mkhedruli for ordinary text and Mtavruli for all-caps
//! emphasis and headings. It has no convention of capitalizing only the first
//! letter of a word. Unicode therefore titlecases both forms to themselves.
//!
//! Applying the Unicode titlecase mapping to the first character and lowercasing
//! the rest would turn `ᲐᲑᲒ` into the inappropriate mixed form `Აბგ`. MRI instead
//! lowercases an initial Mtavruli character, producing Mkhedruli throughout:
//! `"ᲐᲑᲒ".capitalize` becomes `აბგ`. Conceptually, capitalization first lowercases
//! the string, then titlecases its first character; Mkhedruli titlecase is identity.
//!
//! This adjustment applies only to Ruby-style string capitalization. The public
//! `to_titlecase(char)` function retains Unicode's character titlecase mapping.
//!
//! See [Ruby issue #14839](https://bugs.ruby-lang.org/issues/14839) for the
//! rationale and feedback from Georgian speakers, and the [Unicode Georgian
//! description](https://www.unicode.org/versions/Unicode16.0.0/core-spec/chapter-7/)
//! for the distinction between uppercase emphasis and titlecasing.

use crate::unicode::mapping::Mode;

/// Select the lowercase mapping for an initial Mtavruli character when
/// capitalizing a string. All other mapping modes and characters are unchanged.
pub(crate) const fn capitalization_mode(c: char, mode: Mode) -> Mode {
    match (mode, c) {
        (Mode::Title | Mode::TurkicTitle, '\u{1c90}'..='\u{1cbf}') => Mode::Lower,
        _ => mode,
    }
}
