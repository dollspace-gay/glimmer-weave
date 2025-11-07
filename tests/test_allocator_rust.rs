// Only compile these tests if the allocator was successfully built
#![cfg(all(target_arch = "x86_64", not(target_env = "msvc")))]

/**
 * Rust-based unit tests for gl_malloc/gl_free heap allocator
 *
 * Tests the free-list allocator implemented in native_allocator.S
 * using Rust FFI (Foreign Function Interface)
 *
 * Run with: cargo test --test test_allocator_rust
 *
 * NOTE: These tests require GNU assembler (gas) to compile the native allocator.
 * On Windows MSVC, these tests will be skipped. Use one of:
 * - Linux/macOS (gas available by default)
 * - Windows with MinGW: cargo test --target x86_64-pc-windows-gnu
 */

use std::ptr;
use glimmer_weave::native_allocator::*;

//==============================================================================
// Test 1: Basic Allocation
//==============================================================================
#[test]
fn test_basic_allocation() {
    unsafe {
        // Allocate a small block
        let ptr1 = gl_malloc(64);
        assert!(!ptr1.is_null(), "gl_malloc(64) should return non-NULL pointer");

        // Verify pointer is 8-byte aligned
        assert_eq!(
            ptr1 as usize % 8,
            0,
            "Pointer should be 8-byte aligned"
        );

        // Write and read data
        let data = ptr1 as *mut u64;
        *data.offset(0) = 0xDEADBEEFCAFEBABE;
        *data.offset(1) = 0x1234567890ABCDEF;

        assert_eq!(*data.offset(0), 0xDEADBEEFCAFEBABE, "Should be able to write/read data");
        assert_eq!(*data.offset(1), 0x1234567890ABCDEF, "Should be able to write/read data");

        // Free the block
        gl_free(ptr1);
    }
}

//==============================================================================
// Test 2: Multiple Allocations
//==============================================================================
#[test]
fn test_multiple_allocations() {
    unsafe {
        let ptr1 = gl_malloc(32);
        let ptr2 = gl_malloc(64);
        let ptr3 = gl_malloc(128);

        assert!(!ptr1.is_null(), "First allocation should succeed");
        assert!(!ptr2.is_null(), "Second allocation should succeed");
        assert!(!ptr3.is_null(), "Third allocation should succeed");

        // Pointers should be different
        assert_ne!(ptr1, ptr2, "Allocations should return different pointers");
        assert_ne!(ptr2, ptr3, "Allocations should return different pointers");
        assert_ne!(ptr1, ptr3, "Allocations should return different pointers");

        // Write unique data to each block
        *(ptr1 as *mut u64) = 0x1111;
        *(ptr2 as *mut u64) = 0x2222;
        *(ptr3 as *mut u64) = 0x3333;

        // Verify data integrity
        assert_eq!(*(ptr1 as *mut u64), 0x1111, "Data in block 1 should be intact");
        assert_eq!(*(ptr2 as *mut u64), 0x2222, "Data in block 2 should be intact");
        assert_eq!(*(ptr3 as *mut u64), 0x3333, "Data in block 3 should be intact");

        // Free all blocks
        gl_free(ptr1);
        gl_free(ptr2);
        gl_free(ptr3);
    }
}

//==============================================================================
// Test 3: Allocation After Free (Reuse)
//==============================================================================
#[test]
fn test_allocation_after_free() {
    unsafe {
        // Allocate and free a block
        let ptr1 = gl_malloc(128);
        assert!(!ptr1.is_null(), "Initial allocation should succeed");
        gl_free(ptr1);

        // Allocate same size - should reuse the freed block
        let ptr2 = gl_malloc(128);
        assert!(!ptr2.is_null(), "Allocation after free should succeed");

        println!("INFO: ptr1={:p}, ptr2={:p}", ptr1, ptr2);

        gl_free(ptr2);
    }
}

//==============================================================================
// Test 4: Free NULL Pointer (Should be No-op)
//==============================================================================
#[test]
fn test_free_null() {
    unsafe {
        // Freeing NULL should not crash
        gl_free(ptr::null_mut());

        // Should still be able to allocate after freeing NULL
        let ptr = gl_malloc(64);
        assert!(!ptr.is_null(), "Should be able to allocate after gl_free(NULL)");

        gl_free(ptr);
    }
}

//==============================================================================
// Test 5: Forward Coalescing
//==============================================================================
#[test]
fn test_forward_coalescing() {
    unsafe {
        // Allocate three adjacent blocks
        let ptr1 = gl_malloc(64);
        let ptr2 = gl_malloc(64);
        let ptr3 = gl_malloc(64);

        assert!(
            !ptr1.is_null() && !ptr2.is_null() && !ptr3.is_null(),
            "All allocations should succeed"
        );

        // Free middle block first
        gl_free(ptr2);

        // Free first block - should coalesce with middle
        gl_free(ptr1);

        // Free last block - should coalesce with combined block
        gl_free(ptr3);

        // Now allocate a large block that would fit in the coalesced space
        let ptr_large = gl_malloc(192);
        assert!(!ptr_large.is_null(), "Should be able to allocate coalesced space");

        println!("INFO: Coalesced allocation at {:p}", ptr_large);

        gl_free(ptr_large);
    }
}

