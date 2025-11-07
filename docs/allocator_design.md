# Free-List Heap Allocator Design

**Purpose:** Custom heap allocator for Glimmer-Weave native x86-64 codegen, enabling struct and closure allocation without external dependencies.

---

## Design Goals

1. **No external dependencies** - Self-contained, no libc required
2. **Simple and correct** - Prioritize correctness over performance
3. **Embeddable** - Works in AethelOS kernel context (no_std)
4. **Debuggable** - Clear structure for troubleshooting
5. **Coalescable** - Merge adjacent free blocks to reduce fragmentation

---

## Memory Layout

### Block Header Format

Each allocated or free block has an 8-byte header:

```
+------------------+
| size | flags     |  8 bytes (header)
+------------------+
| user data...     |  N bytes
+------------------+
```

**Header Encoding (64-bit):**
```
63                                    1  0
+-------------------------------------+--+
|          size (in bytes)            |F |
+-------------------------------------+--+
```

- **Bits 63-1**: Block size (excluding header), in bytes
- **Bit 0**: Free flag (1 = free, 0 = allocated)

**Why this encoding?**
- Size is always 8-byte aligned, so lowest 3 bits are always 0
- We only need 1 bit for free/allocated status
- Single 64-bit load/store for atomic header operations

### Free Block Structure

Free blocks are linked together in a doubly-linked free list:

```
+------------------+
| size | FREE=1    |  8 bytes (header)
+------------------+
| next_free        |  8 bytes (pointer to next free block)
+------------------+
| prev_free        |  8 bytes (pointer to prev free block)
+------------------+
| unused...        |  Remaining bytes
+------------------+
```

**Minimum free block size:** 24 bytes (header + next + prev)

### Allocated Block Structure

Allocated blocks only have the header:

```
+------------------+
| size | FREE=0    |  8 bytes (header)
+------------------+
| user data...     |  N bytes (user requested + alignment)
+------------------+
```

**Minimum allocated block size:** 8 bytes (header only for zero-byte allocations)

---

## Data Structures

### Global Allocator State

```asm
.data
.align 8

# Free list head (pointer to first free block, or NULL)
gl_free_list_head:
    .quad 0

# Heap start address (set during initialization)
gl_heap_start:
    .quad 0

# Heap current end (grows upward with sbrk)
gl_heap_end:
    .quad 0

# Total allocated bytes (for stats/debugging)
gl_allocated_bytes:
    .quad 0
```

### Free List Organization

- **Singly-linked list** (simpler) vs **doubly-linked list** (faster removal)
  - **Choice:** Doubly-linked for O(1) removal during coalescing

- **Sorted by address** vs **Insertion order**
  - **Choice:** Sorted by address - enables efficient coalescing

- **Segregated lists** (different lists for different sizes) vs **Single list**
  - **Choice:** Single list for MVP, can optimize later

---

## Algorithms

### 1. Initialization (`gl_init_allocator`)

Called once at program startup to initialize the heap.

```
gl_init_allocator():
    1. Request initial heap from OS (sbrk/mmap)
       - Request 64KB initial heap (configurable)

    2. Create initial free block:
       - header.size = heap_size - 8
       - header.free = 1
       - next_free = NULL
       - prev_free = NULL

    3. Set gl_free_list_head = heap_start

    4. Set gl_heap_start = heap_start

    5. Set gl_heap_end = heap_start + heap_size
```

**In x86-64:**
```asm
gl_init_allocator:
    # Request 64KB from OS
    movq $65536, %rdi          # size = 64KB
    call gl_request_memory      # Returns address in %rax

    # Initialize free list
    movq %rax, gl_heap_start
    movq %rax, gl_free_list_head

    # Set heap end
    leaq 65536(%rax), %rbx
    movq %rbx, gl_heap_end

    # Create initial free block header
    movq $65528, %rcx           # size = 64KB - 8
    orq  $1, %rcx               # set FREE bit
    movq %rcx, 0(%rax)          # store header

    # Clear next/prev pointers
    movq $0, 8(%rax)            # next = NULL
    movq $0, 16(%rax)           # prev = NULL

    ret
```

### 2. Allocation (`gl_malloc`)

