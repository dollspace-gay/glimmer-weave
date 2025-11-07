#![cfg(all(target_arch = "x86_64", not(target_env = "msvc")))]

use glimmer_weave::native_allocator::*;

#[test]
fn test_medium_allocation_just_under_initial_heap() {
    unsafe {
        println!("=== Test medium allocation (just under 64KB) ===");

        // Allocate 60KB - should fit in initial 64KB heap
        println!("1. Allocating 60KB...");
        let ptr = gl_malloc(60 * 1024);
        println!("   Ptr: {:p}", ptr);
        assert!(!ptr.is_null(), "60KB allocation should succeed");

        // Free it
        gl_free(ptr);
        println!("   Freed");

        // Now try 70KB - requires slightly more than initial heap
        println!("2. Allocating 70KB...");
        let ptr2 = gl_malloc(70 * 1024);
        println!("   Ptr: {:p}", ptr2);

        if ptr2.is_null() {
            println!("   FAILED: 70KB allocation returned NULL");
            panic!("70KB allocation failed - heap expansion not working");
        }

        println!("   SUCCESS!");
        gl_free(ptr2);
    }
}
