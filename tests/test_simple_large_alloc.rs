// Simple test for large allocation to debug hang
#![cfg(all(target_arch = "x86_64", not(target_env = "msvc")))]

use glimmer_weave::native_allocator::*;

#[test]
fn test_simple_large_alloc() {
    unsafe {
        println!("=== Test Start ===");

        // First, a small allocation to ensure init works
        println!("1. Small allocation...");
        let small = gl_malloc(64);
        println!("   Small ptr: {:p}", small);
        assert!(!small.is_null());
        gl_free(small);
        println!("   Freed small");

        // Now try medium allocation (shouldn't require expansion)
        println!("2. Medium allocation (1KB)...");
        let medium = gl_malloc(1024);
        println!("   Medium ptr: {:p}", medium);
        assert!(!medium.is_null());
        gl_free(medium);
        println!("   Freed medium");

        // Try allocation that requires expansion (70KB - just over 64KB initial heap)
        println!("3. Larger allocation (70KB) - should use initial heap...");
        let larger = gl_malloc(70 * 1024);
        println!("   Larger ptr: {:p}", larger);
        assert!(!larger.is_null(), "70KB allocation failed!");
        gl_free(larger);

        // Now try allocation that definitely requires expansion (128KB)
        println!("4. Large allocation (128KB) - will require heap expansion...");
        let large = gl_malloc(128 * 1024);
        println!("   Large ptr: {:p}", large);
        assert!(!large.is_null(), "Large allocation failed!");

        println!("5. Freeing large allocation...");
        gl_free(large);

        println!("=== Test Complete ===");
    }
}
