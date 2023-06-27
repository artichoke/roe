include!("../../generated/case_mapping.rs");

pub use TITLE as SORTED_TITLECASE_MAPPING;
#[cfg(test)]
mod tests {
    pub use super::TITLE as SORTED_TITLECASE_MAPPING;

    #[test]
    fn test_case_mapping_is_sorted() {
        let mut prev: Option<&u32> = None;
        for (curr, _) in SORTED_TITLECASE_MAPPING {
            if let Some(prev) = prev {
                assert!(curr > prev);
            }
            prev = Some(curr);
        }
    }
}
