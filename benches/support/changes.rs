use std::borrow::Cow;
use std::collections::TryReserveError;

use roe::ruby::CapitalizeMode as C;
use roe::{LowercaseMode as L, SwapcaseMode as S, UppercaseMode as U};

#[derive(Clone, Copy)]
pub enum Strategy {
    // Artichoke reserves spare NUL capacity before comparing the output.
    Consumer,
    // Isolate the collector from that consumer-specific capacity policy.
    Fallible,
    Collect,
    // Detect changes with an iterator clone, then restart only if changed.
    Prescan,
    // Roe continues the iterator from its first differing output byte.
    Lazy,
    // Apply the same spare-NUL policy as Consumer, only to changed output.
    LazyConsumer,
}

fn collect(iter: impl Iterator<Item = u8>, capacity: usize) -> Result<Vec<u8>, TryReserveError> {
    let mut out = Vec::new();
    out.try_reserve(capacity)?;
    for byte in iter {
        if out.len() == out.capacity() {
            out.try_reserve(1)?;
        }
        out.push(byte);
    }
    Ok(out)
}

fn run<'a>(
    input: &'a [u8],
    iter: impl Iterator<Item = u8> + Clone,
    strategy: Strategy,
) -> Cow<'a, [u8]> {
    match strategy {
        Strategy::Consumer => {
            let mut out = collect(iter, input.len()).unwrap();
            out.try_reserve(1).unwrap();
            if out == input {
                Cow::Borrowed(input)
            } else {
                Cow::Owned(out)
            }
        }
        Strategy::Fallible => {
            let out = collect(iter, input.len()).unwrap();
            if out == input {
                Cow::Borrowed(input)
            } else {
                Cow::Owned(out)
            }
        }
        Strategy::Collect => {
            let out: Vec<_> = iter.collect();
            if out == input {
                Cow::Borrowed(input)
            } else {
                Cow::Owned(out)
            }
        }
        Strategy::Prescan => {
            if iter.clone().eq(input.iter().copied()) {
                Cow::Borrowed(input)
            } else {
                Cow::Owned(collect(iter, input.len()).unwrap())
            }
        }
        Strategy::Lazy | Strategy::LazyConsumer => {
            unreachable!("handled before constructing an iterator")
        }
    }
}

#[derive(Clone, Copy)]
pub enum Operation {
    Lower(L),
    Upper(U),
    Swap(S),
    Cap(C),
}

pub fn apply(input: &[u8], op: Operation, s: Strategy) -> Cow<'_, [u8]> {
    if matches!(s, Strategy::Lazy | Strategy::LazyConsumer) {
        let mut result = match op {
            Operation::Lower(m) => roe::try_to_lowercase(input, m).unwrap(),
            Operation::Upper(m) => roe::try_to_uppercase(input, m).unwrap(),
            Operation::Swap(m) => roe::try_to_swapcase(input, m).unwrap(),
            Operation::Cap(m) => roe::ruby::try_to_capitalize(input, m).unwrap(),
        };
        if let (Strategy::LazyConsumer, Cow::Owned(bytes)) = (s, &mut result) {
            bytes.try_reserve(1).unwrap();
        }
        return result;
    }
    match op {
        Operation::Lower(m) => run(input, roe::lowercase(input, m), s),
        Operation::Upper(m) => run(input, roe::uppercase(input, m), s),
        Operation::Swap(m) => run(input, roe::swapcase(input, m), s),
        Operation::Cap(m) => run(input, roe::ruby::capitalize(input, m), s),
    }
}

pub struct Case {
    pub name: String,
    pub input: Vec<u8>,
    pub operation: Operation,
    pub strategy: Strategy,
}

impl Case {
    pub fn run(&self) -> Cow<'_, [u8]> {
        apply(
            std::hint::black_box(&self.input),
            self.operation,
            self.strategy,
        )
    }
}

pub fn cases() -> Vec<Case> {
    let inputs = [
        ("empty", Vec::new()),
        ("ascii_lower_short", b"artichoke".to_vec()),
        ("ascii_upper_short", b"ARTICHOKE".to_vec()),
        ("capitalized_short", b"Artichoke".to_vec()),
        ("punctuation_short", b"123!?!".to_vec()),
        ("ascii_lower_long", b"artichoke ruby 123! ".repeat(256)),
        ("ascii_upper_long", b"ARTICHOKE RUBY 123! ".repeat(256)),
        (
            "early_change",
            [b"A".to_vec(), b"artichoke ruby 123! ".repeat(256)].concat(),
        ),
        (
            "late_change",
            [b"artichoke ruby 123! ".repeat(256), b"A".to_vec()].concat(),
        ),
        ("uncased_short", "東京🦀".as_bytes().to_vec()),
        ("uncased_long", "東京 🦀 123! ".repeat(256).into_bytes()),
        ("mixed", "Straße Αύριο 東京 🦀 ".repeat(128).into_bytes()),
        ("expansions", "İßﬃǅΐᾀ".repeat(128).into_bytes()),
        ("turkic", "Iİiı 123! ".repeat(128).into_bytes()),
        ("invalid", b"\xffartichoke\xfe\xf0\x9f\x87".repeat(256)),
    ];
    let operations = [
        ("lower", Operation::Lower(L::Full)),
        ("lower_ascii", Operation::Lower(L::Ascii)),
        ("upper", Operation::Upper(U::Full)),
        ("fold", Operation::Lower(L::Fold)),
        ("lower_turkic", Operation::Lower(L::Turkic)),
        ("upper_turkic", Operation::Upper(U::Turkic)),
        ("swap", Operation::Swap(S::Full)),
        ("capitalize", Operation::Cap(C::Full)),
    ];
    let mut cases = Vec::new();
    for (opname, operation) in operations {
        for (name, input) in &inputs {
            for (sname, strategy) in [
                ("consumer", Strategy::Consumer),
                ("fallible", Strategy::Fallible),
                ("collect", Strategy::Collect),
                ("prescan", Strategy::Prescan),
                ("lazy", Strategy::Lazy),
                ("lazy_consumer", Strategy::LazyConsumer),
            ] {
                cases.push(Case {
                    name: format!("{opname}/{name}/{sname}"),
                    input: input.clone(),
                    operation,
                    strategy,
                });
            }
        }
    }
    cases
}
