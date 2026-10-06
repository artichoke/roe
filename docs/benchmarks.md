# Performance benchmarks

## Run

```sh
mkdir -p target/benchmarks
cargo bench --bench case_mapping > target/benchmarks/results.csv
BENCH_FILTER=ascii_long BENCH_MS=50 BENCH_SAMPLES=11 cargo bench --bench case_mapping
```

This dependency-free harness runs on stable Rust. `BENCH_FILTER` selects
workload names by substring. `BENCH_MS` sets the minimum batch duration in
milliseconds (default 20), and `BENCH_SAMPLES` sets the sample count (default
7). Fixture construction happens outside the timed loops. Inputs and results
pass through `std::hint::black_box`; iterators are never boxed.

Each workload warms up while calibrating its iteration count, then records seven
timed batches using that count. CSV output includes median, minimum, and maximum
nanoseconds per call, input bytes, iteration count, and sample count. A batch is
timed as a whole, with no per-call clock reads.

Run benchmarks serially on an otherwise idle machine. Use the same compiler,
dependency resolution, release profile, and environment for comparisons. CPU
frequency, system activity, allocator behavior, and compiler code layout can
change results; these measurements are not a performance guarantee.

## Workloads

Entirely ASCII full-mode inputs select the existing ASCII iterator. Mixed inputs
map ASCII bytes directly after draining buffered Unicode output. Turkic `I`/`i`
mappings that produce non-ASCII UTF-8 retain the Unicode path. Expansions,
Georgian capitalization, and malformed-byte preservation retain their existing
semantics.

The benchmark measures both consuming every output byte into a checksum and
collecting output into a `Vec`. It does not use `count` as a conversion
benchmark: `count` intentionally avoids materializing output in several
implementations.

Keep measurements in pull request descriptions or local artifacts. Benchmark
output belongs under the ignored `target/` directory.

## Unchanged mappings

```sh
cargo bench --bench case_mapping_changes > target/benchmarks/changes.csv
cargo bench --bench case_mapping_allocations > target/benchmarks/allocations.csv
BENCH_FILTER=lower/late_change cargo bench --bench case_mapping_changes
```

These `alloc`-only benchmarks compare the fallible `try_to_*` helpers with
iterator collection followed by byte comparison. They include an
Artichoke-shaped fallible collector that reserves spare NUL capacity, the same
collector without that consumer policy, ordinary `Vec` collection, and an
allocation-free detection pass followed by collection when changed. The
detection experiment restarts the mapping iterator; the Roe helpers continue
from the first differing output byte. A second helper strategy adds spare NUL
capacity to owned results, providing a comparison with the same consumer policy
on both sides. The timing executable also measures the existing ASCII
convenience APIs.

Workloads cover empty, short and long ASCII, already-capitalized text,
punctuation, uncased Unicode, mixed text, expansions, Turkic mappings, malformed
UTF-8, and changes near the beginning and end. Names describe inputs; whether an
input changes depends on the operation. The allocation CSV reports that outcome
and output length explicitly.

Timing and allocation counting use separate executables. Timing uses the normal
allocator without instrumentation. Allocation counts include initial allocations
and reallocations; allocated bytes sum requested sizes, including the full new
size of reallocations, rather than measuring peak or retained memory. Fixtures
and formatting are excluded. The counting executable also checks that unchanged
helper results allocate nothing and that initial reservation and growth failures
are returned without changing the input.

Unchanged mutation can retain its existing buffer. Methods that require a
distinct result object may still need to allocate. Spare NUL capacity and string
encoding remain consumer policies; the Roe helpers do not provide either.
