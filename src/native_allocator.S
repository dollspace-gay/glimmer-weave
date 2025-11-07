# Free-List Heap Allocator for Glimmer-Weave
# x86-64 Assembly (AT&T Syntax)
#
# Implements a simple free-list based heap allocator with:
# - First-fit allocation strategy
# - Doubly-linked free list (sorted by address)
# - Forward coalescing on free
# - mmap-based heap expansion
#
# Public API:
#   gl_malloc(size) -> pointer (or NULL on failure)
#   gl_free(pointer)
#   gl_init_allocator()

.data
.align 8

# Global allocator state
gl_free_list_head:
    .quad 0                     # Pointer to first free block (NULL initially)

gl_heap_start:
    .quad 0                     # Start of heap region

gl_heap_end:
    .quad 0                     # End of heap region (grows upward)

gl_allocated_bytes:
    .quad 0                     # Total bytes allocated (for debugging/stats)

gl_initialized:
    .quad 0                     # Initialization flag (0 = not initialized)

# Constants
.equ INITIAL_HEAP_SIZE, 65536   # 64KB initial heap
.equ MIN_BLOCK_SIZE, 24         # Minimum free block size (header + next + prev)
.equ HEADER_SIZE, 8             # Size of block header
.equ FREE_BIT, 1                # Bit 0 of header = free flag

# Linux syscall numbers
.equ SYS_MMAP, 9
.equ SYS_BRK, 12

# mmap flags
.equ PROT_READ, 1
.equ PROT_WRITE, 2
.equ MAP_PRIVATE, 2
.equ MAP_ANONYMOUS, 32

.text
.globl gl_malloc
.globl gl_free
.globl gl_init_allocator

#==============================================================================
# gl_malloc - Allocate memory from heap
#
# Input:  rdi = requested size (in bytes)
# Output: rax = pointer to allocated memory (or NULL on failure)
#
# Algorithm:
#   1. Align size to 8 bytes
#   2. Search free list for suitable block (first-fit)
#   3. If found: split if necessary, remove from free list, return
#   4. If not found: expand heap, use new block
#==============================================================================
gl_malloc:
    pushq   %rbp
    movq    %rsp, %rbp
    pushq   %rbx                # Preserve callee-saved registers
    pushq   %r12
    pushq   %r13
    pushq   %r14
    pushq   %r15

    # Check if allocator is initialized
    movq    gl_initialized(%rip), %rax
    testq   %rax, %rax
    jnz     .malloc_skip_init

    # Initialize allocator on first call
    pushq   %rdi                # Save requested size
    call    gl_init_allocator
    popq    %rdi                # Restore requested size

.malloc_skip_init:
    # Align requested size to 8-byte boundary
    movq    %rdi, %rcx
    addq    $7, %rcx
    andq    $-8, %rcx           # rcx = aligned size

    # Check for zero-byte allocation
    testq   %rcx, %rcx
    jz      .malloc_fail        # Return NULL for zero bytes

    # Enforce minimum allocation size
    # We need at least 16 bytes to store next/prev pointers during clearing
    # (MIN_BLOCK_SIZE - HEADER_SIZE = 24 - 8 = 16)
    cmpq    $(MIN_BLOCK_SIZE - HEADER_SIZE), %rcx
    jge     .malloc_size_ok
    movq    $(MIN_BLOCK_SIZE - HEADER_SIZE), %rcx

.malloc_size_ok:
    # Save aligned size for later
    movq    %rcx, %r12          # r12 = requested aligned size

    # Search free list for suitable block (first-fit)
    movq    gl_free_list_head(%rip), %rbx   # rbx = current block