//==============================================================================
// Test 6: Alignment Requirements
//==============================================================================
#[test]
fn test_alignment() {
    unsafe {
        // Test various allocation sizes to verify alignment
        let sizes = [1, 7, 8, 15, 16, 33, 64, 127, 128];
        let mut ptrs = Vec::new();

        for &size in &sizes {
            let ptr = gl_malloc(size);
            assert!(!ptr.is_null(), "Allocation should succeed");

            // Verify 8-byte alignment
            let addr = ptr as usize;
            assert_eq!(
                addr % 8,
                0,
                "malloc({}) at {:p} must be 8-byte aligned",
                size,
                ptr
            );

            println!(
                "INFO: malloc({}) = {:p} (aligned: {})",
                size,
                ptr,
                if addr % 8 == 0 { "✓" } else { "✗" }
            );

            ptrs.push(ptr);
        }

        // Free all
        for ptr in ptrs {
            gl_free(ptr);
        }
    }
}

//==============================================================================
// Test 7: Statistics Tracking
//==============================================================================
#[test]
fn test_statistics() {
    unsafe {
        let initial_bytes = gl_get_allocated_bytes();
        println!("INFO: Initial allocated bytes: {}", initial_bytes);

        // Allocate 256 bytes
        let ptr = gl_malloc(256);
        assert!(!ptr.is_null(), "Allocation should succeed");

        let after_alloc = gl_get_allocated_bytes();
        println!("INFO: After malloc(256): {} bytes", after_alloc);

        assert!(
            after_alloc >= initial_bytes + 256,
            "Allocated bytes should increase by at least 256"
        );

        // Free the block
        gl_free(ptr);

        let after_free = gl_get_allocated_bytes();
        println!("INFO: After free: {} bytes", after_free);

        assert!(
            after_free <= initial_bytes,
            "Allocated bytes should decrease after free"
        );
    }
}

//==============================================================================
// Test 8: Large Allocation (Heap Expansion)
//==============================================================================
#[test]
fn test_large_allocation() {
    unsafe {
        // Allocate a large block that requires heap expansion
        // Initial heap is 64KB, so allocate 128KB
        println!("DEBUG: Calling gl_malloc(128KB)...");
        let ptr = gl_malloc(128 * 1024);
        println!("DEBUG: Returned {:p}", ptr);
        assert!(!ptr.is_null(), "Large allocation should succeed (heap expansion)");

        println!("INFO: Allocated 128KB at {:p}", ptr);

        // Verify we can write to the entire block
        println!("DEBUG: Writing bytes...");
        ptr::write_bytes(ptr, 0xAB, 128 * 1024);
        println!("DEBUG: Write complete");

        // Check first and last bytes
        println!("DEBUG: Checking first byte...");
        let bytes = ptr;
        assert_eq!(*bytes.offset(0), 0xAB, "First byte should be writable");
        println!("DEBUG: Checking last byte...");
        assert_eq!(
            *bytes.offset(128 * 1024 - 1),
            0xAB,
            "Last byte should be writable"
        );
        println!("DEBUG: Checks passed");

        println!("DEBUG: Calling gl_free...");
        gl_free(ptr);
        println!("DEBUG: gl_free complete");
    }
}

//==============================================================================
// Test 9: Many Small Allocations
//==============================================================================
#[test]
fn test_many_small_allocations() {
    const NUM_ALLOCS: usize = 100;
    let mut ptrs = Vec::with_capacity(NUM_ALLOCS);

    unsafe {
        // Allocate many small blocks
        for i in 0..NUM_ALLOCS {
            let ptr = gl_malloc(16);
            assert!(!ptr.is_null(), "Small allocation should succeed");

            // Write unique value
            *(ptr as *mut u64) = i as u64;
            ptrs.push(ptr);
        }

        // Verify all data is intact
        for (i, &ptr) in ptrs.iter().enumerate() {
            let value = *(ptr as *mut u64);
            assert_eq!(
                value, i as u64,
                "Data should be intact after many allocations"
            );
        }

        // Free all blocks
        for ptr in ptrs.iter() {
            gl_free(*ptr);
        }

        println!("INFO: Successfully allocated and freed {} blocks", NUM_ALLOCS);
    }
}

//==============================================================================
// Test 10: Interleaved Allocation and Free
//==============================================================================
#[test]
fn test_interleaved_alloc_free() {
    unsafe {
        let ptr1 = gl_malloc(64);
        let ptr2 = gl_malloc(128);
        assert!(!ptr1.is_null() && !ptr2.is_null(), "Initial allocations should succeed");

        gl_free(ptr1);

        let ptr3 = gl_malloc(64);
        assert!(!ptr3.is_null(), "Allocation after partial free should succeed");

        gl_free(ptr2);

        let ptr4 = gl_malloc(256);
        assert!(!ptr4.is_null(), "Large allocation after frees should succeed");

        gl_free(ptr3);
        gl_free(ptr4);
    }
}

