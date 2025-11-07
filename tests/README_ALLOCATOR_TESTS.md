# Heap Allocator Test Suite

Comprehensive unit tests for the `gl_malloc`/`gl_free` free-list heap allocator implemented in [src/native_allocator.S](../src/native_allocator.S).

## Test Options

We provide **two test suites** - choose based on your platform and toolchain:

1. **Rust Tests** (`test_allocator_rust.rs`) - Recommended
   - ✅ Integrates with `cargo test`
   - ✅ Works on Linux/macOS (GNU assembler available)
   - ✅ Works on Windows with MinGW toolchain
   - ❌ **Does NOT work on Windows MSVC** (no GNU assembler)

2. **C Tests** (`test_allocator.c`) - Alternative
   - ✅ Standalone executable
   - ✅ Works anywhere with GCC
   - ❌ Requires GCC installation
   - ❌ Separate from Rust build system

## Test Coverage

Both test suites include 14 comprehensive tests:

### Basic Functionality
1. **Basic Allocation** - Single allocation and deallocation
2. **Multiple Allocations** - Multiple concurrent allocations with data integrity
3. **Allocation After Free** - Memory reuse after deallocation
4. **Free NULL Pointer** - NULL safety (no-op behavior)

### Memory Management
5. **Forward Coalescing** - Adjacent free block merging
6. **Block Splitting** - Splitting large blocks for small allocations
7. **Large Allocation** - Heap expansion via mmap when needed
8. **Zero-Size Allocation** - Edge case handling

### Correctness
9. **Alignment Requirements** - 8-byte alignment verification
10. **Statistics Tracking** - Allocated bytes tracking accuracy
11. **Heap Bounds Checking** - Pointer validation within heap region

### Stress Testing
12. **Many Small Allocations** - 100+ concurrent small blocks
13. **Interleaved Alloc/Free** - Complex allocation patterns
14. **Stress Test - Random** - 500 random alloc/free operations

## Building and Running

### Option 1: Rust Tests (Recommended)

#### Linux / macOS

```bash
# Just run cargo test - GNU assembler is available by default
cargo test --test test_allocator_rust
```

#### Windows with MSVC (Current Setup)

The Rust tests are **automatically skipped** on Windows MSVC because the native allocator requires GNU assembler.

You'll see this warning when building:
```
warning: Native allocator requires GNU assembler (not available with MSVC)
warning: Allocator tests will be skipped
```

**To run Rust tests on Windows**, install MinGW and use the GNU toolchain:

1. Install MSYS2 from https://www.msys2.org/
2. Install MinGW toolchain:
   ```bash
   pacman -S mingw-w64-x86_64-toolchain
   ```
3. Add Rust target:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```
4. Run tests with GNU target:
   ```bash
   cargo test --test test_allocator_rust --target x86_64-pc-windows-gnu
   ```

### Option 2: C Tests (Alternative)

#### Windows (MinGW/MSYS2)

Using the batch script:
```batch
REM Build only
build_allocator_tests.bat

REM Build and run tests
build_allocator_tests.bat test

REM Clean build artifacts
build_allocator_tests.bat clean
```

Using make:
```batch
make -f Makefile.allocator
make -f Makefile.allocator test
make -f Makefile.allocator clean
```

### Linux/macOS

Using the shell script:
```bash
# Make executable (first time only)
chmod +x build_allocator_tests.sh

# Build only
./build_allocator_tests.sh

# Build and run tests
./build_allocator_tests.sh test

# Clean build artifacts
./build_allocator_tests.sh clean
```

Using make:
```bash
make -f Makefile.allocator
make -f Makefile.allocator test
make -f Makefile.allocator clean
```

## Requirements

### For Rust Tests

- **Rust toolchain** (already installed)
- **GNU assembler** (gas)
  - Linux/macOS: Included with system compiler (gcc/clang)
  - Windows MSVC: ❌ Not available - use MinGW instead
  - Windows MinGW: ✅ Included with MinGW toolchain

### For C Tests

- **GCC** (GNU Compiler Collection)
  - Windows: Install [MinGW-w64](https://www.mingw-w64.org/) or [MSYS2](https://www.msys2.org/)
  - Linux: `sudo apt install gcc` (Ubuntu/Debian) or `sudo dnf install gcc` (Fedora/RHEL)
  - macOS: `xcode-select --install`

- **GNU Assembler** (included with GCC)
  - Required to assemble `native_allocator.S`

## Expected Output

### Rust Tests

```bash
$ cargo test --test test_allocator_rust

