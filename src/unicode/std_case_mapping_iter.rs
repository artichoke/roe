// This file contain code extracted from std::char::CaseMappingIter.
use core::fmt::{Debug, Display, Formatter, Result, Write};

#[derive(Debug, Clone)]
pub enum CaseMappingIter {
    Three(char, char, char),
    Two(char, char),
    One(char),
    Zero,
}

impl CaseMappingIter {
    pub fn new(chars: [char; 3]) -> CaseMappingIter {
        if chars[2] == '\0' {
            if chars[1] == '\0' {
                CaseMappingIter::One(chars[0]) // Including if chars[0] == '\0'
            } else {
                CaseMappingIter::Two(chars[0], chars[1])
            }
        } else {
            CaseMappingIter::Three(chars[0], chars[1], chars[2])
        }
    }
}

impl Iterator for CaseMappingIter {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        match *self {
            CaseMappingIter::Three(a, b, c) => {
                *self = CaseMappingIter::Two(b, c);
                Some(a)
            }
            CaseMappingIter::Two(b, c) => {
                *self = CaseMappingIter::One(c);
                Some(b)
            }
            CaseMappingIter::One(c) => {
                *self = CaseMappingIter::Zero;
                Some(c)
            }
            CaseMappingIter::Zero => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = match self {
            CaseMappingIter::Three(..) => 3,
            CaseMappingIter::Two(..) => 2,
            CaseMappingIter::One(_) => 1,
            CaseMappingIter::Zero => 0,
        };
        (size, Some(size))
    }
}

impl DoubleEndedIterator for CaseMappingIter {
    fn next_back(&mut self) -> Option<char> {
        match *self {
            CaseMappingIter::Three(a, b, c) => {
                *self = CaseMappingIter::Two(a, b);
                Some(c)
            }
            CaseMappingIter::Two(b, c) => {
                *self = CaseMappingIter::One(b);
                Some(c)
            }
            CaseMappingIter::One(c) => {
                *self = CaseMappingIter::Zero;
                Some(c)
            }
            CaseMappingIter::Zero => None,
        }
    }
}

impl Display for CaseMappingIter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
            CaseMappingIter::Three(a, b, c) => {
                f.write_char(a)?;
                f.write_char(b)?;
                f.write_char(c)
            }
            CaseMappingIter::Two(b, c) => {
                f.write_char(b)?;
                f.write_char(c)
            }
            CaseMappingIter::One(c) => f.write_char(c),
            CaseMappingIter::Zero => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{format, vec::Vec};

    use super::CaseMappingIter;

    #[test]
    fn test_case_mapping_iter() {
        fn test_iterator(iter: CaseMappingIter, expected: &[char], expected_rev: &[char]) {
            assert_eq!(iter.size_hint(), (expected.len(), Some(expected.len())));
            assert_eq!(iter.clone().collect::<Vec<_>>(), expected);
            assert_eq!(iter.rev().collect::<Vec<_>>(), expected_rev);
        }
        test_iterator(
            CaseMappingIter::new(['F', 'f', 'l']),
            &['F', 'f', 'l'],
            &['l', 'f', 'F'],
        );
        test_iterator(
            CaseMappingIter::new(['S', 's', '\0']),
            &['S', 's'],
            &['s', 'S'],
        );
        test_iterator(CaseMappingIter::new(['A', '\0', '\0']), &['A'], &['A']);
        test_iterator(CaseMappingIter::new(['\0', '\0', '\0']), &['\0'], &['\0']);
    }

    #[test]
    fn test_fmt() {
        let zero = CaseMappingIter::Zero;
        assert_eq!(format!("{zero}"), "");

        let one = CaseMappingIter::One('A');
        assert_eq!(format!("{one}"), "A");

        let two = CaseMappingIter::Two('S', 's');
        assert_eq!(format!("{two}"), "Ss");

        let three = CaseMappingIter::Three('F', 'f', 'l');
        assert_eq!(format!("{three}"), "Ffl");
    }
}
