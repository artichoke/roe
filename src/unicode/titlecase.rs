use crate::unicode::std_case_mapping_iter::CaseMappingIter;
use crate::unicode::ucd_generated_case_mapping::SORTED_TITLECASE_MAPPING;
use core::iter::FusedIterator;

/// Take a [`char`] and return its Unicode titlecase as 3 `char`s.
///
/// Trailing NUL bytes in the returned array should be ignored.
///
/// # Examples
///
/// ```
/// use roe::to_titlecase;
///
/// assert_eq!(to_titlecase('Ǆ'), ['ǅ', '\0', '\0']);
///
/// // Ligatures
/// assert_eq!(to_titlecase('ﬄ'), ['F', 'f', 'l']);
///
/// // Locale is ignored
/// assert_eq!(to_titlecase('i'), ['I', '\0', '\0']);
///
/// // A character already titlecased map to itself
/// assert_eq!(to_titlecase('A'), ['A', '\0', '\0']);
/// ```
#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn to_titlecase(c: char) -> [char; 3] {
    let codepoint = c as u32;
    if let Ok(index) = SORTED_TITLECASE_MAPPING.binary_search_by(|&(key, _)| key.cmp(&codepoint)) {
        let chars = SORTED_TITLECASE_MAPPING[index].1;
        [
            char::from_u32(chars[0]).unwrap_or(c),
            char::from_u32(chars[1]).unwrap_or('\0'),
            char::from_u32(chars[2]).unwrap_or('\0'),
        ]
    } else {
        [c, '\0', '\0']
    }
}

/// Returns an iterator that yields the titlecase equivalent of a `char`.
///
/// This `struct` is created by the [`to_titlecase`] method.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct ToTitlecase(CaseMappingIter);

impl Iterator for ToTitlecase {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for ToTitlecase {
    fn next_back(&mut self) -> Option<char> {
        self.0.next_back()
    }
}

impl FusedIterator for ToTitlecase {}

impl ExactSizeIterator for ToTitlecase {}

pub trait Titlecase {
    fn to_titlecase(self) -> ToTitlecase;
}

impl Titlecase for char {
    fn to_titlecase(self) -> ToTitlecase {
        ToTitlecase(CaseMappingIter::new(to_titlecase(self)))
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use crate::unicode::titlecase::Titlecase;

    #[test]
    fn test_char_to_titlecase() {
        assert_eq!('ß'.to_titlecase().collect::<Vec<_>>(), ['S', 's']);
        assert_eq!('Ǆ'.to_titlecase().collect::<Vec<_>>(), ['ǅ']);
        assert_eq!('ﬄ'.to_titlecase().collect::<Vec<_>>(), ['F', 'f', 'l']);
        assert_eq!('i'.to_titlecase().collect::<Vec<_>>(), ['I']);
        assert_eq!('A'.to_titlecase().collect::<Vec<_>>(), ['A']);
    }

    #[test]
    fn test_next_back() {
        let mut iter = 'ﬄ'.to_titlecase();
        assert_eq!(iter.next_back(), Some('l'));
        assert_eq!(iter.next_back(), Some('f'));
        assert_eq!(iter.next_back(), Some('F'));
        assert_eq!(iter.next_back(), None);
    }
}