.malloc_search_loop:
    testq   %rbx, %rbx          # if (current == NULL)
    jz      .malloc_expand_heap # No suitable block, expand heap

    # Validate rbx is within heap bounds before dereferencing
    pushq   %rax
    movq    gl_heap_start(%rip), %rax
    cmpq    %rax, %rbx
    popq    %rax
    jl      .malloc_expand_heap  # rbx < heap_start, treat as end of list

    pushq   %rax
    movq    gl_heap_end(%rip), %rax
    cmpq    %rax, %rbx
    popq    %rax
    jge     .malloc_expand_heap  # rbx >= heap_end, treat as end of list

    # Validate pointer consistency (FD->bk == P check)
    pushq   %rdi
    movq    %rbx, %rdi
    call    gl_validate_free_block
    popq    %rdi
    testq   %rax, %rax
    jz      .malloc_expand_heap  # Corruption detected, skip this block

    # Load block header
    movq    0(%rbx), %rdx       # rdx = header (size | FREE)

    # Extract size (clear FREE bit)
    movq    %rdx, %r13
    andq    $-2, %r13           # r13 = block size (excluding header)

    # Check if block is large enough
    cmpq    %r12, %r13          # if (block_size >= requested_size)
    jge     .malloc_found_block

    # Move to next free block
    movq    8(%rbx), %rbx       # rbx = next_free
    jmp     .malloc_search_loop

.malloc_found_block:
    # Found suitable block in rbx, size in r13, requested size in r12

    # Save current block's next/prev before removing from list
    movq    8(%rbx), %r14       # r14 = current.next
    movq    16(%rbx), %r15      # r15 = current.prev

    # Should we split the block?
    # remainder = block_size - requested_size - HEADER_SIZE
    movq    %r13, %rax
    subq    %r12, %rax
    subq    $HEADER_SIZE, %rax  # rax = remainder

    cmpq    $MIN_BLOCK_SIZE, %rax
    jl      .malloc_no_split    # Not enough for split, use whole block

    # Split block: create new free block from remainder
    # new_free_block = current + HEADER_SIZE + requested_size
    leaq    HEADER_SIZE(%rbx, %r12, 1), %rcx    # rcx = new free block address

    # Set new free block header
    # new_block.size = remainder - HEADER_SIZE
    subq    $HEADER_SIZE, %rax
    orq     $FREE_BIT, %rax     # Set FREE bit
    movq    %rax, 0(%rcx)       # Store header

    # Initialize next/prev to NULL (will be set by insert_free_sorted)
    movq    $0, 8(%rcx)         # new_block.next = NULL
    movq    $0, 16(%rcx)        # new_block.prev = NULL

    # Save remainder block address on stack for later insertion
    pushq   %rcx                # Push remainder block address
    jmp     .malloc_split_done

.malloc_no_split:
    # No split, push NULL so we know not to insert later
    pushq   $0                  # Push NULL (no remainder)

.malloc_split_done:
    # Stack now contains: remainder address (or NULL if no split)
    # Update current block size
    movq    %r12, %r13          # Use only requested size

    # Remove current block from free list
    # rbx = block to remove
    # r14 = current.next (saved earlier)
    # r15 = current.prev (saved earlier)

    # If prev != NULL and valid: prev.next = next
    testq   %r15, %r15
    jz      .malloc_remove_no_prev

    # Validate r15 is within heap bounds
    pushq   %rax
    movq    gl_heap_start(%rip), %rax
    cmpq    %rax, %r15
    popq    %rax
    jl      .malloc_remove_no_prev

    pushq   %rax
    movq    gl_heap_end(%rip), %rax
    cmpq    %rax, %r15
    popq    %rax
    jge     .malloc_remove_no_prev

    movq    %r14, 8(%r15)       # prev.next = current.next

.malloc_remove_no_prev:
    # If next != NULL and valid: next.prev = prev
    testq   %r14, %r14
    jz      .malloc_remove_no_next

    # Validate r14 is within heap bounds
    pushq   %rax
    movq    gl_heap_start(%rip), %rax
    cmpq    %rax, %r14
    popq    %rax
    jl      .malloc_remove_no_next

    pushq   %rax
    movq    gl_heap_end(%rip), %rax
    cmpq    %rax, %r14
    popq    %rax
    jge     .malloc_remove_no_next

    movq    %r15, 16(%r14)      # next.prev = current.prev

