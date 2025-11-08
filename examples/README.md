# Glimmer-Weave Example Programs

This directory contains example programs written in Glimmer-Weave, the scripting language for AethelOS.

## Examples

| File | Description | Expected Output |
|------|-------------|-----------------|
| `01_hello_world.gw` | Basic string binding | "Hello, World!" |
| `02_arithmetic.gw` | Arithmetic operations (+, -, *, /, %) | 42 |
| `03_variables.gw` | Immutable (bind) and mutable (weave/set) variables | 15 |
| `04_conditionals.gw` | If/then/otherwise control flow | "Large number" |
| `05_loops.gw` | For-each iteration with ranges | 10 |
| `06_functions.gw` | Function definitions and calls | 30 |
| `07_lists.gw` | List literals and indexing | 20 |
| `08_maps.gw` | Map literals and field access | "Elara" |
| `09_factorial.gw` | Recursive factorial function | 120 |
| `10_fibonacci.gw` | Recursive Fibonacci function | 55 |
| `11_string_functions.gw` | String manipulation | Various |
| `12_math_functions.gw` | Mathematical operations | Various |
| `13_list_functions.gw` | List operations | Various |
| `14_map_functions.gw` | Map operations | Various |
| `15_type_conversion.gw` | Type conversions | Various |
| `16_comprehensive_test.gw` | Comprehensive language test | Various |
| `17_structs.gw` | Custom data types with named fields | 50 |
| **Ownership & Borrowing** | | |
| `20_ownership_basics.gw` | Basic ownership and move semantics | Move errors demonstrated |
| `21_shared_borrowing.gw` | Read-only borrowing with multiple borrows | Sum and max computations |
| `22_mutable_borrowing.gw` | Exclusive mutable borrowing | In-place modifications |
| `23_lifetimes.gw` | Explicit lifetime annotations | References with lifetimes |
| `24_ownership_patterns.gw` | Common ownership patterns (builder, transform, etc.) | Chained transformations |
| `25_ownership_errors.gw` | Common errors and how to fix them | Error demonstrations |
| `26_structs_ownership.gw` | Ownership with custom data types | Person struct with borrows |
| `27_collections_ownership.gw` | Collections and ownership patterns | List and map operations |
| `28_advanced_ownership.gw` | Advanced patterns (iterators, builders, etc.) | Iterator and builder patterns |
| `29_migration_guide.gw` | Migrating code to use ownership system | Performance comparisons |

## Running Examples

Glimmer-Weave has **three execution engines**, each with different trade-offs:

### 1. Tree-Walking Interpreter (Recommended for Development)

```rust
use glimmer_weave::{Lexer, Parser, Evaluator};

// Read source file
let source = std::fs::read_to_string("examples/01_hello_world.gw")?;

// Tokenize
let mut lexer = Lexer::new(&source);
let tokens = lexer.tokenize();

// Parse
let mut parser = Parser::new(tokens);
let ast = parser.parse()?;

// Evaluate directly from AST
let mut evaluator = Evaluator::new();
let result = evaluator.eval(&ast)?;

println!("{:?}", result);
```

**Pros:** Full feature support, easy debugging
**Cons:** Slower execution, higher memory usage

### 2. Bytecode VM (Recommended for Production)

```rust
use glimmer_weave::{Lexer, Parser, BytecodeCompiler, VirtualMachine};

let source = std::fs::read_to_string("examples/01_hello_world.gw")?;

// Tokenize and parse (same as interpreter)
let mut lexer = Lexer::new(&source);
let tokens = lexer.tokenize();
let mut parser = Parser::new(tokens);
let ast = parser.parse()?;

// Compile to bytecode
let mut compiler = BytecodeCompiler::new();
let chunk = compiler.compile(&ast)?;

// Execute in VM
let mut vm = VirtualMachine::new();
vm.load_chunk(chunk);
let result = vm.run()?;

println!("{:?}", result);
```

**Pros:** Fast execution, low memory usage, full feature support
**Cons:** Less debuggable than interpreter

### 3. Native x86-64 Code Generator (Experimental)

