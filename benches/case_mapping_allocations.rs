#[path = "support/changes.rs"]
mod changes;

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
// Only explicit failure checks arm this counter; allocation measurements do not.
static FAIL_ON: AtomicUsize = AtomicUsize::new(usize::MAX);

// Instrument only this standalone benchmark executable. Roe itself forbids
// unsafe code. Every allocator operation delegates to System with unchanged
// pointers and layouts; the counters do not allocate or dereference pointers.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let allocation = ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        if allocation == FAIL_ON.load(Ordering::Relaxed) {
            return std::ptr::null_mut();
        }
        BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        // SAFETY: forward the caller's valid allocation layout to System.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: allocations originate from System; forward the same pointer
        // and allocation layout supplied by the caller.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let allocation = ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        if allocation == FAIL_ON.load(Ordering::Relaxed) {
            return std::ptr::null_mut();
        }
        BYTES.fetch_add(new_size, Ordering::Relaxed);
        // SAFETY: preserve the caller's pointer, layout, and valid new size.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn main() {
    println!("workload,input_bytes,output_bytes,outcome,allocations,allocated_bytes");
    check_allocation_failures();
    for case in changes::cases() {
        if let Ok(filter) = std::env::var("BENCH_FILTER") {
            if !case.name.contains(&filter) {
                continue;
            }
        }
        let allocations = ALLOCATIONS.load(Ordering::Relaxed);
        let bytes = BYTES.load(Ordering::Relaxed);
        let output = black_box(case.run());
        let allocations = ALLOCATIONS.load(Ordering::Relaxed) - allocations;
        let bytes = BYTES.load(Ordering::Relaxed) - bytes;
        if matches!(
            case.strategy,
            changes::Strategy::Lazy | changes::Strategy::LazyConsumer
        ) && output.as_ref() == case.input
        {
            assert_eq!(allocations, 0);
            assert_eq!(bytes, 0);
        }
        let outcome = if output.as_ref() == case.input {
            "unchanged"
        } else {
            "changed"
        };
        println!(
            "{},{},{},{outcome},{allocations},{bytes}",
            case.name,
            case.input.len(),
            output.len()
        );
    }
}

fn check_allocation_failures() {
    use roe::{LowercaseMode, UppercaseMode};

    let input = "ΐ".repeat(8).into_bytes();
    let original = input.clone();
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);
    FAIL_ON.store(allocations, Ordering::Relaxed);
    // Unchanged input succeeds even when the next allocation would fail.
    assert!(matches!(
        roe::try_to_lowercase(b"artichoke", LowercaseMode::Full).unwrap(),
        std::borrow::Cow::Borrowed(_)
    ));
    assert_eq!(ALLOCATIONS.load(Ordering::Relaxed), allocations);
    // Initial reservation failure is returned, rather than aborting or modifying
    // the input. The allocator resumes normally after this single rejection.
    assert!(roe::try_to_uppercase(&input, UppercaseMode::Full).is_err());
    assert_eq!(input, original);
    // This expansion needs a second reservation; fail growth after the first
    // successful allocation, and verify the same fallible contract.
    FAIL_ON.store(ALLOCATIONS.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
    assert!(roe::try_to_uppercase(&input, UppercaseMode::Full).is_err());
    assert_eq!(input, original);
    FAIL_ON.store(usize::MAX, Ordering::Relaxed);
}