.malloc_remove_no_next:
    # If block was head: update head
    cmpq    gl_free_list_head(%rip), %rbx
    jne     .malloc_remove_done
    movq    %r14, gl_free_list_head(%rip)    # head = current.next

.malloc_remove_done:
    # Mark block as allocated (clear FREE bit)
    movq    %r13, 0(%rbx)       # Store size with FREE=0

    # Update statistics
    addq    %r13, gl_allocated_bytes(%rip)

    # Clear next/prev pointers in allocated block to prevent stale values
    # This is CRITICAL: when the block is later freed, these stale pointers
    # would cause corruption in the free list
    movq    $0, 8(%rbx)         # Clear next pointer
    movq    $0, 16(%rbx)        # Clear prev pointer

    # Pop remainder block address from stack and insert it into free list
    popq    %rcx                # rcx = remainder block (or NULL)
    testq   %rcx, %rcx
    jz      .malloc_no_remainder_to_insert

    # Save rbx and rax (current block and return value)
    pushq   %rbx
    pushq   %rax

    # Insert remainder block into free list
    movq    %rcx, %rdi
    call    gl_insert_free_sorted

    # Restore rbx and rax
    popq    %rax
    popq    %rbx

.malloc_no_remainder_to_insert:
    # Return pointer past header
    leaq    HEADER_SIZE(%rbx), %rax

    # Restore registers and return
    popq    %r15
    popq    %r14
    popq    %r13
    popq    %r12
    popq    %rbx
    popq    %rbp
    ret

.malloc_expand_heap:
    # No suitable block found - expand heap
    # r12 = requested size

    # Calculate expansion size (at least requested + 2*header, aligned to 4KB)
    # We need: expansion >= requested + header + safety_margin
    # This ensures block_size (expansion - header) >= requested
    movq    %r12, %rdi
    addq    $HEADER_SIZE, %rdi  # requested + header
    addq    $HEADER_SIZE, %rdi  # Add another header size for safety
    addq    $4095, %rdi         # Round up to page boundary
    andq    $-4096, %rdi        # rdi = expansion size (rounded up to 4KB)

    # Request memory from OS
    call    gl_request_memory
    testq   %rax, %rax
    jz      .malloc_fail        # OOM - return NULL

    # rax = new memory region
    movq    %rax, %rbx          # rbx = new block

    # Calculate block size (expansion - header)
    movq    %rdi, %r13
    subq    $HEADER_SIZE, %r13

    # Create free block header
    orq     $FREE_BIT, %r13
    movq    %r13, 0(%rbx)

    # Clear next/prev pointers
    movq    $0, 8(%rbx)
    movq    $0, 16(%rbx)

    # Add to free list
    pushq   %rdi                # Save expansion size
    movq    %rbx, %rdi
    call    gl_insert_free_sorted
    popq    %rdi                # Restore expansion size

    # Update heap bounds
    # Check if new block extends our tracked heap range
    # new_block_end = rbx + rdi
    movq    %rbx, %rax
    addq    %rdi, %rax          # rax = new block end address

    # Update heap_start if this block is before current start
    movq    gl_heap_start(%rip), %rcx
    cmpq    %rcx, %rbx
    jge     .expand_check_end   # rbx >= heap_start, don't update start
    movq    %rbx, gl_heap_start(%rip)

.expand_check_end:
    # Update heap_end if this block extends past current end
    movq    gl_heap_end(%rip), %rcx
    cmpq    %rcx, %rax
    jle     .expand_bounds_done # rax <= heap_end, don't update end
    movq    %rax, gl_heap_end(%rip)

.expand_bounds_done:

    # Now retry search from the beginning
    # The newly inserted block will be found and properly size-checked
    jmp     .malloc_search_loop

.malloc_fail:
    xorq    %rax, %rax          # Return NULL
    popq    %r15
    popq    %r14
    popq    %r13
    popq    %r12
    popq    %rbx
    popq    %rbp
    ret

