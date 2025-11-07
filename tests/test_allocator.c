/**
 * Unit tests for gl_malloc/gl_free heap allocator
 *
 * Tests the free-list allocator implemented in native_allocator.S
 *
 * Compile and run:
 *   gcc -o test_allocator test_allocator.c src/native_allocator.S
 *   ./test_allocator
 */

#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <assert.h>

// External functions from native_allocator.S
extern void* gl_malloc(size_t size);
extern void gl_free(void* ptr);
extern uint64_t gl_get_allocated_bytes(void);
extern void* gl_get_heap_start(void);
extern void* gl_get_heap_end(void);

// Test result tracking
static int tests_passed = 0;
static int tests_failed = 0;

#define TEST(name) \
    printf("\n=== TEST: %s ===\n", name); \
    if (run_##name())

#define ASSERT(condition, message) \
    do { \
        if (!(condition)) { \
            printf("  ❌ FAILED: %s\n", message); \
            printf("     at line %d\n", __LINE__); \
            tests_failed++; \
            return 0; \
        } \
    } while(0)

#define PASS() \
    do { \
        printf("  ✅ PASSED\n"); \
        tests_passed++; \
        return 1; \
    } while(0)

//==============================================================================
// Test 1: Basic Allocation
//==============================================================================
int run_test_basic_allocation(void) {
    // Allocate a small block
    void* ptr1 = gl_malloc(64);
    ASSERT(ptr1 != NULL, "gl_malloc(64) should return non-NULL pointer");

    // Verify pointer is 8-byte aligned
    ASSERT(((uintptr_t)ptr1 % 8) == 0, "Pointer should be 8-byte aligned");

    // Write and read data
    uint64_t* data = (uint64_t*)ptr1;
    data[0] = 0xDEADBEEFCAFEBABE;
    data[1] = 0x1234567890ABCDEF;

    ASSERT(data[0] == 0xDEADBEEFCAFEBABE, "Should be able to write/read data");
    ASSERT(data[1] == 0x1234567890ABCDEF, "Should be able to write/read data");

    // Free the block
    gl_free(ptr1);

    PASS();
}

//==============================================================================
// Test 2: Multiple Allocations
//==============================================================================
int run_test_multiple_allocations(void) {
    void* ptr1 = gl_malloc(32);
    void* ptr2 = gl_malloc(64);
    void* ptr3 = gl_malloc(128);

    ASSERT(ptr1 != NULL, "First allocation should succeed");
    ASSERT(ptr2 != NULL, "Second allocation should succeed");
    ASSERT(ptr3 != NULL, "Third allocation should succeed");

    // Pointers should be different
    ASSERT(ptr1 != ptr2, "Allocations should return different pointers");
    ASSERT(ptr2 != ptr3, "Allocations should return different pointers");
    ASSERT(ptr1 != ptr3, "Allocations should return different pointers");

    // Write unique data to each block
    *(uint64_t*)ptr1 = 0x1111;
    *(uint64_t*)ptr2 = 0x2222;
    *(uint64_t*)ptr3 = 0x3333;

    // Verify data integrity
    ASSERT(*(uint64_t*)ptr1 == 0x1111, "Data in block 1 should be intact");
    ASSERT(*(uint64_t*)ptr2 == 0x2222, "Data in block 2 should be intact");
    ASSERT(*(uint64_t*)ptr3 == 0x3333, "Data in block 3 should be intact");

    // Free all blocks
    gl_free(ptr1);
    gl_free(ptr2);
    gl_free(ptr3);

    PASS();
}

//==============================================================================
// Test 3: Allocation After Free (Reuse)
//==============================================================================
int run_test_allocation_after_free(void) {
    // Allocate and free a block
    void* ptr1 = gl_malloc(128);
    ASSERT(ptr1 != NULL, "Initial allocation should succeed");
    gl_free(ptr1);

    // Allocate same size - should reuse the freed block
    void* ptr2 = gl_malloc(128);
    ASSERT(ptr2 != NULL, "Allocation after free should succeed");

    // Should likely get the same or nearby address (reuse freed block)
    // Note: Exact reuse is not guaranteed due to block splitting/coalescing
    printf("  INFO: ptr1=%p, ptr2=%p\n", ptr1, ptr2);

    gl_free(ptr2);

    PASS();
}

//==============================================================================
// Test 4: Free NULL Pointer (Should be No-op)
//==============================================================================
int run_test_free_null(void) {
    // Freeing NULL should not crash
    gl_free(NULL);

    // Should still be able to allocate after freeing NULL
    void* ptr = gl_malloc(64);
    ASSERT(ptr != NULL, "Should be able to allocate after gl_free(NULL)");

    gl_free(ptr);

    PASS();
}

//==============================================================================
// Test 5: Forward Coalescing
//==============================================================================
int run_test_forward_coalescing(void) {
    // Allocate three adjacent blocks
    void* ptr1 = gl_malloc(64);
    void* ptr2 = gl_malloc(64);
    void* ptr3 = gl_malloc(64);

    ASSERT(ptr1 != NULL && ptr2 != NULL && ptr3 != NULL,
           "All allocations should succeed");

    // Free middle block first
    gl_free(ptr2);

    // Free first block - should coalesce with middle
    gl_free(ptr1);

    // Free last block - should coalesce with combined block
    gl_free(ptr3);

    // Now allocate a large block that would fit in the coalesced space
    void* ptr_large = gl_malloc(192);
    ASSERT(ptr_large != NULL, "Should be able to allocate coalesced space");

    printf("  INFO: Coalesced allocation at %p\n", ptr_large);

    gl_free(ptr_large);

    PASS();
}

//==============================================================================
// Test 6: Alignment Requirements
//==============================================================================
int run_test_alignment(void) {
    // Test various allocation sizes to verify alignment
    size_t sizes[] = { 1, 7, 8, 15, 16, 33, 64, 127, 128 };
    void* ptrs[9];

    for (int i = 0; i < 9; i++) {
        ptrs[i] = gl_malloc(sizes[i]);
        ASSERT(ptrs[i] != NULL, "Allocation should succeed");

        // Verify 8-byte alignment
        uintptr_t addr = (uintptr_t)ptrs[i];
        ASSERT((addr % 8) == 0, "All allocations must be 8-byte aligned");

        printf("  INFO: malloc(%zu) = %p (aligned: %s)\n",
               sizes[i], ptrs[i], (addr % 8 == 0) ? "✓" : "✗");
    }

    // Free all
    for (int i = 0; i < 9; i++) {
        gl_free(ptrs[i]);
    }

    PASS();
}

//==============================================================================
// Test 7: Statistics Tracking
//==============================================================================
int run_test_statistics(void) {
    uint64_t initial_bytes = gl_get_allocated_bytes();
    printf("  INFO: Initial allocated bytes: %llu\n", (unsigned long long)initial_bytes);

    // Allocate 256 bytes
    void* ptr = gl_malloc(256);
    ASSERT(ptr != NULL, "Allocation should succeed");

    uint64_t after_alloc = gl_get_allocated_bytes();
    printf("  INFO: After malloc(256): %llu bytes\n", (unsigned long long)after_alloc);

    ASSERT(after_alloc >= initial_bytes + 256,
           "Allocated bytes should increase by at least 256");

    // Free the block
    gl_free(ptr);

    uint64_t after_free = gl_get_allocated_bytes();
    printf("  INFO: After free: %llu bytes\n", (unsigned long long)after_free);

    ASSERT(after_free <= initial_bytes,
           "Allocated bytes should decrease after free");

    PASS();
}

//==============================================================================
// Test 8: Large Allocation (Heap Expansion)
//==============================================================================
int run_test_large_allocation(void) {
    // Allocate a large block that requires heap expansion
    // Initial heap is 64KB, so allocate 128KB
    void* ptr = gl_malloc(128 * 1024);
    ASSERT(ptr != NULL, "Large allocation should succeed (heap expansion)");

    printf("  INFO: Allocated 128KB at %p\n", ptr);

    // Verify we can write to the entire block
    memset(ptr, 0xAB, 128 * 1024);

    // Check first and last bytes
    uint8_t* bytes = (uint8_t*)ptr;
    ASSERT(bytes[0] == 0xAB, "First byte should be writable");
    ASSERT(bytes[128 * 1024 - 1] == 0xAB, "Last byte should be writable");

    gl_free(ptr);

    PASS();
}

//==============================================================================
// Test 9: Many Small Allocations
//==============================================================================
int run_test_many_small_allocations(void) {
    #define NUM_ALLOCS 100
    void* ptrs[NUM_ALLOCS];

    // Allocate many small blocks
    for (int i = 0; i < NUM_ALLOCS; i++) {
        ptrs[i] = gl_malloc(16);
        ASSERT(ptrs[i] != NULL, "Small allocation should succeed");

        // Write unique value
        *(uint64_t*)ptrs[i] = (uint64_t)i;
    }

    // Verify all data is intact
    for (int i = 0; i < NUM_ALLOCS; i++) {
        uint64_t value = *(uint64_t*)ptrs[i];
        ASSERT(value == (uint64_t)i, "Data should be intact after many allocations");
    }

    // Free all blocks
    for (int i = 0; i < NUM_ALLOCS; i++) {
        gl_free(ptrs[i]);
    }

    printf("  INFO: Successfully allocated and freed %d blocks\n", NUM_ALLOCS);

    PASS();
}

//==============================================================================
// Test 10: Interleaved Allocation and Free
//==============================================================================
int run_test_interleaved_alloc_free(void) {
    void* ptr1 = gl_malloc(64);
    void* ptr2 = gl_malloc(128);
    ASSERT(ptr1 != NULL && ptr2 != NULL, "Initial allocations should succeed");

    gl_free(ptr1);

    void* ptr3 = gl_malloc(64);
    ASSERT(ptr3 != NULL, "Allocation after partial free should succeed");

    gl_free(ptr2);

    void* ptr4 = gl_malloc(256);
    ASSERT(ptr4 != NULL, "Large allocation after frees should succeed");

    gl_free(ptr3);
    gl_free(ptr4);

    PASS();
}

//==============================================================================
// Test 11: Zero-Size Allocation
//==============================================================================
int run_test_zero_size_allocation(void) {
    // Zero-size allocation behavior is implementation-defined
    // Our allocator should either return NULL or a valid pointer
    void* ptr = gl_malloc(0);

    printf("  INFO: gl_malloc(0) returned %p\n", ptr);

    // If non-NULL, should be able to free it
    if (ptr != NULL) {
        gl_free(ptr);
    }

    // Should still be able to allocate normally after
    void* ptr2 = gl_malloc(64);
    ASSERT(ptr2 != NULL, "Normal allocation after zero-size should work");
    gl_free(ptr2);

    PASS();
}

//==============================================================================
// Test 12: Block Splitting
//==============================================================================
int run_test_block_splitting(void) {
    // Allocate a large block and free it
    void* large = gl_malloc(1024);
    ASSERT(large != NULL, "Large allocation should succeed");
    gl_free(large);

    // Now allocate a small block - should split the large free block
    void* small1 = gl_malloc(64);
    void* small2 = gl_malloc(64);
    void* small3 = gl_malloc(64);

    ASSERT(small1 != NULL && small2 != NULL && small3 != NULL,
           "Small allocations should succeed after splitting large block");

    printf("  INFO: Split allocations: %p, %p, %p\n", small1, small2, small3);

    gl_free(small1);
    gl_free(small2);
    gl_free(small3);

    PASS();
}

//==============================================================================
// Test 13: Heap Bounds Checking
//==============================================================================
int run_test_heap_bounds(void) {
    void* heap_start = gl_get_heap_start();
    void* heap_end = gl_get_heap_end();

    printf("  INFO: Heap range: %p - %p\n", heap_start, heap_end);

    ASSERT(heap_start != NULL, "Heap start should be initialized");
    ASSERT(heap_end != NULL, "Heap end should be initialized");
    ASSERT(heap_end > heap_start, "Heap end should be after heap start");

    // Allocate some memory and verify it's within bounds
    void* ptr = gl_malloc(256);
    ASSERT(ptr != NULL, "Allocation should succeed");

    ASSERT(ptr >= heap_start, "Allocated pointer should be >= heap_start");
    ASSERT(ptr < heap_end, "Allocated pointer should be < heap_end");

    printf("  INFO: Allocated pointer %p is within heap bounds\n", ptr);

    gl_free(ptr);

    PASS();
}

//==============================================================================
// Test 14: Stress Test - Random Allocations
//==============================================================================
int run_test_stress_random(void) {
    #define STRESS_ITERS 500
    void* active[50];
    int active_count = 0;

    printf("  INFO: Running %d random alloc/free operations...\n", STRESS_ITERS);

    for (int i = 0; i < STRESS_ITERS; i++) {
        int op = rand() % 100;

        if (op < 60 && active_count < 50) {
            // 60% chance: allocate
            size_t size = (rand() % 1024) + 16;
            void* ptr = gl_malloc(size);

            if (ptr != NULL) {
                active[active_count++] = ptr;

                // Write pattern
                memset(ptr, (i & 0xFF), size);
            }
        } else if (active_count > 0) {
            // 40% chance: free
            int idx = rand() % active_count;
            gl_free(active[idx]);

            // Remove from active list
            active[idx] = active[--active_count];
        }
    }

    // Clean up remaining allocations
    for (int i = 0; i < active_count; i++) {
        gl_free(active[i]);
    }

    printf("  INFO: Stress test completed successfully\n");

    PASS();
}

//==============================================================================
// Main Test Runner
//==============================================================================
int main(void) {
    printf("\n");
    printf("╔════════════════════════════════════════════════════════════╗\n");
    printf("║  Glimmer-Weave Heap Allocator Test Suite                  ║\n");
    printf("║  Testing gl_malloc/gl_free implementation                 ║\n");
    printf("╚════════════════════════════════════════════════════════════╝\n");

    // Run all tests
    TEST("basic_allocation") { }
    TEST("multiple_allocations") { }
    TEST("allocation_after_free") { }
    TEST("free_null") { }
    TEST("forward_coalescing") { }
    TEST("alignment") { }
    TEST("statistics") { }
    TEST("large_allocation") { }
    TEST("many_small_allocations") { }
    TEST("interleaved_alloc_free") { }
    TEST("zero_size_allocation") { }
    TEST("block_splitting") { }
    TEST("heap_bounds") { }
    TEST("stress_random") { }

    // Print summary
    printf("\n");
    printf("╔════════════════════════════════════════════════════════════╗\n");
    printf("║  Test Results                                              ║\n");
    printf("╠════════════════════════════════════════════════════════════╣\n");
    printf("║  ✅ Passed: %-4d                                           ║\n", tests_passed);
    printf("║  ❌ Failed: %-4d                                           ║\n", tests_failed);
    printf("╚════════════════════════════════════════════════════════════╝\n");

    return tests_failed > 0 ? 1 : 0;
}
