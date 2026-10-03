mod support;

use std::hint::black_box;

use roe::ruby::{CapitalizeMode, capitalize};
use roe::{LowercaseMode, SwapcaseMode, UppercaseMode, lowercase, swapcase, uppercase};

#[derive(Clone, Copy)]
enum Operation {
    Lower(LowercaseMode),
    Upper(UppercaseMode),
    Swap(SwapcaseMode),
    Capitalize(CapitalizeMode),
}

fn consume(iter: impl Iterator<Item = u8>) -> usize {
    black_box(iter.fold(0_usize, |sum, byte| sum.wrapping_add(usize::from(byte))))
}

fn collect(iter: impl Iterator<Item = u8>) -> usize {
    black_box(iter.collect::<Vec<_>>()).len()
}

#[derive(Clone, Copy)]
enum Consumer {
    Stream,
    Collect,
}

impl Consumer {
    fn apply(self, iter: impl Iterator<Item = u8>) -> usize {
        match self {
            Self::Stream => consume(iter),
            Self::Collect => collect(iter),
        }
    }
}

fn main() {
    support::header();
    let inputs = [
        ("empty", Vec::new()),
        ("ruby_name", b"Artichoke::String#casecmp?".to_vec()),
        (
            "ascii_long",
            b"The Quick Brown FOX jumps over the lazy DOG! 0123456789\n".repeat(64),
        ),
        (
            "mixed",
            "Artichoke Ruby: Straße Αύριο 東京 🦀\n"
                .repeat(32)
                .into_bytes(),
        ),
        ("expansions", "İßﬃǅΐᾀ".repeat(64).into_bytes()),
        ("turkic", "iIİı Istanbul İSTANBUL\n".repeat(32).into_bytes()),
        ("invalid", b"AbC\xff\xfeI\xf0\x9f\x87xyz\0".repeat(64)),
    ];
    let operations = [
        ("lower_full", Operation::Lower(LowercaseMode::Full)),
        ("lower_ascii", Operation::Lower(LowercaseMode::Ascii)),
        ("lower_turkic", Operation::Lower(LowercaseMode::Turkic)),
        ("fold", Operation::Lower(LowercaseMode::Fold)),
        ("upper_full", Operation::Upper(UppercaseMode::Full)),
        ("upper_turkic", Operation::Upper(UppercaseMode::Turkic)),
        ("swap_full", Operation::Swap(SwapcaseMode::Full)),
        ("swap_turkic", Operation::Swap(SwapcaseMode::Turkic)),
        (
            "capitalize_full",
            Operation::Capitalize(CapitalizeMode::Full),
        ),
        (
            "capitalize_turkic",
            Operation::Capitalize(CapitalizeMode::Turkic),
        ),
    ];
    for (input_name, input) in &inputs {
        for &(operation_name, operation) in &operations {
            for (consumer_name, consumer) in
                [("stream", Consumer::Stream), ("collect", Consumer::Collect)]
            {
                let name = format!("{operation_name}/{input_name}/{consumer_name}");
                support::measure(&name, input.len(), || {
                    let input = black_box(input.as_slice());
                    // Measure production dispatch without boxing the iterators.
                    // Stream and collect use identical fixtures.
                    let result = match operation {
                        Operation::Lower(mode) => consumer.apply(lowercase(input, black_box(mode))),
                        Operation::Upper(mode) => consumer.apply(uppercase(input, black_box(mode))),
                        Operation::Swap(mode) => consumer.apply(swapcase(input, black_box(mode))),
                        Operation::Capitalize(mode) => {
                            consumer.apply(capitalize(input, black_box(mode)))
                        }
                    };
                    black_box(result);
                });
            }
        }
    }
}