#==============================================================================
# gl_init_allocator - Initialize the heap allocator
#
# Called automatically on first gl_malloc, or can be called explicitly.
# Requests initial heap from OS and creates first free block.
#==============================================================================
gl_init_allocator:
    pushq   %rbp
    movq    %rsp, %rbp
    pushq   %rbx

    # Check if already initialized
    movq    gl_initialized(%rip), %rax
    testq   %rax, %rax
    jnz     .init_done

    # Request initial heap
    movq    $INITIAL_HEAP_SIZE, %rdi
    call    gl_request_memory
    testq   %rax, %rax
    jz      .init_fail

    # rax = heap start
    movq    %rax, gl_heap_start(%rip)
    movq    %rax, gl_free_list_head(%rip)

    # Calculate heap end
    leaq    INITIAL_HEAP_SIZE(%rax), %rbx
    movq    %rbx, gl_heap_end(%rip)

    # Create initial free block
    # size = INITIAL_HEAP_SIZE - HEADER_SIZE
    movq    $INITIAL_HEAP_SIZE, %rcx
    subq    $HEADER_SIZE, %rcx
    orq     $FREE_BIT, %rcx
    movq    %rcx, 0(%rax)       # Store header

    # Clear next/prev pointers
    movq    $0, 8(%rax)
    movq    $0, 16(%rax)

    # Mark as initialized
    movq    $1, gl_initialized(%rip)

.init_done:
    popq    %rbx
    popq    %rbp
    ret

.init_fail:
    # Failed to initialize - fatal error
    # For now, just return and let malloc fail
    popq    %rbx
    popq    %rbp
    ret

#==============================================================================
# gl_request_memory - Request memory from OS via mmap
#
# Input:  rdi = size in bytes
# Output: rax = pointer to memory (or NULL on failure)
#==============================================================================
gl_request_memory:
    pushq   %rbp
    movq    %rsp, %rbp

    # Setup mmap syscall
    # mmap(NULL, size, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0)

    movq    %rdi, %rsi          # length = size
    movq    $SYS_MMAP, %rax     # syscall number
    xorq    %rdi, %rdi          # addr = NULL (let kernel choose)
    movq    $(PROT_READ | PROT_WRITE), %rdx    # prot
    movq    $(MAP_PRIVATE | MAP_ANONYMOUS), %r10   # flags
    movq    $-1, %r8            # fd = -1
    xorq    %r9, %r9            # offset = 0
    syscall

    # Check for error (MAP_FAILED = -1 or very large negative)
    cmpq    $-4096, %rax        # Error if rax in [-4096, -1]
    jae     .request_failed

    # Success - rax contains mapped address
    popq    %rbp
    ret

.request_failed:
    xorq    %rax, %rax          # Return NULL
    popq    %rbp
    ret

#==============================================================================
# gl_insert_free_sorted - Insert block into free list (sorted by address)
#
# Input: rdi = block to insert
# Modifies: rax, rbx, rcx
#==============================================================================
gl_insert_free_sorted:
    pushq   %rbp
    movq    %rsp, %rbp

    # rdi = block to insert
    movq    gl_free_list_head(%rip), %rbx   # rbx = current
    xorq    %rcx, %rcx          # rcx = prev (NULL initially)

.insert_search:
    testq   %rbx, %rbx          # if (current == NULL)
    jz      .insert_at_end

    # Validate current is within bounds
    pushq   %rax
    movq    gl_heap_start(%rip), %rax
    cmpq    %rax, %rbx
    popq    %rax
    jl      .insert_at_end      # Treat invalid as end of list

    pushq   %rax
    movq    gl_heap_end(%rip), %rax
    cmpq    %rax, %rbx
    popq    %rax
    jge     .insert_at_end      # Treat invalid as end of list

    cmpq    %rdi, %rbx          # if (block < current)
    jg      .insert_before

    # Move to next
    movq    %rbx, %rcx          # prev = current
    movq    8(%rbx), %rbx       # current = current.next
    jmp     .insert_search

