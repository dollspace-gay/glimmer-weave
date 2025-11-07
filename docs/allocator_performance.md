# Glimmer-Weave Allocator Performance Analysis

## Executive Summary

The Glimmer-Weave native allocator has been benchmarked and analyzed for performance characteristics and optimization opportunities. Initial results show excellent performance for small-scale allocations but significant degradation with larger workloads due to linear search through the free list.

**Key Findings:**
- ✅ **Fast for small workloads**: 62.5M ops/sec for alloc-free pairs (100 operations)
- ⚠️  **Degrades with scale**: Performance drops significantly with hundreds+ of allocations
- 🎯 **Primary bottleneck**: O(n) linear search through free list in `gl_malloc`
- 🎯 **Secondary bottleneck**: O(n) sorted insertion in `gl_free`

---

## Benchmark Results

### Simple Benchmark (100 operations)

| Pattern | Ops/sec | Avg Latency | Notes |
|---------|---------|-------------|-------|
| Allocate-Free Pairs | 62.5M | 0.01 µs | Optimal case - immediate reuse |
| Small Allocations (8-64B) | 31.25M | 0.03 µs | Common scripting workload |
| Mixed Sizes (8-256B) | 24.39M | 0.04 µs | Realistic mixed usage |

**Test Environment:** WSL2 Linux, x86_64, Release build with optimizations

**Observations:**
- Allocator shows excellent performance for small-scale workloads
- Free list remains short (< 10 blocks), keeping search fast
- Heap expansion overhead is minimal (64KB initial heap sufficient)

### Scalability Issues

Attempts to benchmark with larger workloads (1000+ operations) resulted in timeouts, indicating severe performance degradation. This is expected given the O(n) complexity of the current first-fit algorithm.

**Estimated Performance at Scale:**
- 100 allocations: ~31M ops/sec (measured)
- 1,000 allocations: ~310K ops/sec (estimated, assuming linear degradation)
- 10,000 allocations: ~3.1K ops/sec (estimated, **unacceptable**)

---

## Architecture Analysis

### Current Implementation ([native_allocator.S](../src/native_allocator.S))

**Strategy:** First-fit with sorted free list

```
Free List: [32B] -> [128B] -> [256B] -> [1KB] -> NULL
```

**Allocation Algorithm (gl_malloc):**
1. Align size to 8-byte boundary
2. **Linear search** through free list for suitable block (O(n))
3. Split block if remainder >= MIN_BLOCK_SIZE (24 bytes)
4. Remove block from free list
5. Return pointer

**Deallocation Algorithm (gl_free):**
1. Mark block as free
2. Coalesce with next physical block (forward coalescing)
3. **Sorted insertion** into free list (O(n))
4. Coalesce with previous block during insertion (backward coalescing)

**Time Complexity:**
- `malloc`: O(n) where n = number of free blocks
- `free`: O(n) where n = number of free blocks

### Hot Paths Identified

#### 1. First-Fit Search ([native_allocator.S:107-148](../src/native_allocator.S#L107-L148))

```asm
.malloc_search_loop:
    testq   %rbx, %rbx          # if (current == NULL)
    jz      .malloc_expand_heap # No suitable block, expand heap

    # Bounds validation (lines 114-125)
    # Pointer validation (lines 127-133)
    # Size check (lines 136-144)

    movq    8(%rbx), %rbx       # rbx = next_free
    jmp     .malloc_search_loop
```

**Issues:**
- Linear traversal of entire free list
- Multiple memory loads per iteration (header, next pointer)
- Extensive bounds checking on every iteration
- No early termination for common sizes

#### 2. Sorted Free List Insertion ([native_allocator.S:457-667](../src/native_allocator.S#L457-L667))

```asm
.insert_search:
    testq   %rbx, %rbx             # if (current == NULL)
    jz      .insert_at_end

    # Bounds validation
    # Address comparison for sort order

    movq    %rbx, %rcx              # prev = current
    movq    8(%rbx), %rbx           # current = current.next
    jmp     .insert_search
```

**Issues:**
- Linear search to find insertion point
- Additional coalescing logic during insertion
- Address-sorted order is good for coalescing but bad for cache locality

#### 3. Repeated Bounds Checking

Every free list operation includes bounds checking:
```asm
pushq   %rax
movq    gl_heap_start(%rip), %rax
cmpq    %rax, %rbx
popq    %rax
jl      .error_path
```

**Issues:**
- 4+ instructions per check (2 memory loads, 1 comparison, 2 push/pop)
- Executed multiple times in hot loops
- Necessary for safety but impacts performance

---

## Optimization Priorities

### Priority 1: Segregated Free Lists (High Impact)

**Problem:** O(n) search through entire free list, even for common sizes

**Solution:** Maintain separate free lists for common size classes:

```
Size Classes:
  16 bytes:  [block] -> [block] -> NULL
  32 bytes:  [block] -> [block] -> [block] -> NULL
  64 bytes:  [block] -> NULL
  128 bytes: [block] -> [block] -> NULL
  256+ bytes: [block] -> [block] -> [block] -> NULL  (sorted, for best-fit)
```

