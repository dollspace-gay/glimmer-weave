# Glimmer-Weave Programming Language

> *"The code should read like poetry, not algebra."*

**Glimmer-Weave** is a modern, expressive scripting language designed for AethelOS but built to run anywhere. It combines natural language syntax with Rust-inspired safety features, offering memory safety, pattern matching, and Result/Option types without garbage collection.

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Language Guide](#language-guide)
  - [Variables](#1-variables)
  - [Data Types](#2-data-types)
  - [Control Flow](#3-control-flow)
  - [Functions](#4-functions)
  - [Pattern Matching](#5-pattern-matching)
  - [Error Handling](#6-error-handling)
  - [Custom Types (Structs)](#7-custom-types-structs)
  - [Custom Enums](#8-custom-enums)
  - [Traits (Interfaces)](#9-traits-interfaces)
  - [Module System](#10-module-system)
  - [Iterators](#11-iterators)
  - [Pipeline Operator](#12-pipeline-operator)
  - [Type Annotations](#13-type-annotations-optional)
  - [Built-in Functions](#14-built-in-functions)
- [Examples](#examples)
- [Running Programs](#running-programs)
- [Contributing](#contributing)
- [License](#license)

---

## Features

✨ **Natural Language Syntax** - Readable keywords like `bind`, `weave`, `chant`, `yield`
🦀 **Rust-Inspired Safety** - No null pointers, explicit error handling, pattern matching
⚡ **Multiple Execution Engines** - Interpreter, bytecode VM, and native x86-64 codegen
🎯 **No Runtime Dependencies** - `no_std` compatible for embedded/OS development
🔒 **Memory Safe** - Strong typing with optional type annotations
🌊 **Functional Features** - First-class functions, closures, pipeline operator
📦 **Module System** - Organize code with natural keywords (`grove`, `offer`, `summon`)
🎭 **Traits** - Interfaces with natural language (`aspect`, `embody`, `invoke`)
🔢 **Custom Enums** - Tagged unions with exhaustive pattern matching
🔁 **Iterators** - Lazy iteration with map, filter, fold, and more
🎯 **Advanced Control Flow** - Break/continue, error propagation operator (`?`)

---

## Quick Start

### Hello, World!

```glimmer-weave
# hello.gw
"Hello, World!"
```

### Factorial Function

```glimmer-weave
# factorial.gw
chant factorial(n) then
    should n <= 1 then
        yield 1
    otherwise
        yield n * factorial(n - 1)
    end
end

factorial(5)  # Returns 120
```

### Data Pipeline

```glimmer-weave
# pipeline.gw
chant double(x) then yield x * 2 end
chant add_one(x) then yield x + 1 end

5 | double | add_one  # Returns 11
```

### Module System

```glimmer-weave
# math.gw
grove Math with
    chant sqrt(x) then
        # implementation
        yield x
    end

    offer sqrt
end

# main.gw
summon Math from "math.gw"
Math.sqrt(16)
```

---

## Installation

### Prerequisites

- **Rust** 1.70 or later ([Install Rust](https://rustup.rs/))
- **Git** (optional, for cloning the repository)

### Building from Source

#### Windows

```powershell
# Clone the repository
git clone https://github.com/yourusername/glimmer-weave.git
cd glimmer-weave

# Build the library
cargo build --release

# Build the REPL (interactive shell)
cargo build --bin glimmer-repl --features repl --release

# Run tests
cargo test

# Compiled binaries are in target\release\
# - Library: target\release\glimmer_weave.dll
# - REPL: target\release\glimmer-repl.exe
```

#### macOS / Linux

```bash
# Clone the repository
git clone https://github.com/yourusername/glimmer-weave.git
cd glimmer-weave

# Build the library
cargo build --release

# Build the REPL (interactive shell)
cargo build --bin glimmer-repl --features repl --release

# Run tests
cargo test

# Compiled binaries are in target/release/
# - Library: target/release/libglimmer_weave.so (Linux) or .dylib (macOS)
# - REPL: target/release/glimmer-repl
```

**Note:** The `--features repl` flag is required when building the REPL, as it includes optional dependencies for line editing and history.

### Running the REPL

Glimmer-Weave includes an interactive REPL (Read-Eval-Print Loop) for rapid prototyping and testing.

**Quick start:**

```bash
# Run directly (compiles and runs)
cargo run --bin glimmer-repl --features repl

# Or build once and run multiple times
cargo build --bin glimmer-repl --features repl --release

# Windows
.\target\release\glimmer-repl.exe

# macOS/Linux
./target/release/glimmer-repl
```

**REPL Features:**
- **Line editing** - Arrow keys, home/end, command history
- **Multi-line input** - Automatically detects incomplete expressions
- **Command history** - Saved between sessions
- **Special commands:**
  - `:help` - Show help message
  - `:quit` or `:exit` - Exit the REPL
  - `:clear` - Clear the screen
  - `:reset` - Reset environment (clear all variables)
- **Keyboard shortcuts:**
  - `Ctrl+C` - Cancel current input
  - `Ctrl+D` - Exit REPL

---

## Language Guide

### 1. Variables

Glimmer-Weave has two types of variable bindings:

```glimmer-weave
# Immutable binding (cannot be changed)
bind name to "Alice"
bind age to 30
bind pi to 3.14159

# Mutable variable (can be changed)
weave counter as 0
set counter to counter + 1
```

**Philosophy:** Immutable by default (like Rust), use `weave` only when mutation is needed.

---

### 2. Data Types

#### Primitives

```glimmer-weave
# Numbers (f64)
bind x to 42
bind y to 3.14

# Text (strings)
bind name to "Glimmer"
bind message to "Hello, World!"

# Truth (booleans)
bind is_valid to true
bind is_complete to false

# Nothing (null/nil)
bind empty to nothing
```

#### Collections

```glimmer-weave
# Lists
bind numbers to [1, 2, 3, 4, 5]
bind mixed to [1, "two", 3.0, true]

# Maps (key-value pairs)
bind person to {
    name: "Alice",
    age: 30,
    city: "Seattle"
}

# Access elements
bind first to numbers[0]          # 1
bind person_name to person["name"] # "Alice"
bind person_age to person.age      # 30 (dot notation)
```

---

### 3. Control Flow

#### Conditionals

```glimmer-weave
should age >= 18 then
    "Adult"
otherwise
    "Minor"
end
```

#### Loops

```glimmer-weave
# For-each loop
bind items to [1, 2, 3, 4, 5]
weave sum as 0

for each item in items then
    set sum to sum + item
end

# While loop
weave count as 0
whilst count less than 10 then
    set count to count + 1
end

# Break and continue
for each item in [1, 2, 3, 4, 5] then
    should item is 3 then
        continue  # Skip 3
    end

    should item is 5 then
        break  # Stop at 5
    end

    item
end

# Ranges
for each i in range(1, 11) then
    # Prints 1 through 10
    i
end
```

---

### 4. Functions

```glimmer-weave
# Function definition
chant greet(name) then
    yield "Hello, " + name + "!"
end

# Function call
bind message to greet("Alice")

# Function with multiple parameters
chant add(a, b) then
    yield a + b
end

# Variadic functions (accept any number of arguments)
chant sum(...numbers) then
    weave total as 0
    for each n in numbers then
        set total to total + n
    end
    yield total
end

bind result to sum(1, 2, 3, 4, 5)  # 15

# Recursive function with tail-call optimization
chant factorial(n) then
    should n <= 1 then
        yield n
    otherwise
        yield fibonacci(n - 1) + fibonacci(n - 2)
    end
end

# Functions are first-class values
bind my_func to add
bind result to my_func(5, 3)  # 8

# Closures capture their environment
chant make_adder(x) then
    yield chant(y) then
        yield x + y
    end
end

bind add_10 to make_adder(10)
add_10(5)  # 15
```

---

### 5. Pattern Matching

```glimmer-weave
# Match with literals
bind number to 42
match number with
    when 1 then "one"
    when 2 then "two"
    when 42 then "the answer"
    when _ then "something else"
end

# Match with enums (Maybe type)
chant find_first(list, predicate) then
    for each item in list then
        should predicate(item) then
            yield Present(item)
        end
    end
    yield Absent
end

bind result to find_first([1, 2, 3], chant(x) then yield x greater than 2 end)

match result with
    when Present(value) then
        "Found: " + to_text(value)
    when Absent then
        "Not found"
end

# Match with custom enums
envisage Status with
    | Pending
    | Active(Number)
    | Completed(Text)
end

bind status to Active(42)

match status with
    when Pending then "Waiting..."
    when Active(id) then "Active: " + to_text(id)
    when Completed(msg) then "Done: " + msg
end
```

**Built-in Enums:**
- `Present(value)` / `Absent` - Maybe/Option type
- `Triumph(value)` / `Mishap(error)` - Outcome/Result type

---

### 6. Error Handling

```glimmer-weave
# Try-catch style error handling
attempt then
    bind result to risky_operation()
    "Success: " + result
harmonize on "NetworkError" then
    "Network failed, retrying..."
harmonize on _ then
    "Unknown error occurred"
end

# Using Outcome type (like Rust's Result)
chant divide(a, b) then
    should b is 0 then
        yield Mishap("Division by zero")
    otherwise
        yield Triumph(a / b)
    end
end

bind result to divide(10, 2)
match result with
    when Triumph(value) then
        "Result: " + to_text(value)
    when Mishap(error) then
        "Error: " + error
end

# Error propagation operator (?)
chant safe_divide(a, b) then
    bind result to divide(a, b)?  # Propagates Mishap
    yield Triumph(result * 2)
end
```

---

### 7. Custom Types (Structs)

```glimmer-weave
# Define a struct
form Person with
    name as Text
    age as Number
    city as Text
end

# Create an instance
bind alice to Person {
    name: "Alice",
    age: 30,
    city: "Seattle"
}

# Access fields
bind alice_name to alice.name
bind alice_age to alice.age

# Nested structs
form Point with
    x as Number
    y as Number
end

form Rectangle with
    top_left as Point
    bottom_right as Point
end

bind rect to Rectangle {
    top_left: Point { x: 0, y: 10 },
    bottom_right: Point { x: 5, y: 0 }
}

bind width to rect.bottom_right.x - rect.top_left.x
```

---

### 8. Custom Enums

Define your own tagged unions with pattern matching:

```glimmer-weave
# Define a custom enum
envisage Color with
    | Red
    | Green
    | Blue
    | RGB(Number, Number, Number)
end

# Create enum values
bind primary to Red
bind custom to RGB(128, 64, 255)

# Pattern match on enums
match primary with
    when Red then "Red color"
    when Green then "Green color"
    when Blue then "Blue color"
    when RGB(r, g, b) then
        "Custom: " + to_text(r) + "," + to_text(g) + "," + to_text(b)
end

# Enums in structs
form Button with
    label as Text
    color as Color
end

bind btn to Button {
    label: "Click me",
    color: Blue
}
```

---

### 9. Traits (Interfaces)

Define interfaces that types can implement:

```glimmer-weave
# Define a trait
aspect Drawable with
    chant draw() as Text
end

# Implement trait for a type
embody Drawable for Circle then
    chant draw() as Text then
        yield "Drawing a circle"
    end
end

# Use trait methods
bind shape to Circle { radius: 5 }
invoke Drawable.draw on shape  # "Drawing a circle"

# Traits with multiple methods
aspect Shape with
    chant area() as Number
    chant perimeter() as Number
end

form Rectangle with
    width as Number
    height as Number
end

embody Shape for Rectangle then
    chant area() as Number then
        yield self.width * self.height
    end

    chant perimeter() as Number then
        yield 2 * (self.width + self.height)
    end
end
```

---

### 10. Module System

Organize code into reusable modules:

```glimmer-weave
# math.gw - Module file
grove Math with
    # Private helper (not exported)
    chant _internal_helper() then
        yield 42
    end

    # Public function
    chant sqrt(x) then
        # Newton's method implementation
        yield x / 2  # Simplified
    end

    chant pow(base, exp) then
        yield base * exp  # Simplified
    end

    # Export public functions
    offer sqrt, pow
end

# main.gw - Using the module

# Import entire module
summon Math from "math.gw"
bind result to Math.sqrt(16)

# Selective import (gather specific items)
gather sqrt, pow from Math
bind root to sqrt(25)      # No prefix needed
bind power to pow(2, 10)

# Import with alias
summon Math from "std/math.gw" as M
bind value to M.sqrt(100)

# Gather with alias
gather sqrt as square_root from Math
bind r to square_root(16)
```

**Module Keywords:**
- `grove` - Define a module
- `offer` - Export symbols from a module
- `summon` - Import entire module
- `gather` - Selectively import specific symbols
- `as` - Alias for imports

---

### 11. Iterators

Lazy iteration with transformation pipelines:

```glimmer-weave
# Create an iterator from a list
bind numbers to [1, 2, 3, 4, 5]
bind it to iter(numbers)

# Transform with map
bind doubled to it | iter_map(chant(x) then yield x * 2 end)

# Filter values
bind evens to doubled | iter_filter(chant(x) then yield x % 2 is 0 end)

# Collect back to list
bind result to evens | iter_collect  # [4, 8]

# Full pipeline
bind result to iter([1, 2, 3, 4, 5])
    | iter_map(chant(x) then yield x * 2 end)
    | iter_filter(chant(x) then yield x greater than 5 end)
    | iter_collect  # [6, 8, 10]

# Iterator operations
iter_fold(iter([1, 2, 3, 4, 5]), 0, chant(acc, x) then
    yield acc + x
end)  # 15

iter_take(iter([1, 2, 3, 4, 5]), 3)  # [1, 2, 3]
iter_skip(iter([1, 2, 3, 4, 5]), 2)  # [3, 4, 5]

# Chaining operations
bind sum to iter(range(1, 11))
    | iter_filter(chant(x) then yield x % 2 is 0 end)
    | iter_map(chant(x) then yield x * x end)
    | iter_fold(0, chant(acc, x) then yield acc + x end)
# Sum of squares of even numbers: 2² + 4² + 6² + 8² + 10² = 220

# Find first match
iter_find(iter([1, 2, 3, 4, 5]), chant(x) then
    yield x greater than 3
end)  # Present(4)

# Check conditions
iter_any(iter([1, 2, 3]), chant(x) then yield x > 2 end)  # true
iter_all(iter([1, 2, 3]), chant(x) then yield x > 0 end)  # true
```

**Iterator Functions:**
- `iter(list)` - Create iterator from list
- `iter_next(it)` - Get next value (Present or Absent)
- `iter_map(it, fn)` - Transform each element
- `iter_filter(it, predicate)` - Keep only matching elements
- `iter_fold(it, init, fn)` - Reduce to single value
- `iter_collect(it)` - Collect to list
- `iter_take(it, n)` - Take first n elements
- `iter_skip(it, n)` - Skip first n elements
- `iter_find(it, predicate)` - Find first match
- `iter_any(it, predicate)` - Check if any match
- `iter_all(it, predicate)` - Check if all match
- `iter_zip(it1, it2)` - Zip two iterators

---

### 12. Pipeline Operator

The pipeline operator (`|`) enables functional composition by threading values through functions:

```glimmer-weave
# Basic pipeline
5 | double | add_one | square  # ((5 * 2) + 1)² = 121

# Pipeline with additional arguments
10 | add(5) | multiply(2)  # (10 + 5) * 2 = 30

# Data transformation pipeline
bind data to [-2, -1, 0, 1, 2, 3, 4, 5]

data
  | filter_positive    # [1, 2, 3, 4, 5]
  | double_all         # [2, 4, 6, 8, 10]
  | sum                # 30

# With iterators
[1, 2, 3, 4, 5]
  | iter
  | iter_map(chant(x) then yield x * 2 end)
  | iter_filter(chant(x) then yield x > 5 end)
  | iter_collect  # [6, 8, 10]
```

---

### 13. Type Annotations (Optional)

```glimmer-weave
# Variables with type annotations
bind name as Text to "Alice"
weave count as Number as 0

# Functions with type annotations
chant add(a as Number, b as Number) as Number then
    yield a + b
end

# Structs with typed fields
form Person with
    name as Text
    age as Number
    is_active as Truth
end

# Generic type parameters (for traits)
aspect Container<T> with
    chant get() as T
    chant set(value as T)
end
```

---

### 14. Built-in Functions

Glimmer-Weave provides extensive built-in functions for common operations:

#### List Operations

```glimmer-weave
list_length([1, 2, 3])           # 3
list_push([1, 2], 3)             # [1, 2, 3]
list_pop([1, 2, 3])              # [1, 2]
list_first([1, 2, 3])            # 1
list_last([1, 2, 3])             # 3
list_slice([1, 2, 3, 4, 5], 1, 4) # [2, 3, 4]
list_concat([1, 2], [3, 4])      # [1, 2, 3, 4]
list_reverse([1, 2, 3])          # [3, 2, 1]
list_contains([1, 2, 3], 2)      # true
list_sum([1, 2, 3, 4, 5])        # 15.0
```

#### String Operations

```glimmer-weave
to_text(42)                      # "42"
text_length("hello")             # 5
text_concat("Hello", " World")   # "Hello World"
text_slice("hello", 1, 4)        # "ell"
text_uppercase("hello")          # "HELLO"
text_lowercase("HELLO")          # "hello"
text_contains("hello", "ell")    # true
```

#### Math Operations

```glimmer-weave
math_floor(3.7)                  # 3.0
math_ceil(3.2)                   # 4.0
math_round(3.5)                  # 4.0
math_abs(-5.3)                   # 5.3
```

#### Iteration

```glimmer-weave
range(1, 6)                      # [1, 2, 3, 4, 5]
iter([1, 2, 3])                  # Create iterator
iter_next(iterator)              # Get next value
iter_map(it, fn)                 # Transform elements
iter_filter(it, predicate)       # Filter elements
iter_fold(it, init, fn)          # Reduce to value
iter_collect(it)                 # Collect to list
```

---

## Examples

### Example 1: FizzBuzz

```glimmer-weave
chant fizzbuzz(n) then
    should n % 15 is 0 then
        yield "FizzBuzz"
    otherwise
        should n % 3 is 0 then
            yield "Fizz"
        otherwise
            should n % 5 is 0 then
                yield "Buzz"
            otherwise
                yield to_text(n)
            end
        end
    end
end

for each i in range(1, 101) then
    fizzbuzz(i)
end
```

### Example 2: List Processing with Iterators

```glimmer-weave
# Filter and transform using iterators
bind numbers to [-2, -1, 0, 1, 2, 3, 4, 5]

bind result to iter(numbers)
    | iter_filter(chant(x) then yield x greater than 0 end)
    | iter_map(chant(x) then yield x * 2 end)
    | iter_collect  # [2, 4, 6, 8, 10]
```

### Example 3: Working with Structs and Enums

```glimmer-weave
# Define enum for account status
envisage AccountStatus with
    | Active
    | Frozen(Text)
    | Closed
end

# Define struct
form Account with
    owner as Text
    balance as Number
    status as AccountStatus
end

chant withdraw(account, amount) then
    match account.status with
        when Active then
            should amount greater than account.balance then
                yield Mishap("Insufficient funds")
            otherwise
                yield Triumph(Account {
                    owner: account.owner,
                    balance: account.balance - amount,
                    status: Active
                })
            end
        when Frozen(reason) then
            yield Mishap("Account frozen: " + reason)
        when Closed then
            yield Mishap("Account is closed")
    end
end

bind my_account to Account {
    owner: "Alice",
    balance: 100,
    status: Active
}

bind withdrawal to withdraw(my_account, 30)
match withdrawal with
    when Triumph(new_account) then
        "New balance: " + to_text(new_account.balance)
    when Mishap(error) then
        "Error: " + error
end
```

### Example 4: Module System

```glimmer-weave
# geometry.gw
grove Geometry with
    form Point with
        x as Number
        y as Number
    end

    chant distance(p1, p2) then
        bind dx to p2.x - p1.x
        bind dy to p2.y - p1.y
        yield math_sqrt(dx * dx + dy * dy)
    end

    offer Point, distance
end

# main.gw
summon Geometry from "geometry.gw"

bind p1 to Geometry.Point { x: 0, y: 0 }
bind p2 to Geometry.Point { x: 3, y: 4 }

Geometry.distance(p1, p2)  # 5.0
```

### Example 5: Traits

```glimmer-weave
# Define a trait
aspect Printable with
    chant to_string() as Text
end

# Define a struct
form User with
    name as Text
    age as Number
end

# Implement trait for struct
embody Printable for User then
    chant to_string() as Text then
        yield self.name + " (age " + to_text(self.age) + ")"
    end
end

# Use trait
bind user to User { name: "Bob", age: 25 }
invoke Printable.to_string on user  # "Bob (age 25)"
```

---

## Running Programs

### Using the REPL

The fastest way to try Glimmer-Weave:

```bash
cargo run --bin glimmer-repl --features repl
```

### Using the Library

Glimmer-Weave is primarily a library. To run programs, use the interpreter:

```rust
use glimmer_weave::{Lexer, Parser, Evaluator};

fn main() {
    let source = r#"
        chant factorial(n) then
            should n <= 1 then
                yield 1
            otherwise
                yield n * factorial(n - 1)
            end
        end

        factorial(5)
    "#;

    // Tokenize
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    // Parse
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("Parse error");

    // Evaluate
    let mut evaluator = Evaluator::new();
    let result = evaluator.eval(&ast).expect("Runtime error");

    println!("Result: {:?}", result);
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test test_iterator_integration
cargo test --lib semantic::

# Run with output
cargo test -- --nocapture
```

---

## Language Philosophy

Glimmer-Weave follows these core principles:

### 1. Readability Over Brevity

Choose descriptive keywords over terse symbols:

| Concept | Glimmer-Weave | Traditional |
|---------|---------------|-------------|
| Immutable | `bind x to 5` | `let x = 5` |
| Mutable | `weave x as 5` | `let mut x = 5` |
| Assignment | `set x to 10` | `x = 10` |
| Function | `chant add(a, b)` | `fn add(a, b)` |
| Return | `yield result` | `return result` |
| If/Else | `should...then...otherwise` | `if...else` |
| While | `whilst condition then` | `while condition` |
| Module | `grove Math with...end` | `mod math { }` |
| Trait | `aspect Drawable with...end` | `trait Drawable { }` |
| Enum | `envisage Color with...end` | `enum Color { }` |

### 2. Safety Without Garbage Collection

- **No null pointers** - Use `Present(value)` / `Absent` instead
- **Explicit error handling** - Use `Triumph(value)` / `Mishap(error)`
- **Pattern matching** - Exhaustive matches catch errors at compile time
- **Strong typing** - Optional type annotations for documentation

### 3. Multiple Execution Strategies

- **Tree-walking interpreter** - Full features, best for development
- **Bytecode VM** - 5-10x faster, production-ready
- **Native x86-64 codegen** - Fastest, for performance-critical code

---

## Project Structure

```
glimmer-weave/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── token.rs            # Token definitions
│   ├── lexer.rs            # Tokenizer
│   ├── ast.rs              # Abstract syntax tree
│   ├── parser.rs           # Parser
│   ├── eval.rs             # Tree-walking interpreter
│   ├── semantic.rs         # Semantic analysis
│   ├── type_inference.rs   # Type inference engine
│   ├── bytecode.rs         # Bytecode instruction set
│   ├── bytecode_compiler.rs # Bytecode compiler
│   ├── vm.rs               # Bytecode virtual machine
│   ├── codegen.rs          # Native x86-64 code generator
│   ├── runtime.rs          # Runtime functions
│   ├── native_runtime.rs   # Native runtime helpers
│   ├── module_resolver.rs  # Module system resolver
│   ├── monomorphize.rs     # Generic type monomorphization
│   └── bin/repl.rs         # Interactive REPL
├── docs/                   # Design documentation
│   ├── module_system_design.md
│   └── iterator_design.md
├── examples/               # Example programs
├── tests/                  # Integration tests
├── Cargo.toml             # Rust package manifest
├── CLAUDE.md              # AI assistant development guide
└── README.md              # This file
```

---

## Contributing

Contributions are welcome! Please follow these guidelines:

1. **Use the `bd` issue tracker** for all work tracking
2. **Maintain all three execution engines** when adding features
3. **Write comprehensive tests** for new features
4. **Follow the natural language philosophy** for naming
5. **Document limitations clearly** if a feature can't be implemented

### Development Workflow

```bash
# Create a new issue
bd create "Add closure support"

# Mark issue as in progress
bd update glimmer-weave-xxx --status in_progress

# Make changes and test
cargo test

# Close issue when done
bd close glimmer-weave-xxx --reason "Implemented closure support with tests"
```

For detailed development guidelines, see [CLAUDE.md](CLAUDE.md).

---

## Roadmap

### Current Features ✅

- ✅ Variables (immutable/mutable)
- ✅ Control flow (if/else, loops, break/continue)
- ✅ Functions with tail-call optimization
- ✅ Variadic functions
- ✅ Closures and first-class functions
- ✅ Pattern matching (exhaustive)
- ✅ Custom types (structs)
- ✅ Custom enums (tagged unions)
- ✅ Traits (interfaces)
- ✅ **Module system** (grove, offer, summon, gather) - NEW!
- ✅ **Iterators** (map, filter, fold, etc.) - NEW!
- ✅ Pipeline operator
- ✅ Error handling (attempt/harmonize)
- ✅ **Error propagation operator** (?) - NEW!
- ✅ Built-in collections (lists, maps)
- ✅ Expanded standard library (30+ built-in functions)
- ✅ **REPL** (interactive shell) - NEW!
- ✅ Three execution engines (interpreter, bytecode VM, native codegen)

### Planned Features 🚧

- [ ] Generics/parametric polymorphism
- [ ] Generic trait implementations
- [ ] Async/await
- [ ] Package manager
- [ ] LSP (Language Server Protocol) for IDE support
- [ ] Debugger
- [ ] Standard library expansion (filesystem, network, etc.)

---

## Keyword Reference

| Keyword | Purpose | Example |
|---------|---------|---------|
| `bind` | Immutable variable | `bind x to 42` |
| `weave` | Mutable variable | `weave counter as 0` |
| `set` | Assignment | `set counter to 10` |
| `chant` | Define function | `chant add(a, b) then...end` |
| `yield` | Return from function | `yield result` |
| `...` | Variadic parameter | `chant sum(...numbers) then...end` |
| `should` | If statement | `should x > 0 then...end` |
| `otherwise` | Else clause | `otherwise...end` |
| `for each` | For-each loop | `for each item in list then...end` |
| `whilst` | While loop | `whilst condition then...end` |
| `break` | Exit loop | `break` |
| `continue` | Skip to next iteration | `continue` |
| `match` | Pattern matching | `match x with when 1 then...end` |
| `when` | Match arm | `when 42 then "found"` |
| `form` | Define struct | `form Point with x as Number end` |
| `envisage` | Define enum | `envisage Color with \| Red \| Green end` |
| `aspect` | Define trait | `aspect Drawable with...end` |
| `embody` | Implement trait | `embody Drawable for Circle then...end` |
| `invoke` | Call trait method | `invoke Trait.method on value` |
| `grove` | Define module | `grove Math with...end` |
| `offer` | Export from module | `offer sqrt, pow` |
| `summon` | Import module | `summon Math from "math.gw"` |
| `gather` | Selective import | `gather sqrt, pow from Math` |
| `as` | Alias | `summon Math as M` |
| `attempt` | Try block | `attempt then...harmonize...end` |
| `harmonize` | Catch block | `harmonize on "Error" then...end` |
| `?` | Error propagation | `bind x to risky()?` |
| `Present` | Some/Just value | `Present(42)` |
| `Absent` | None/Nothing | `Absent` |
| `Triumph` | Ok/Success | `Triumph(result)` |
| `Mishap` | Err/Failure | `Mishap("error")` |
| `\|` | Pipeline operator | `x \| f \| g` |

---

## Resources

- **Documentation**: See [CLAUDE.md](CLAUDE.md) for development guide
- **Examples**: Browse [examples/](examples/) directory
- **Tests**: See [tests/](tests/) for comprehensive test suite
- **Design Docs**: See [docs/](docs/) for design documents
- **Issue Tracker**: Use `bd` command-line tool

---

## License

[Specify your license here - e.g., MIT, Apache 2.0]

---

## Acknowledgments

Inspired by:
- **Rust** - Memory safety, ownership, pattern matching, traits
- **Elixir** - Pipeline operator, functional patterns
- **Swift** - Natural language syntax, readability
- **Haskell** - Lazy evaluation, iterators

Designed for **AethelOS** but built to run anywhere.

---

**Questions? Feedback?** Please open an issue using the `bd` issue tracker.

Happy coding! ✨