.insert_before:
    # Check if prev is physically adjacent (backward coalescing)
    testq   %rcx, %rcx
    jz      .insert_no_coalesce_before   # No prev block, can't coalesce

    # Validate prev block header before accessing
    pushq   %rdx
    pushq   %r15
    movq    gl_heap_start(%rip), %r15
    cmpq    %r15, %rcx
    jl      .insert_skip_coalesce_before2  # prev < heap_start
    movq    gl_heap_end(%rip), %r15
    cmpq    %r15, %rcx
    jge     .insert_skip_coalesce_before2  # prev >= heap_end

    # Validate new block pointer before accessing
    movq    gl_heap_start(%rip), %r15
    cmpq    %r15, %rdi
    jl      .insert_skip_coalesce_before2  # new block < heap_start
    movq    gl_heap_end(%rip), %r15
    cmpq    %r15, %rdi
    jge     .insert_skip_coalesce_before2  # new block >= heap_end

    # Calculate where prev ends: prev_addr + HEADER_SIZE + prev_size
    movq    0(%rcx), %rax       # Load prev block header
    testq   $FREE_BIT, %rax     # Verify it's free
    jz      .insert_skip_coalesce_before2  # Not free, can't coalesce

    andq    $-2, %rax           # Clear FREE bit to get size

    # Sanity check: size should be reasonable
    cmpq    $0, %rax
    jle     .insert_skip_coalesce_before2  # Invalid size

    leaq    HEADER_SIZE(%rcx, %rax, 1), %rdx    # rdx = prev_end

    # Check if prev ends exactly where new block starts
    cmpq    %rdi, %rdx
    jne     .insert_skip_coalesce_before2    # Not adjacent

    # Backward coalesce: merge new block into prev
    movq    0(%rcx), %rax       # Load prev size
    andq    $-2, %rax           # Clear FREE bit
    movq    0(%rdi), %rdx       # Load new block size
    andq    $-2, %rdx           # Clear FREE bit
    addq    %rdx, %rax          # Add new block size
    addq    $HEADER_SIZE, %rax  # Add header size
    orq     $FREE_BIT, %rax     # Set FREE bit
    movq    %rax, 0(%rcx)       # Update prev block size

    popq    %r15
    popq    %rdx
    # Block merged into prev, no need to insert
    jmp     .insert_done

.insert_skip_coalesce_before2:
    popq    %r15

.insert_skip_coalesce_before:
    popq    %rdx

.insert_no_coalesce_before:
    # Insert between prev and current (no coalescing)
    movq    %rbx, 8(%rdi)       # block.next = current
    movq    %rcx, 16(%rdi)      # block.prev = prev

    testq   %rcx, %rcx
    jz      .insert_update_head
    movq    %rdi, 8(%rcx)       # prev.next = block
    jmp     .insert_update_current

.insert_update_head:
    movq    %rdi, gl_free_list_head(%rip)

.insert_update_current:
    # Update current.prev only if current exists and is valid
    testq   %rbx, %rbx
    jz      .insert_done

    # Validate that rbx is within heap bounds
    pushq   %rax
    movq    gl_heap_start(%rip), %rax
    cmpq    %rax, %rbx
    popq    %rax
    jl      .insert_done        # rbx < heap_start, invalid

    pushq   %rax
    movq    gl_heap_end(%rip), %rax
    cmpq    %rax, %rbx
    popq    %rax
    jge     .insert_done        # rbx >= heap_end, invalid

    movq    %rdi, 16(%rbx)      # current.prev = block

.insert_done:
    popq    %rbp
    ret