**Input:** %rdi = requested size (in bytes)
**Output:** %rax = pointer to allocated memory (or NULL on failure)

**Algorithm:**

```
gl_malloc(size):
    1. Align size to 8-byte boundary
       actual_size = (size + 7) & ~7

    2. Search free list for suitable block (first-fit)
       current = gl_free_list_head
       while current != NULL:
           if current.size >= actual_size:
               # Found suitable block
               goto allocate_from_block
           current = current.next_free

    3. No suitable block found - request more memory
       new_block = gl_expand_heap(actual_size + 8)
       if new_block == NULL:
           return NULL  # Out of memory
       # new_block is now a free block, use it
       current = new_block

    4. allocate_from_block:
       # Should we split the block?
       remainder = current.size - actual_size - 8
       if remainder >= 24:  # Enough for a free block
           # Split block
           new_free = current + 8 + actual_size
           new_free.size = remainder - 8
           new_free.free = 1
           new_free.next = current.next
           new_free.prev = current.prev

           # Insert new_free into free list
           gl_insert_free(new_free)

           # Update current block size
           current.size = actual_size

       # Remove current from free list
       gl_remove_free(current)

       # Mark as allocated
       current.free = 0

       # Return pointer past header
       return current + 8
```

**Pseudocode for first-fit search:**
```asm
gl_malloc:
    # Align size to 8 bytes
    movq %rdi, %rcx
    addq $7, %rcx
    andq $-8, %rcx              # rcx = aligned size

    # Search free list
    movq gl_free_list_head, %rbx
.search_loop:
    testq %rbx, %rbx            # if (rbx == NULL)
    jz .no_block_found

    # Check if block is large enough
    movq 0(%rbx), %rdx          # Load header
    andq $-2, %rdx              # Clear FREE bit to get size
    cmpq %rcx, %rdx             # if (size >= requested)
    jge .found_block

    # Move to next block
    movq 8(%rbx), %rbx          # rbx = next_free
    jmp .search_loop

.found_block:
    # ... allocation logic
```

### 3. Deallocation (`gl_free`)

**Input:** %rdi = pointer to allocated memory
**Output:** none

**Algorithm:**

```
gl_free(ptr):
    1. Validate pointer
       if ptr == NULL:
           return

       block = ptr - 8  # Get header address

       if block < gl_heap_start || block >= gl_heap_end:
           # Invalid pointer - panic or return
           return

    2. Mark block as free
       block.free = 1

    3. Try to coalesce with previous physical block
       prev_block = gl_find_prev_physical(block)
       if prev_block != NULL && prev_block.free == 1:
           # Merge with previous
           prev_block.size += block.size + 8
           block = prev_block

    4. Try to coalesce with next physical block
       next_block = block + block.size + 8
       if next_block < gl_heap_end && next_block.free == 1:
           # Merge with next
           gl_remove_free(next_block)
           block.size += next_block.size + 8

    5. Insert block into free list (sorted by address)
       gl_insert_free(block)
```

### 4. Coalescing (`gl_coalesce`)

**Purpose:** Merge adjacent free blocks to reduce fragmentation.

**Called by:** `gl_free` after marking block as free

**Algorithm:**

```
gl_coalesce(block):
    # Coalesce with next block
    next_block = block + block.size + 8
    if next_block < gl_heap_end:
        next_header = *next_block
        if next_header.free == 1:
            # Merge with next
            gl_remove_free(next_block)
            block.size += next_block.size + 8
            # Update header
            *(block) = (block.size | 1)

    # Coalesce with previous block
    # This requires walking the heap from start (slow)
    # Or maintaining doubly-linked ALL blocks (memory overhead)
    # For MVP: Only coalesce forward, not backward
```

**Optimization:** For backward coalescing, we need to track previous physical block. Options:

1. **Footer approach:** Store size at end of each block
   - Pros: O(1) backward coalescing
   - Cons: 8 bytes overhead per block

2. **Walk from start:** Traverse heap to find previous block
   - Pros: No overhead
   - Cons: O(n) performance

**Choice for MVP:** Forward coalescing only (simpler), add backward coalescing later if needed.

### 5. Heap Expansion (`gl_expand_heap`)

**Purpose:** Request more memory from OS when free list is exhausted.

