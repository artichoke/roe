#[path = "support/changes.rs"]
mod changes;
mod support;

use std::hint::black_box;

fn main() {
    support::header();
    for case in changes::cases() {
        support::measure(&case.name, case.input.len(), || {
            black_box(case.run());
        });
    }
    // Existing ASCII convenience APIs always create a distinct byte vector.
    for (name, input) in [
        ("ascii_short", b"artichoke".to_vec()),
        ("ascii_long", b"artichoke ruby 123! ".repeat(256)),
    ] {
        support::measure(&format!("convenience/{name}/lower"), input.len(), || {
            black_box(roe::to_ascii_lowercase(black_box(&input)));
        });
        support::measure(&format!("convenience/{name}/upper"), input.len(), || {
            black_box(roe::to_ascii_uppercase(black_box(&input)));
        });
    }
}