.insert_at_end:
    # Check if prev is physically adjacent (backward coalescing)
    testq   %rcx, %rcx
    jz      .insert_no_coalesce_end   # No prev block, can't coalesce

    # Validate prev block header before accessing
    pushq   %rdx
    pushq   %r15
    movq    gl_heap_start(%rip), %r15
    cmpq    %r15, %rcx
    jl      .insert_skip_coalesce_end2  # prev < heap_start
    movq    gl_heap_end(%rip), %r15
    cmpq    %r15, %rcx
    jge     .insert_skip_coalesce_end2  # prev >= heap_end

    # Validate new block pointer before accessing
    movq    gl_heap_start(%rip), %r15
    cmpq    %r15, %rdi
    jl      .insert_skip_coalesce_end2  # new block < heap_start
    movq    gl_heap_end(%rip), %r15
    cmpq    %r15, %rdi
    jge     .insert_skip_coalesce_end2  # new block >= heap_end

    # Calculate where prev ends: prev_addr + HEADER_SIZE + prev_size
    movq    0(%rcx), %rax       # Load prev block header
    testq   $FREE_BIT, %rax     # Verify it's free
    jz      .insert_skip_coalesce_end2  # Not free, can't coalesce

    andq    $-2, %rax           # Clear FREE bit to get size

    # Sanity check: size should be reasonable
    cmpq    $0, %rax
    jle     .insert_skip_coalesce_end2  # Invalid size

    leaq    HEADER_SIZE(%rcx, %rax, 1), %rdx    # rdx = prev_end

    # Check if prev ends exactly where new block starts
    cmpq    %rdi, %rdx
    jne     .insert_skip_coalesce_end    # Not adjacent

    # Backward coalesce: merge new block into prev
    movq    0(%rcx), %rax       # Load prev size
    andq    $-2, %rax           # Clear FREE bit
    movq    0(%rdi), %rdx       # Load new block size
    andq    $-2, %rdx           # Clear FREE bit
    addq    %rdx, %rax          # Add new block size
    addq    $HEADER_SIZE, %rax  # Add header size
    orq     $FREE_BIT, %rax     # Set FREE bit
    movq    %rax, 0(%rcx)       # Update prev block size

    popq    %r15
    popq    %rdx
    # Block merged into prev, no need to insert
    jmp     .insert_done

.insert_skip_coalesce_end2:
    # Validation failed, restore registers and do normal insertion
    popq    %r15
    popq    %rdx
    jmp     .insert_no_coalesce_end

.insert_skip_coalesce_end:
    popq    %r15
    popq    %rdx

.insert_no_coalesce_end:
    # Append to end of list (no coalescing)
    movq    $0, 8(%rdi)         # block.next = NULL
    movq    %rcx, 16(%rdi)      # block.prev = prev

    testq   %rcx, %rcx
    jz      .insert_make_head
    movq    %rdi, 8(%rcx)       # prev.next = block
    popq    %rbp
    ret

.insert_make_head:
    movq    %rdi, gl_free_list_head(%rip)
    popq    %rbp
    ret