//==============================================================================
// Test 11: Zero-Size Allocation
//==============================================================================
#[test]
fn test_zero_size_allocation() {
    unsafe {
        // Zero-size allocation behavior is implementation-defined
        let ptr = gl_malloc(0);

        println!("INFO: gl_malloc(0) returned {:p}", ptr);

        // If non-NULL, should be able to free it
        if !ptr.is_null() {
            gl_free(ptr);
        }

        // Should still be able to allocate normally after
        let ptr2 = gl_malloc(64);
        assert!(!ptr2.is_null(), "Normal allocation after zero-size should work");
        gl_free(ptr2);
    }
}

//==============================================================================
// Test 12: Block Splitting
//==============================================================================
#[test]
fn test_block_splitting() {
    unsafe {
        println!("DEBUG: test_block_splitting starting");

        // Allocate a large block and free it
        println!("DEBUG: Calling gl_malloc(1024)...");
        let large = gl_malloc(1024);
        println!("DEBUG: gl_malloc(1024) returned {:p}", large);
        assert!(!large.is_null(), "Large allocation should succeed");

        println!("DEBUG: Calling gl_free({:p})...", large);
        gl_free(large);
        println!("DEBUG: gl_free completed");

        // Now allocate small blocks - should split the large free block
        println!("DEBUG: Calling gl_malloc(64) for small1...");
        let small1 = gl_malloc(64);
        println!("DEBUG: small1 = {:p}", small1);

        println!("DEBUG: Calling gl_malloc(64) for small2...");
        let small2 = gl_malloc(64);
        println!("DEBUG: small2 = {:p}", small2);

        println!("DEBUG: Calling gl_malloc(64) for small3...");
        let small3 = gl_malloc(64);
        println!("DEBUG: small3 = {:p}", small3);

        assert!(
            !small1.is_null() && !small2.is_null() && !small3.is_null(),
            "Small allocations should succeed after splitting large block"
        );

        println!("INFO: Split allocations: {:p}, {:p}, {:p}", small1, small2, small3);

        println!("DEBUG: Freeing small1...");
        gl_free(small1);
        println!("DEBUG: Freeing small2...");
        gl_free(small2);
        println!("DEBUG: Freeing small3...");
        gl_free(small3);
        println!("DEBUG: test_block_splitting completed");
    }
}

//==============================================================================
// Test 13: Heap Bounds Checking
//==============================================================================
#[test]
fn test_heap_bounds() {
    unsafe {
        let heap_start = gl_get_heap_start();
        let heap_end = gl_get_heap_end();

        println!("INFO: Heap range: {:p} - {:p}", heap_start, heap_end);

        assert!(!heap_start.is_null(), "Heap start should be initialized");
        assert!(!heap_end.is_null(), "Heap end should be initialized");
        assert!(heap_end > heap_start, "Heap end should be after heap start");

        // Allocate some memory and verify it's within bounds
        let ptr = gl_malloc(256);
        assert!(!ptr.is_null(), "Allocation should succeed");

        assert!(
            ptr >= heap_start,
            "Allocated pointer should be >= heap_start"
        );
        assert!(ptr < heap_end, "Allocated pointer should be < heap_end");

        println!("INFO: Allocated pointer {:p} is within heap bounds", ptr);

        gl_free(ptr);
    }
}

//==============================================================================
// Test 14: Stress Test - Random Allocations
//==============================================================================
#[test]
fn test_stress_random() {
    use std::collections::HashMap;

    const STRESS_ITERS: usize = 500;
    let mut active: HashMap<usize, (*mut u8, usize)> = HashMap::new();
    let mut next_id = 0;

    println!("INFO: Running {} random alloc/free operations...", STRESS_ITERS);

    unsafe {
        for i in 0..STRESS_ITERS {
            let op = i % 100;

            if op < 60 && active.len() < 50 {
                // 60% chance: allocate
                let size = (i % 1024) + 16;
                let ptr = gl_malloc(size);

                if !ptr.is_null() {
                    // Write pattern
                    ptr::write_bytes(ptr, (i & 0xFF) as u8, size);
                    active.insert(next_id, (ptr, size));
                    next_id += 1;
                }
            } else if !active.is_empty() {
                // 40% chance: free
                let ids: Vec<_> = active.keys().copied().collect();
                let idx = ids[i % ids.len()];
                if let Some((ptr, _size)) = active.remove(&idx) {
                    gl_free(ptr);
                }
            }
        }

        // Clean up remaining allocations
        for (_id, (ptr, _size)) in active.drain() {
            gl_free(ptr);
        }
    }

    println!("INFO: Stress test completed successfully");
}
