// Allocator Performance Benchmarks
//
// Benchmarks various allocation patterns to identify performance characteristics
// and guide optimization efforts.

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
    allocated_bytes: usize,
}

impl BenchResult {
    fn print(&self) {
        println!("\n{}", "=".repeat(70));
        println!("Benchmark: {}", self.name);
        println!("{}", "-".repeat(70));
        println!("  Duration:       {:?}", self.duration);
        println!("  Operations:     {}", self.operations);
        println!("  Ops/sec:        {:.2}", self.ops_per_sec);
        println!("  Allocated:      {} bytes", self.allocated_bytes);
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
    // Warm up
    for _ in 0..10 {
        f();
    }

    // Actual benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let duration = start.elapsed();

    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    let allocated_bytes = unsafe { gl_get_allocated_bytes() as usize };

    BenchResult {
        name: name.to_string(),
        duration,
        operations: iterations,
        ops_per_sec,
        allocated_bytes,
    }
}

/// Benchmark 1: Many small allocations (common case for scripting)
fn bench_many_small_allocs(count: usize) -> BenchResult {
    let mut ptrs = Vec::with_capacity(count);

    let result = benchmark("Many Small Allocations (8-64 bytes)", count, || {
        unsafe {
            let size = 8 + (ptrs.len() % 7) * 8; // Sizes: 8, 16, 24, 32, 40, 48, 56, 64
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

/// Benchmark 2: Few large allocations
fn bench_few_large_allocs(count: usize) -> BenchResult {
    let mut ptrs = Vec::with_capacity(count);

    let result = benchmark("Few Large Allocations (1KB-16KB)", count, || {
        unsafe {
            let size = 1024 + (ptrs.len() % 16) * 1024; // 1KB to 16KB
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

/// Benchmark 3: Mixed allocation sizes
fn bench_mixed_allocs(count: usize) -> BenchResult {
    let mut ptrs = Vec::with_capacity(count);

    let sizes = [8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

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

/// Benchmark 4: Allocate-free pairs (no fragmentation)
fn bench_alloc_free_pairs(iterations: usize) -> BenchResult {
    benchmark("Allocate-Free Pairs (No Fragmentation)", iterations, || {
        unsafe {
            let ptr = gl_malloc(64);
            if !ptr.is_null() {
                gl_free(ptr);
            }
        }
    })
}

/// Benchmark 5: Alternating alloc/free (high fragmentation)
fn bench_alternating_alloc_free(iterations: usize) -> BenchResult {
    let mut ptrs = Vec::with_capacity(100);

    // Pre-allocate some blocks
    for _ in 0..100 {
        unsafe {
            let ptr = gl_malloc(64);
            if !ptr.is_null() {
                ptrs.push(ptr);
            }
        }
    }

    let result = benchmark("Alternating Alloc/Free (Fragmentation)", iterations, || {
        unsafe {
            // Free every other block
            if ptrs.len() > 50 {
                let idx = ptrs.len() / 2;
                gl_free(ptrs.remove(idx));
            }

            // Allocate new block
            let ptr = gl_malloc(64);
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

/// Benchmark 6: Worst-case search (allocate until free list is long)
fn bench_worst_case_search() -> BenchResult {
    let mut ptrs = Vec::with_capacity(1000);

    // Create fragmented free list
    for _ in 0..1000 {
        unsafe {
            let ptr = gl_malloc(64);
            if !ptr.is_null() {
                ptrs.push(ptr);
            }
        }
    }

    // Free every other block to create fragmented free list
    let mut i = 0;
    ptrs.retain(|&ptr| {
        i += 1;
        if i % 2 == 0 {
            unsafe { gl_free(ptr) };
            false
        } else {
            true
        }
    });

    // Now benchmark allocation with long free list
    let result = benchmark("Worst-Case Search (500 free blocks)", 100, || {
        unsafe {
            // Try to allocate size that will require searching most of free list
            let ptr = gl_malloc(64);
            if !ptr.is_null() {
                gl_free(ptr);
            }
        }
    });

    // Cleanup
    for ptr in ptrs {
        unsafe { gl_free(ptr) };
    }

    result
}

/// Benchmark 7: Common size allocations (to test segregated list benefits)
fn bench_common_sizes(iterations: usize) -> BenchResult {
    let mut ptrs = Vec::with_capacity(iterations);

    // Common sizes in scripting: 16, 32, 64 bytes (structs, closures, strings)
    let common_sizes = [16, 32, 64];

    let result = benchmark("Common Sizes (16, 32, 64 bytes)", iterations, || {
        unsafe {
            let size = common_sizes[ptrs.len() % common_sizes.len()];
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

/// Benchmark 8: Realloc pattern (common in growing arrays)
fn bench_realloc_pattern(iterations: usize) -> BenchResult {
    let mut ptrs = Vec::new();

    let result = benchmark("Realloc Pattern (Growing Arrays)", iterations, || {
        unsafe {
            // Simulate realloc: alloc new, copy, free old
            let new_size = (ptrs.len() + 1) * 8;
            let new_ptr = gl_malloc(new_size);

            if !new_ptr.is_null() {
                if let Some(old_ptr) = ptrs.last() {
                    // Simulate copy (just write a byte)
                    *new_ptr = 42;
                    gl_free(*old_ptr);
                }
                ptrs.push(new_ptr);
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
    println!("\nGlimmer-Weave Allocator Performance Benchmarks");
    println!("===============================================\n");

    // Initialize allocator
    unsafe {
        gl_init_allocator();
    }

    // Run benchmarks
    let results = vec![
        bench_alloc_free_pairs(10000),
        bench_many_small_allocs(1000),
        bench_few_large_allocs(100),
        bench_mixed_allocs(1000),
        bench_alternating_alloc_free(1000),
        bench_common_sizes(1000),
        bench_realloc_pattern(100),
        bench_worst_case_search(),
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

    println!("\nOptimization Opportunities:");
    println!("  1. Segregated free lists for sizes 16, 32, 64 bytes");
    println!("  2. Fast path for alloc-free pairs");
    println!("  3. Limit free list search depth");
    println!("  4. Cache last allocation for realloc patterns");
}