```rust
use glimmer_weave::{Lexer, Parser, NativeCodegen};

let source = std::fs::read_to_string("examples/01_hello_world.gw")?;

let mut lexer = Lexer::new(&source);
let tokens = lexer.tokenize();
let mut parser = Parser::new(tokens);
let ast = parser.parse()?;

// Generate native assembly
let mut codegen = NativeCodegen::new();
codegen.compile(&ast)?;
let assembly = codegen.to_assembly();

// Write to .s file for assembling
std::fs::write("output.s", assembly)?;
```

**Pros:** Fastest execution (compiled to native code)
**Cons:** Limited feature support (no heap allocation yet), requires assembler/linker

**Feature Comparison:**

| Feature | Interpreter | Bytecode VM | Native Codegen |
|---------|-------------|-------------|----------------|
| Variables | ✅ | ✅ | ✅ |
| Control flow | ✅ | ✅ | ✅ |
| Functions | ✅ | ✅ | ✅ |
| Recursion | ✅ | ✅ | ✅ |
| Tail call optimization | ❌ | ✅ | ✅ |
| Lists | ✅ | ✅ | ⚠️ Limited |
| Maps | ✅ | ✅ | ⚠️ Limited |
| Structs | ✅ | ✅ | ❌ (requires heap) |
| Pattern matching | ✅ | ✅ | ⚠️ Basic only |
| Error handling | ✅ | ✅ | ⚠️ Limited |
| Outcome/Maybe types | ✅ | ✅ | ✅ |

## Language Features Demonstrated

### Variables
- **Immutable bindings**: `bind x to 42`
- **Mutable variables**: `weave counter as 0`
- **Mutation**: `set counter to 10`

### Control Flow
- **Conditionals**: `should x > 5 then ... otherwise ... end`
- **Loops**: `for each item in list then ... end`
- **Ranges**: `range(1, 10)`

### Functions
- **Definition**: `chant add(a, b) then ... end`
- **Return**: `yield result`
- **Recursion**: Fully supported
- **Closures**: Functions capture their environment

### Data Structures
- **Lists**: `[1, 2, 3]`
- **Maps**: `{name: "Elara", age: 42}`
- **Structs**: `form Person with name as Text age as Number end`
- **Indexing**: `list[0]`, `map[key]`
- **Field access**: `map.field`, `struct.field`

### Operators
- **Arithmetic**: `+`, `-`, `*`, `/`, `%`
- **Comparison**: `>`, `<`, `>=`, `<=`, `is`, `is not`
- **Logical**: `and`, `or`, `not`

## Implementation Status

✅ **Complete**:
- Lexer (tokenization with position tracking)
- Parser (AST generation with source spans)
- Evaluator (basic execution)
- Variables (bind, weave, set)
- Control flow (if, for, whilst)
- Functions (chant, yield)
- Data structures (lists, maps, structs)
- Arithmetic and logic
- Pattern matching (`match ... when`)
- Error handling (`attempt ... harmonize`)
- Outcome/Maybe types (`Triumph`, `Mishap`, `Present`, `Absent`)
- Custom structs (`form ... with ... end`)
- **Ownership & Borrowing System** (`borrow`, `borrow mut`, lifetimes)
- Borrow checking (compile-time safety)
- Lifetime inference and validation
- Precise error messages with source locations

⚪ **Not Yet Implemented**:
- Pipelines (`|`)
- World-Tree queries (`seek where`)
- Capability requests (`request ... with justification`)
- Generics / parametric polymorphism
- Traits / interfaces

⚠️ **Limited Support**:
- **Native codegen for structs** - Requires heap allocation runtime (not yet implemented)
  - Structs work fully in interpreter and bytecode VM
  - Native codegen emits clear error messages directing users to use interpreter/VM instead

## Philosophy

Glimmer-Weave emphasizes natural expression over terse syntax:

```glimmer-weave
# Natural language keywords
should age >= 18 then
    VGA.write("Welcome, " + name)
otherwise
    VGA.write("Access denied")
end
```

The language is designed to feel like expressing intent rather than commanding a machine.