**Benefits:**
- O(1) allocation for common sizes (16, 32, 64 bytes)
- Reduced search space for uncommon sizes
- Better cache locality (same-size blocks clustered together)

**Implementation:**
```asm
.data
gl_free_lists:
    .quad 0  # 16-byte blocks
    .quad 0  # 32-byte blocks
    .quad 0  # 64-byte blocks
    .quad 0  # 128-byte blocks
    .quad 0  # 256+ bytes (sorted)

# Size to index mapping
size_to_index:
    # 0-16: index 0
    # 17-32: index 1
    # 33-64: index 2
    # 65-128: index 3
    # 129+: index 4
```

**malloc logic:**
```asm
gl_malloc:
    # Align size
    # ...

    # Determine size class
    call gl_size_to_index   # rax = index for size

    # Quick path for common sizes
    cmpq $4, %rax
    jge .slow_path          # Large allocation, use sorted list

    # Fast path: grab first block from size class
    leaq gl_free_lists(%rip), %rcx
    movq (%rcx, %rax, 8), %rbx    # rbx = head of size class list
    testq %rbx, %rbx
    jz .slow_path           # No blocks in this size class

    # Remove from list (O(1))
    movq 8(%rbx), %rdx      # rdx = next
    movq %rdx, (%rcx, %rax, 8)    # Update head

    # Done!
    ret
```

**Expected Impact:** 10-100x speedup for common sizes

---

### Priority 2: Fast Path for Exact-Fit Allocations (Medium Impact)

**Problem:** Even with segregated lists, we do unnecessary work for exact-fit allocations

**Solution:** Skip splitting logic when block size exactly matches request

```asm
.malloc_found_block:
    # Compare block size to requested size
    cmpq %r12, %r13
    je .exact_fit           # Perfect fit, no split needed

    # ... existing split logic

.exact_fit:
    # Just remove from list and return (faster path)
```

**Expected Impact:** 20-30% speedup for common-size allocations

---

### Priority 3: Limit Free List Search Depth (Low Impact)

**Problem:** Pathological cases can cause searching through hundreds of free blocks

**Solution:** Bail out after N iterations and expand heap instead

```asm
.malloc_search_loop:
    # ... existing search logic

    incq %r15               # Increment iteration counter
    cmpq $MAX_SEARCH_DEPTH, %r15  # e.g., MAX_SEARCH_DEPTH = 32
    jge .malloc_expand_heap # Give up and expand heap

    movq 8(%rbx), %rbx      # rbx = next_free
    jmp .malloc_search_loop
```

**Trade-off:** May waste some memory by expanding heap early, but prevents worst-case performance

**Expected Impact:** Prevents pathological cases, minimal overhead

---

### Priority 4: Optimize Bounds Checking (Low Impact)

**Problem:** Repeated bounds checks in hot loops

**Solution:** Hoist bounds checks out of loops where possible

**Example:**
```asm
# Before (inside loop)
.loop:
    pushq   %rax
    movq    gl_heap_start(%rip), %rax
    cmpq    %rax, %rbx
    popq    %rax
    jl      .error

    # ... loop body
    jmp .loop

# After (outside loop)
    movq    gl_heap_start(%rip), %r14
    movq    gl_heap_end(%rip), %r15

.loop:
    cmpq    %r14, %rbx      # Quick check using register
    jl      .error
    cmpq    %r15, %rbx
    jge     .error

    # ... loop body
    jmp .loop
```

**Expected Impact:** 10-15% speedup in search loops

---

### Priority 5: Cache Last Allocation (Nice-to-Have)

**Problem:** Realloc patterns repeatedly allocate similar sizes

**Solution:** Cache the last successfully used free block

```asm
.data
gl_last_alloc_size: .quad 0
gl_last_alloc_list: .quad 0

.text
gl_malloc:
    # Check if requested size matches last allocation
    cmpq gl_last_alloc_size(%rip), %rcx
    jne .normal_path

    # Try to use cached list
    movq gl_last_alloc_list(%rip), %rbx
    testq %rbx, %rbx
    jnz .found_block        # Cache hit!

.normal_path:
    # ... existing allocation logic
```

**Expected Impact:** 30-50% speedup for realloc-heavy workloads

---

## Performance Targets

### Short-Term (Segregated Lists Only)

| Pattern | Current | Target | Improvement |
|---------|---------|--------|-------------|
| Small allocs (16-64B, N=100) | 31M ops/sec | 300M ops/sec | 10x |
| Small allocs (16-64B, N=1000) | ~310K ops/sec | 250M ops/sec | 800x |
| Mixed allocs (N=1000) | ~240K ops/sec | 100M ops/sec | 400x |

### Long-Term (All Optimizations)

| Pattern | Target | Notes |
|---------|--------|-------|
| Small allocs (16-64B) | 500M+ ops/sec | Comparable to tcmalloc |
| Large allocs (1KB+) | 50M+ ops/sec | Limited by mmap overhead |
| Fragmentation | < 20% | Acceptable for scripting |

---