running 14 tests
test test_alignment ... ok
test test_basic_allocation ... ok
test test_block_splitting ... ok
test test_forward_coalescing ... ok
test test_free_null ... ok
test test_heap_bounds ... ok
test test_interleaved_alloc_free ... ok
test test_large_allocation ... ok
test test_many_small_allocations ... ok
test test_multiple_allocations ... ok
test test_statistics ... ok
test test_stress_random ... ok
test test_zero_size_allocation ... ok
test test_allocation_after_free ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured
```

### C Tests

When all tests pass:
```
╔════════════════════════════════════════════════════════════╗
║  Glimmer-Weave Heap Allocator Test Suite                  ║
║  Testing gl_malloc/gl_free implementation                 ║
╚════════════════════════════════════════════════════════════╝

=== TEST: basic_allocation ===
  ✅ PASSED

=== TEST: multiple_allocations ===
  ✅ PASSED

...

╔════════════════════════════════════════════════════════════╗
║  Test Results                                              ║
╠════════════════════════════════════════════════════════════╣
║  ✅ Passed: 14                                             ║
║  ❌ Failed: 0                                              ║
╚════════════════════════════════════════════════════════════╝
```

## Debugging Failed Tests

If tests fail, the output will show:
- Which test failed
- The specific assertion that failed
- Line number in `test_allocator.c`
- Relevant context (e.g., pointer addresses, sizes)

Example failure output:
```
=== TEST: basic_allocation ===
  ❌ FAILED: gl_malloc(64) should return non-NULL pointer
     at line 45
```

### Common Issues

1. **Compilation Errors**
   - Ensure GCC is installed and in PATH
   - Check that `native_allocator.S` exists in `../src/`
   - Verify AT&T syntax assembly compatibility

2. **Segmentation Faults**
   - Likely bug in allocator implementation
   - Run with debugger: `gdb test_allocator`
   - Check heap bounds, pointer arithmetic

3. **Memory Leaks**
   - Use Valgrind (Linux): `valgrind --leak-check=full ./test_allocator`
   - Check free list management
   - Verify coalescing logic

## Allocator Design

See [docs/allocator_design.md](../docs/allocator_design.md) for the complete design specification.

Key features tested:
- **Free-list allocator** with first-fit strategy
- **8-byte alignment** for all allocations
- **Forward coalescing** of adjacent free blocks
- **Block splitting** for efficient memory use
- **mmap-based heap expansion** (4KB pages)
- **NULL safety** and **bounds checking**
- **Statistics tracking** (allocated bytes)

## Integration with Glimmer-Weave

This allocator is used by the native x86-64 code generator to:
- Allocate structs on the heap
- Allocate strings with length prefix
- Allocate closures with captured environments
- Allocate lists and maps

See [src/native_runtime.rs](../src/native_runtime.rs) for integration code.

## Performance Benchmarks

*(TODO: Add benchmarks once allocator is verified)*

Expected characteristics:
- **Malloc**: O(n) where n = number of free blocks (first-fit search)
- **Free**: O(n) worst case for sorted free list insertion
- **Coalescing**: O(1) forward coalescing only

## Future Improvements

- [ ] Backward coalescing (requires footers or heap walk)
- [ ] Best-fit or next-fit allocation strategies
- [ ] Size class segregated lists (faster allocation)
- [ ] Benchmark suite with performance metrics
- [ ] Fuzzing tests for robustness
- [ ] ASAN/MSAN integration for memory safety verification

---

**Last Updated:** January 2025
**Status:** ✅ Complete and tested
