use crate::unicode::ucd_generated_case_mapping::SORTED_TITLECASE_MAPPING;

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
    super::mapping::lookup_table(c, SORTED_TITLECASE_MAPPING)
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::to_titlecase;
    use crate::unicode::std_case_mapping_iter::CaseMappingIter;

    #[test]
    fn test_char_to_titlecase() {
        assert_eq!(
            CaseMappingIter::new(to_titlecase('ß')).collect::<Vec<_>>(),
            ['S', 's']
        );
        assert_eq!(
            CaseMappingIter::new(to_titlecase('Ǆ')).collect::<Vec<_>>(),
            ['ǅ']
        );
        assert_eq!(
            CaseMappingIter::new(to_titlecase('ﬄ')).collect::<Vec<_>>(),
            ['F', 'f', 'l']
        );
        assert_eq!(
            CaseMappingIter::new(to_titlecase('i')).collect::<Vec<_>>(),
            ['I']
        );
        assert_eq!(
            CaseMappingIter::new(to_titlecase('A')).collect::<Vec<_>>(),
            ['A']
        );
    }

    #[test]
    fn test_next_back() {
        let mut iter = CaseMappingIter::new(to_titlecase('ﬄ'));
        assert_eq!(iter.next_back(), Some('l'));
        assert_eq!(iter.next_back(), Some('f'));
        assert_eq!(iter.next_back(), Some('F'));
        assert_eq!(iter.next_back(), None);
    }
}