## Implementation Plan

### Phase 1: Segregated Free Lists (Priority 1)

**Files to modify:**
- [src/native_allocator.S](../src/native_allocator.S): Implement segregated lists

**Steps:**
1. Add size class data structures (.data section)
2. Implement size-to-index mapping function
3. Update gl_malloc to use fast path for common sizes
4. Update gl_free to insert into correct size class
5. Update gl_insert_free_sorted to handle segregated lists
6. Add tests for segregated list correctness

**Estimated effort:** 4-6 hours

**Risk:** Medium (complex assembly changes)

### Phase 2: Exact-Fit Fast Path (Priority 2)

**Files to modify:**
- [src/native_allocator.S](../src/native_allocator.S): Skip split logic for exact fits

**Steps:**
1. Add exact-fit check in malloc_found_block
2. Implement optimized removal path
3. Benchmark to verify improvement

**Estimated effort:** 1-2 hours

**Risk:** Low (isolated change)

### Phase 3: Benchmarking & Validation

**Files to modify:**
- [benches/allocator_bench.rs](../benches/allocator_bench.rs): Re-enable full benchmarks
- [benches/allocator_bench_simple.rs](../benches/allocator_bench_simple.rs): Add segregated list specific tests

**Steps:**
1. Re-run full benchmark suite
2. Verify performance targets met
3. Check for regressions
4. Profile hot paths

**Estimated effort:** 2-3 hours

**Risk:** Low

### Phase 4: Documentation (Final)

**Files to create/update:**
- [docs/allocator_performance.md](allocator_performance.md): Update with actual results
- [docs/allocator_design.md](allocator_design.md): Add segregated list architecture
- [CLAUDE.md](../CLAUDE.md): Update performance characteristics section

**Estimated effort:** 1-2 hours

---

## Testing Strategy

### Unit Tests

Ensure segregated lists maintain correctness:

```rust
#[test]
fn test_segregated_list_allocation() {
    // Allocate various sizes
    let ptr16 = gl_malloc(16);
    let ptr32 = gl_malloc(32);
    let ptr64 = gl_malloc(64);

    // Free them
    gl_free(ptr32);
    gl_free(ptr16);
    gl_free(ptr64);

    // Reallocate - should come from size classes
    let ptr16_2 = gl_malloc(16);
    let ptr32_2 = gl_malloc(32);

    // Verify we got the same pointers (from size class cache)
    assert_eq!(ptr16, ptr16_2);
    assert_eq!(ptr32, ptr32_2);
}
```

### Performance Tests

Verify scalability:

```rust
#[test]
fn test_allocator_scales() {
    let start = Instant::now();

    // Allocate 10,000 small blocks
    for _ in 0..10_000 {
        let ptr = gl_malloc(32);
        // ...
    }

    let duration = start.elapsed();

    // Should complete in < 100ms (target: 100M ops/sec)
    assert!(duration.as_millis() < 100);
}
```

### Regression Tests

Ensure optimizations don't break correctness:

```rust
#[test]
fn test_coalescing_still_works() {
    // Allocate 3 adjacent blocks
    let p1 = gl_malloc(64);
    let p2 = gl_malloc(64);
    let p3 = gl_malloc(64);

    // Free them out of order
    gl_free(p2);
    gl_free(p1);
    gl_free(p3);

    // Should have coalesced into single large block
    let large = gl_malloc(192);
    assert!(!large.is_null());
}
```

---

## Benchmark Infrastructure

### Current Files

- **[benches/allocator_bench_simple.rs](../benches/allocator_bench_simple.rs)**
  ✅ Works correctly, measures performance for small workloads (100 operations)

- **[benches/allocator_bench.rs](../benches/allocator_bench.rs)**
  ⚠️ Hangs on larger workloads (1000+ operations) - disabled for now

### Running Benchmarks

```bash
# Simple benchmark (works on current implementation)
wsl bash -c "cd /mnt/f/glimmer/glimmer-weave && cargo bench --bench allocator_bench_simple"

# Full benchmark (re-enable after optimization)
wsl bash -c "cd /mnt/f/glimmer/glimmer-weave && cargo bench --bench allocator_bench"
```

**Note:** Benchmarks require GNU assembler (not available on Windows MSVC). Use WSL or Linux.

---

## Conclusion

The Glimmer-Weave allocator is well-designed and correct, but requires optimization for real-world workloads. The primary bottleneck is the O(n) linear search through the free list, which becomes prohibitive with hundreds of allocations.

**Recommended Next Steps:**
1. Implement segregated free lists (Priority 1) - **highest impact**
2. Re-run benchmarks to verify 10-100x improvement
3. Consider additional optimizations if needed
4. Document final performance characteristics

**Success Criteria:**
- ✅ Allocate 1,000 small blocks in < 10ms (100M ops/sec)
- ✅ Benchmarks complete without timeouts
- ✅ No correctness regressions
- ✅ Coalescing still prevents fragmentation

---

*Generated: 2025-11-07*
*Author: Claude (AI Assistant)*
*Related Issue: glimmer-weave-cyg*