#==============================================================================
# gl_free - Deallocate memory and return to free list
#
# Input:  rdi = pointer to memory (returned from gl_malloc)
# Output: none
#
# Algorithm:
#   1. Validate pointer (NULL check, bounds check)
#   2. Get block header (ptr - HEADER_SIZE)
#   3. Mark as free
#   4. Try to coalesce with next physical block (forward coalescing)
#   5. Insert into sorted free list
#==============================================================================
gl_free:
    pushq   %rbp
    movq    %rsp, %rbp
    pushq   %rbx
    pushq   %r12
    pushq   %r13
    pushq   %r14

    # Check for NULL pointer
    testq   %rdi, %rdi
    jz      .free_done          # Freeing NULL is a no-op

    # Get block header address (ptr - HEADER_SIZE)
    leaq    -HEADER_SIZE(%rdi), %rbx    # rbx = block header

    # Validate pointer is within heap bounds
    movq    gl_heap_start(%rip), %rax
    cmpq    %rax, %rbx
    jl      .free_invalid       # block < heap_start

    movq    gl_heap_end(%rip), %rax
    cmpq    %rax, %rbx
    jge     .free_invalid       # block >= heap_end

    # Load block header
    movq    0(%rbx), %r12       # r12 = header (size | FREE)

    # Check if already free (double-free detection)
    testq   $FREE_BIT, %r12
    jnz     .free_double_free   # Already free - error

    # Extract size (clear FREE bit)
    movq    %r12, %r13
    andq    $-2, %r13           # r13 = block size

    # Update statistics (subtract from allocated count)
    subq    %r13, gl_allocated_bytes(%rip)

    # Try to coalesce with next physical block (forward coalescing)
    # next_block = block + HEADER_SIZE + size (use size WITHOUT FREE bit)
    movq    %r13, %r14
    andq    $-2, %r14           # Clear FREE bit to get actual size
    leaq    HEADER_SIZE(%rbx, %r14, 1), %r14    # r14 = next block address

    # Now mark current block as free (set FREE bit)
    orq     $FREE_BIT, %r13
    movq    %r13, 0(%rbx)

    # Check if next block is within heap
    movq    gl_heap_end(%rip), %rax
    cmpq    %rax, %r14
    jge     .free_no_coalesce   # Next block beyond heap end

    # Load next block header
    movq    0(%r14), %rax       # rax = next header

    # Check if next block is free
    testq   $FREE_BIT, %rax
    jz      .free_no_coalesce   # Next block not free

    # Coalesce with next block
    # Remove next block from free list first
    pushq   %rdi                # Save original pointer
    movq    %r14, %rdi
    call    gl_remove_free_block
    popq    %rdi                # Restore

    # Merge: current.size += next.size + HEADER_SIZE
    movq    0(%r14), %rax       # Load next block size
    andq    $-2, %rax           # Clear FREE bit
    addq    %rax, %r13          # Add next block size
    addq    $HEADER_SIZE, %r13  # Add header size

    # Update current block header with merged size
    orq     $FREE_BIT, %r13
    movq    %r13, 0(%rbx)

.free_no_coalesce:
    # Insert block into sorted free list
    # (gl_insert_free_sorted will handle backward coalescing automatically)
    movq    %rbx, %rdi
    call    gl_insert_free_sorted

.free_done:
    popq    %r14
    popq    %r13
    popq    %r12
    popq    %rbx
    popq    %rbp
    ret

.free_invalid:
    # Invalid pointer - for now, just return (could panic in debug mode)
    # TODO: Add debug assertion or error logging
    popq    %r14
    popq    %r13
    popq    %r12
    popq    %rbx
    popq    %rbp
    ret

.free_double_free:
    # Double-free detected - for now, just return
    # TODO: Add debug assertion or error logging
    # In production, this could be a security issue
    popq    %r14
    popq    %r13
    popq    %r12
    popq    %rbx
    popq    %rbp
    ret

#==============================================================================
# gl_remove_free_block - Remove a block from the free list
#
# Input: rdi = block to remove
# Modifies: rax, rcx
#
# This is used internally for coalescing and allocation.
#==============================================================================
gl_remove_free_block:
    pushq   %rbp
    movq    %rsp, %rbp

    # Load prev and next pointers
    movq    16(%rdi), %rax      # rax = prev_free
    movq    8(%rdi), %rcx       # rcx = next_free

    # If prev != NULL and valid: prev.next = next
    testq   %rax, %rax
    jz      .remove_no_prev

    # Validate rax is within heap bounds
    pushq   %rdx
    movq    gl_heap_start(%rip), %rdx
    cmpq    %rdx, %rax
    popq    %rdx
    jl      .remove_no_prev

    pushq   %rdx
    movq    gl_heap_end(%rip), %rdx
    cmpq    %rdx, %rax
    popq    %rdx
    jge     .remove_no_prev

    movq    %rcx, 8(%rax)

