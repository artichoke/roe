use std::hint::black_box;
use std::time::{Duration, Instant};

pub fn measure(name: &str, bytes: usize, mut operation: impl FnMut()) {
    if let Ok(filter) = std::env::var("BENCH_FILTER") {
        if !name.contains(&filter) {
            return;
        }
    }
    let milliseconds = std::env::var("BENCH_MS")
        .map_or(Ok(20), |value| value.parse::<u64>())
        .expect("BENCH_MS must be an integer");
    let samples = std::env::var("BENCH_SAMPLES")
        .map_or(Ok(7), |value| value.parse::<usize>())
        .expect("BENCH_SAMPLES must be an integer");
    assert!(milliseconds > 0 && samples > 0);
    let target = Duration::from_millis(milliseconds);
    let mut iterations = 1_u64;
    // Warm up and calibrate each workload before sampling. Fixture creation and
    // calibration are excluded from the reported times.
    loop {
        let start = Instant::now();
        for _ in 0..iterations {
            operation();
        }
        if start.elapsed() >= target {
            break;
        }
        iterations = iterations.checked_mul(2).expect("iteration overflow");
    }
    let mut timings = Vec::with_capacity(samples);
    for _ in 0..samples {
        let start = Instant::now();
        for _ in 0..iterations {
            operation();
        }
        timings.push(start.elapsed().as_secs_f64() * 1e9 / iterations as f64);
    }
    timings.sort_by(f64::total_cmp);
    println!(
        "{name},{bytes},{:.3},{:.3},{:.3},{iterations},{samples}",
        timings[timings.len() / 2],
        timings[0],
        timings[timings.len() - 1]
    );
    black_box(operation);
}

pub fn header() {
    println!("workload,input_bytes,median_ns,min_ns,max_ns,iterations,samples");
}