```
gl_expand_heap(min_size):
    1. Calculate expansion size
       expansion = max(min_size, 4096)  # At least 4KB
       expansion = align_to_page(expansion)

    2. Request memory from OS
       new_mem = sbrk(expansion)  # or mmap
       if new_mem == NULL:
           return NULL  # OOM

    3. Create free block from new memory
       new_block = new_mem
       new_block.size = expansion - 8
       new_block.free = 1

    4. Try to coalesce with last block in heap
       last_block = gl_heap_end - last_size - 8
       if last_block.free == 1:
           # Merge
           last_block.size += expansion
           return last_block

    5. Otherwise, insert new block into free list
       gl_insert_free(new_block)
       gl_heap_end += expansion
       return new_block
```

---

## Helper Functions

### `gl_remove_free(block)`
Remove a block from the free list.

```asm
gl_remove_free:
    # Input: %rdi = block address

    # Load prev and next
    movq 16(%rdi), %rax         # prev_free
    movq 8(%rdi), %rbx          # next_free

    # If prev != NULL: prev.next = next
    testq %rax, %rax
    jz .no_prev
    movq %rbx, 8(%rax)
.no_prev:

    # If next != NULL: next.prev = prev
    testq %rbx, %rbx
    jz .no_next
    movq %rax, 16(%rbx)
.no_next:

    # If block was head: update head
    cmpq gl_free_list_head, %rdi
    jne .done
    movq %rbx, gl_free_list_head
.done:
    ret
```

### `gl_insert_free(block)`
Insert a block into the free list (sorted by address).

```asm
gl_insert_free:
    # Input: %rdi = block to insert

    # Find insertion point
    movq gl_free_list_head, %rbx
    xorq %rcx, %rcx              # prev = NULL

.search:
    testq %rbx, %rbx             # if (current == NULL)
    jz .insert_at_end

    cmpq %rdi, %rbx              # if (block < current)
    jg .insert_before

    # Move to next
    movq %rbx, %rcx              # prev = current
    movq 8(%rbx), %rbx           # current = current.next
    jmp .search

.insert_before:
    # Insert between prev and current
    movq %rbx, 8(%rdi)           # block.next = current
    movq %rcx, 16(%rdi)          # block.prev = prev

    testq %rcx, %rcx
    jz .update_head
    movq %rdi, 8(%rcx)           # prev.next = block
    jmp .update_current

.update_head:
    movq %rdi, gl_free_list_head

.update_current:
    movq %rdi, 16(%rbx)          # current.prev = block
    ret

.insert_at_end:
    # Append to list
    movq $0, 8(%rdi)             # block.next = NULL
    movq %rcx, 16(%rdi)          # block.prev = prev

    testq %rcx, %rcx
    jz .make_head
    movq %rdi, 8(%rcx)           # prev.next = block
    ret

.make_head:
    movq %rdi, gl_free_list_head
    ret
```

---

## OS Interface

### Memory Request (sbrk/mmap)

For portability, we'll implement a thin abstraction layer:

```asm
# Request memory from OS
# Input: %rdi = size in bytes
# Output: %rax = pointer to memory (or NULL on failure)
gl_request_memory:
    # On Linux: use mmap
    movq %rdi, %rsi              # length = size
    movq $9, %rax                # syscall: mmap
    xorq %rdi, %rdi              # addr = NULL (let kernel choose)
    movq $3, %rdx                # prot = PROT_READ | PROT_WRITE
    movq $34, %r10               # flags = MAP_PRIVATE | MAP_ANONYMOUS
    movq $-1, %r8                # fd = -1
    xorq %r9, %r9                # offset = 0
    syscall

    # Check for error (MAP_FAILED = -1)
    cmpq $-1, %rax
    je .map_failed
    ret

.map_failed:
    xorq %rax, %rax              # Return NULL
    ret
```

**Alternative (sbrk):**
```asm
gl_request_memory:
    # On Linux: use brk syscall
    movq $12, %rax               # syscall: brk
    xorq %rdi, %rdi              # NULL = query current break
    syscall

    # rax now contains current break
    movq %rax, %rbx              # Save old break

    # Expand heap
    addq %rdi, %rax              # new_break = old + size
    movq $12, %rax               # syscall: brk
    syscall

    # Check if successful
    cmpq %rax, %rbx
    je .brk_failed

    movq %rbx, %rax              # Return old break (start of new memory)
    ret

.brk_failed:
    xorq %rax, %rax              # Return NULL
    ret
```