.remove_no_prev:
    # If next != NULL and valid: next.prev = prev
    testq   %rcx, %rcx
    jz      .remove_no_next

    # Validate rcx is within heap bounds
    pushq   %rdx
    movq    gl_heap_start(%rip), %rdx
    cmpq    %rdx, %rcx
    popq    %rdx
    jl      .remove_no_next

    pushq   %rdx
    movq    gl_heap_end(%rip), %rdx
    cmpq    %rdx, %rcx
    popq    %rdx
    jge     .remove_no_next

    movq    %rax, 16(%rcx)

.remove_no_next:
    # If block was head: update head
    cmpq    gl_free_list_head(%rip), %rdi
    jne     .remove_done
    movq    %rcx, gl_free_list_head(%rip)

.remove_done:
    popq    %rbp
    ret

#==============================================================================
# gl_validate_free_block - Validate pointer consistency (FD->bk == P check)
#
# Input:  rdi = block to validate
# Output: rax = 1 if valid, 0 if corrupt
#
# Validates that:
#   - If block.next != NULL: next.prev == block (forward consistency)
#   - If block.prev != NULL: prev.next == block (backward consistency)
#
# This catches free-list corruption early, following glibc's validation approach.
#==============================================================================
gl_validate_free_block:
    pushq   %rbp
    movq    %rsp, %rbp
    pushq   %rbx
    pushq   %rcx

    # Validate forward link: if next != NULL, check next.prev == block
    movq    8(%rdi), %rbx           # rbx = block.next
    testq   %rbx, %rbx
    jz      .validate_check_prev    # Skip if next is NULL

    # Bounds check for next pointer
    pushq   %rax
    movq    gl_heap_start(%rip), %rax
    cmpq    %rax, %rbx
    popq    %rax
    jl      .validate_failed        # next < heap_start = corrupt

    pushq   %rax
    movq    gl_heap_end(%rip), %rax
    cmpq    %rax, %rbx
    popq    %rax
    jge     .validate_failed        # next >= heap_end = corrupt

    # Check if next.prev == block
    movq    16(%rbx), %rcx          # rcx = next.prev
    cmpq    %rdi, %rcx
    jne     .validate_failed        # Corruption: next.prev != block

.validate_check_prev:
    # Validate backward link: if prev != NULL, check prev.next == block
    movq    16(%rdi), %rbx          # rbx = block.prev
    testq   %rbx, %rbx
    jz      .validate_success       # Skip if prev is NULL

    # Bounds check for prev pointer
    pushq   %rax
    movq    gl_heap_start(%rip), %rax
    cmpq    %rax, %rbx
    popq    %rax
    jl      .validate_failed        # prev < heap_start = corrupt

    pushq   %rax
    movq    gl_heap_end(%rip), %rax
    cmpq    %rax, %rbx
    popq    %rax
    jge     .validate_failed        # prev >= heap_end = corrupt

    # Check if prev.next == block
    movq    8(%rbx), %rcx           # rcx = prev.next
    cmpq    %rdi, %rcx
    jne     .validate_failed        # Corruption: prev.next != block

.validate_success:
    movq    $1, %rax                # Return 1 = valid
    popq    %rcx
    popq    %rbx
    popq    %rbp
    ret

.validate_failed:
    xorq    %rax, %rax              # Return 0 = corrupt
    popq    %rcx
    popq    %rbx
    popq    %rbp
    ret

#==============================================================================
# Debug/Stats Functions (optional)
#==============================================================================

.globl gl_get_allocated_bytes
gl_get_allocated_bytes:
    movq    gl_allocated_bytes(%rip), %rax
    ret

.globl gl_get_heap_start
gl_get_heap_start:
    # Auto-initialize if needed
    cmpq    $0, gl_heap_start(%rip)
    jne     .get_start_initialized
    call    gl_init_allocator
.get_start_initialized:
    movq    gl_heap_start(%rip), %rax
    ret

.globl gl_get_heap_end
gl_get_heap_end:
    # Auto-initialize if needed
    cmpq    $0, gl_heap_end(%rip)
    jne     .get_end_initialized
    call    gl_init_allocator
.get_end_initialized:
    movq    gl_heap_end(%rip), %rax
    ret
