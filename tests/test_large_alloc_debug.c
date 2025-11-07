// Debug test for large allocation issue
// Compile: gcc -o test_large_alloc_debug test_large_alloc_debug.c -L../target/debug -lglimmer_weave -no-pie
// Run: LD_LIBRARY_PATH=../target/debug ./test_large_alloc_debug

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>

// FFI declarations
extern void gl_init_allocator(void);
extern void* gl_malloc(size_t size);
extern void gl_free(void* ptr);
extern size_t gl_get_allocated_bytes(void);
extern size_t gl_get_free_bytes(void);

// Assembly functions we want to trace
extern void* gl_free_list_head;
extern void* gl_heap_start;
extern void* gl_heap_end;

int main() {
    printf("=== Large Allocation Debug Test ===\n\n");

    // Explicitly initialize
    printf("1. Initializing allocator...\n");
    gl_init_allocator();
    printf("   Heap start: %p\n", gl_heap_start);
    printf("   Heap end: %p\n", gl_heap_end);
    printf("   Free list head: %p\n", gl_free_list_head);
    printf("   Free bytes: %zu\n", gl_get_free_bytes());
    printf("\n");

    // Small allocation to verify initialization
    printf("2. Small allocation (64 bytes)...\n");
    void* small = gl_malloc(64);
    printf("   Ptr: %p\n", small);
    printf("   Allocated bytes: %zu\n", gl_get_allocated_bytes());
    printf("   Free bytes: %zu\n", gl_get_free_bytes());
    printf("\n");

    // Free it
    printf("3. Freeing small allocation...\n");
    gl_free(small);
    printf("   Allocated bytes: %zu\n", gl_get_allocated_bytes());
    printf("   Free bytes: %zu\n", gl_get_free_bytes());
    printf("\n");

    // Medium allocation (1KB)
    printf("4. Medium allocation (1024 bytes)...\n");
    void* medium = gl_malloc(1024);
    printf("   Ptr: %p\n", medium);
    printf("   Allocated bytes: %zu\n", gl_get_allocated_bytes());
    printf("   Free bytes: %zu\n", gl_get_free_bytes());
    printf("\n");

    // Free it
    printf("5. Freeing medium allocation...\n");
    gl_free(medium);
    printf("   Allocated bytes: %zu\n", gl_get_allocated_bytes());
    printf("   Free bytes: %zu\n", gl_get_free_bytes());
    printf("\n");

    // Now try a large allocation that will hang
    printf("6. Large allocation (128KB = 131072 bytes)...\n");
    printf("   This requires heap expansion (initial heap is 64KB)\n");
    printf("   Calling gl_malloc(131072)...\n");
    fflush(stdout);  // Make sure output is written before potential hang

    void* large = gl_malloc(131072);

    printf("   SUCCESS! Ptr: %p\n", large);
    printf("   Allocated bytes: %zu\n", gl_get_allocated_bytes());
    printf("   Free bytes: %zu\n", gl_get_free_bytes());
    printf("\n");

    // Free it
    printf("7. Freeing large allocation...\n");
    gl_free(large);
    printf("   Allocated bytes: %zu\n", gl_get_allocated_bytes());
    printf("   Free bytes: %zu\n", gl_get_free_bytes());
    printf("\n");

    printf("=== Test Complete ===\n");
    return 0;
}