**Choice:** Use `mmap` for MVP (more portable, easier to debug)

---

## Memory Map Example

After several allocations and frees:

```
Heap Start                                              Heap End
↓                                                            ↓
+--------+--------+--------+--------+--------+--------+------+
| Alloc  | Free   | Alloc  | Free   | Alloc  | Free   | Free |
| 48B    | 32B    | 64B    | 128B   | 16B    | 256B   | ...  |
+--------+--------+--------+--------+--------+--------+------+
         ↑                  ↑                  ↑
         |                  |                  |
    Free List: -------------+------------------+

Free List (sorted by address):
head → [32B @ 0x60] → [128B @ 0xA0] → [256B @ 0x150] → NULL
```

---

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| `malloc` | O(n) | Linear search through free list (first-fit) |
| `free` | O(n) | Insert into sorted free list |
| `coalesce` | O(1) | Forward coalescing only (MVP) |
| `expand_heap` | O(1) | Single syscall + O(n) free list insert |

**Optimization opportunities:**
1. **Segregated free lists** - O(1) malloc for common sizes
2. **Best-fit** instead of first-fit - Better fragmentation
3. **Boundary tags** - Enable O(1) backward coalescing
4. **Thread-local caches** - Reduce contention in concurrent code

---

## Testing Strategy

### Unit Tests

1. **Basic allocation:**
   - Allocate single block
   - Verify pointer is non-NULL
   - Verify memory is writable

2. **Multiple allocations:**
   - Allocate 100 blocks
   - Verify all unique pointers
   - Write to each block

3. **Deallocation:**
   - Allocate + free single block
   - Verify block returns to free list
   - Reallocate and verify same block used

4. **Coalescing:**
   - Allocate 3 adjacent blocks
   - Free middle block (no coalesce)
   - Free left block (coalesce with middle)
   - Verify single large free block

5. **Heap expansion:**
   - Allocate blocks until heap expands
   - Verify new blocks allocated from expanded region
   - Check gl_heap_end increased

6. **Fragmentation:**
   - Alternating alloc/free pattern
   - Verify free list has multiple small blocks
   - Large allocation forces heap expansion

7. **Edge cases:**
   - Zero-byte allocation
   - Huge allocation (larger than heap)
   - Free NULL pointer
   - Double free detection (optional)

### Integration Tests

1. **Struct allocation:**
   - Compile Glimmer-Weave struct code
   - Execute with allocator
   - Verify struct fields accessible

2. **Stress test:**
   - Random alloc/free for 10,000 iterations
   - Verify no crashes
   - Check heap stats (total allocated, fragmentation)

---

## Assembly File Structure

```
src/native_allocator.S  (new file)

.data
    gl_free_list_head:  .quad 0
    gl_heap_start:      .quad 0
    gl_heap_end:        .quad 0
    gl_allocated_bytes: .quad 0

.text
.globl gl_malloc
.globl gl_free
.globl gl_init_allocator

gl_malloc:
    # ... implementation

gl_free:
    # ... implementation

gl_init_allocator:
    # ... implementation

gl_remove_free:
    # ... helper

gl_insert_free:
    # ... helper

gl_expand_heap:
    # ... helper

gl_request_memory:
    # ... syscall wrapper
```

---

## Next Steps

1. **Implement `gl_malloc`** (glimmer-weave-if2)
   - Core allocation logic
   - First-fit search
   - Block splitting

2. **Implement `gl_free`** (glimmer-weave-n3n)
   - Deallocation logic
   - Free list insertion

3. **Implement coalescing** (glimmer-weave-afi)
   - Forward coalescing
   - Optimize fragmentation

4. **Write tests** (glimmer-weave-bwo)
   - Unit tests for each function
   - Integration with codegen

5. **Integrate with codegen** (glimmer-weave-e2u)
   - Replace malloc@PLT with gl_malloc
   - Link native_allocator.S with generated code

6. **Optimize and benchmark** (glimmer-weave-cyg)
   - Measure performance
   - Add segregated lists if needed
