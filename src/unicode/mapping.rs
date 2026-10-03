use super::std_case_mapping_iter::CaseMappingIter;
use super::ucd_generated_case_mapping::{FOLD, LOWER, SWAP, TITLE, UPPER};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Mode {
    Lower,
    Upper,
    Title,
    Fold,
    Swap,
    TurkicSwap,
    TurkicLower,
    TurkicUpper,
    TurkicTitle,
}

impl Mode {
    pub(crate) const fn is_turkic(self) -> bool {
        matches!(
            self,
            Self::TurkicLower | Self::TurkicUpper | Self::TurkicTitle | Self::TurkicSwap
        )
    }
}

pub(crate) fn lookup(c: char, mode: Mode) -> CaseMappingIter {
    // MRI applies these language-specific I mappings without contextual casing.
    let special = match (mode, c) {
        (Mode::TurkicLower | Mode::TurkicSwap, 'I') => Some('\u{131}'),
        (Mode::TurkicLower | Mode::TurkicSwap, '\u{130}') => Some('i'),
        (Mode::TurkicUpper | Mode::TurkicTitle | Mode::TurkicSwap, 'i') => Some('\u{130}'),
        _ => None,
    };
    if let Some(c) = special {
        return CaseMappingIter::new([c, '\0', '\0']);
    }
    if c.is_ascii() {
        let c = match mode {
            Mode::Swap | Mode::TurkicSwap if c.is_ascii_lowercase() => c.to_ascii_uppercase(),
            Mode::Lower | Mode::TurkicLower | Mode::Fold | Mode::Swap | Mode::TurkicSwap => {
                c.to_ascii_lowercase()
            }
            Mode::Upper | Mode::TurkicUpper | Mode::Title | Mode::TurkicTitle => {
                c.to_ascii_uppercase()
            }
        };
        return CaseMappingIter::new([c, '\0', '\0']);
    }
    let mode = crate::ruby::georgian::capitalization_mode(c, mode);
    let table = match mode {
        Mode::Lower | Mode::TurkicLower => LOWER,
        Mode::Upper | Mode::TurkicUpper => UPPER,
        Mode::Title | Mode::TurkicTitle => TITLE,
        Mode::Fold => FOLD,
        Mode::Swap | Mode::TurkicSwap => SWAP,
    };
    CaseMappingIter::new(lookup_table(c, table))
}

pub(crate) fn lookup_table(c: char, table: &[(u32, [u32; 3])]) -> [char; 3] {
    if let Ok(index) = table.binary_search_by_key(&(c as u32), |&(key, _)| key) {
        let chars = table[index].1;
        [
            char::from_u32(chars[0]).unwrap_or(c),
            char::from_u32(chars[1]).unwrap_or('\0'),
            char::from_u32(chars[2]).unwrap_or('\0'),
        ]
    } else {
        [c, '\0', '\0']
    }
}
