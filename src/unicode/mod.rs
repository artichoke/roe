mod std_case_mapping_iter;
pub mod titlecase;

#[allow(clippy::all)]
#[allow(clippy::pedantic)]
mod ucd_generated_case_mapping;

pub use titlecase::{to_titlecase, Titlecase, ToTitlecase};
