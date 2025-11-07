// Simple Allocator Performance Benchmarks
//
// Reduced iterations for initial performance testing

// Only compile this benchmark if the allocator was successfully built
#![cfg(all(target_arch = "x86_64", not(target_env = "msvc")))]

use std::time::{Duration, Instant};
use glimmer_weave::native_allocator::*;

/// Benchmark result containing timing and memory statistics
#[derive(Debug)]
struct BenchResult {
    name: String,
    duration: Duration,
    operations: usize,
    ops_per_sec: f64,
}

impl BenchResult {
    fn print(&self) {
        println!("\n{}", "=".repeat(70));
        println!("Benchmark: {}", self.name);
        println!("{}", "-".repeat(70));
        println!("  Duration:       {:?}", self.duration);
        println!("  Operations:     {}", self.operations);
        println!("  Ops/sec:        {:.2}", self.ops_per_sec);
        println!("  Avg latency:    {:.2} µs/op",
                 self.duration.as_micros() as f64 / self.operations as f64);
        println!("{}", "=".repeat(70));
    }
}

/// Run a benchmark and return timing results
fn benchmark<F>(name: &str, iterations: usize, mut f: F) -> BenchResult
where
    F: FnMut(),
{
    // Warm up (reduced to 2 iterations)
    for _ in 0..2 {
        f();
    }

    // Actual benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let duration = start.elapsed();

    let ops_per_sec = iterations as f64 / duration.as_secs_f64();

    BenchResult {
        name: name.to_string(),
        duration,
        operations: iterations,
        ops_per_sec,
    }
}

/// Benchmark 1: Many small allocations
fn bench_many_small_allocs(count: usize) -> BenchResult {
    let mut ptrs = Vec::with_capacity(count);

    let result = benchmark("Many Small Allocations (8-64 bytes)", count, || {
        unsafe {
            let size = 8 + (ptrs.len() % 7) * 8;
            let ptr = gl_malloc(size);
            if !ptr.is_null() {
                ptrs.push(ptr);
            }
        }
    });

    // Cleanup
    for ptr in ptrs {
        unsafe { gl_free(ptr) };
    }

    result
}

/// Benchmark 2: Allocate-free pairs
fn bench_alloc_free_pairs(iterations: usize) -> BenchResult {
    benchmark("Allocate-Free Pairs", iterations, || {
        unsafe {
            let ptr = gl_malloc(64);
            if !ptr.is_null() {
                gl_free(ptr);
            }
        }
    })
}

/// Benchmark 3: Mixed sizes
fn bench_mixed_allocs(count: usize) -> BenchResult {
    let mut ptrs = Vec::with_capacity(count);
    let sizes = [8, 16, 32, 64, 128, 256];

    let result = benchmark("Mixed Allocation Sizes", count, || {
        unsafe {
            let size = sizes[ptrs.len() % sizes.len()];
            let ptr = gl_malloc(size);
            if !ptr.is_null() {
                ptrs.push(ptr);
            }
        }
    });

    // Cleanup
    for ptr in ptrs {
        unsafe { gl_free(ptr) };
    }

    result
}

fn main() {
    println!("\nGlimmer-Weave Allocator Performance Benchmarks (Simple)");
    println!("========================================================\n");

    // Initialize allocator
    unsafe {
        gl_init_allocator();
    }

    println!("Running reduced-iteration benchmarks...\n");

    // Run benchmarks with reduced iterations
    let results = vec![
        bench_alloc_free_pairs(100),      // Was 10000
        bench_many_small_allocs(100),     // Was 1000
        bench_mixed_allocs(100),          // Was 1000
    ];

    // Print results
    for result in &results {
        result.print();
    }

    // Summary
    println!("\n{}", "=".repeat(70));
    println!("Summary");
    println!("{}", "-".repeat(70));

    let total_ops: usize = results.iter().map(|r| r.operations).sum();
    let total_time: Duration = results.iter().map(|r| r.duration).sum();
    let avg_ops_per_sec: f64 = results.iter().map(|r| r.ops_per_sec).sum::<f64>() / results.len() as f64;

    println!("  Total operations:  {}", total_ops);
    println!("  Total time:        {:?}", total_time);
    println!("  Avg ops/sec:       {:.2}", avg_ops_per_sec);
    println!("{}", "=".repeat(70));

    println!("\n✓ Benchmarks completed successfully!");
    println!("\nNext steps:");
    println!("  - Increase iterations if performance is acceptable");
    println!("  - Add segregated free lists for common sizes (16, 32, 64 bytes)");
    println!("  - Implement fast path for alloc-free pairs");
    println!("  - Consider limiting free list search depth");
}
