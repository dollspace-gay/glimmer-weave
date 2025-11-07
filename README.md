# Glimmer-Weave Programming Language

> *"The code should read like poetry, not algebra."*

**Glimmer-Weave** is a modern, expressive scripting language designed for AethelOS but built to run anywhere. It combines natural language syntax with Rust-inspired safety features, offering memory safety, pattern matching, and Result/Option types without garbage collection.

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Language Guide](#language-guide)
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

# Build the project
cargo build --release

# Run tests
cargo test

# The compiled library will be in target/release/
```

#### macOS

```bash
# Clone the repository
git clone https://github.com/yourusername/glimmer-weave.git
cd glimmer-weave

# Build the project
cargo build --release

# Run tests
cargo test

# The compiled library will be in target/release/
```

#### Linux

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone the repository
git clone https://github.com/yourusername/glimmer-weave.git
cd glimmer-weave

# Build the project
cargo build --release

# Run tests
cargo test

# The compiled library will be in target/release/
```

### Development Build

For faster compilation during development:

```bash
cargo build
cargo test
```

### Running the REPL

Glimmer-Weave includes an interactive REPL (Read-Eval-Print Loop) for rapid prototyping and testing:

```bash
# Run the REPL
cargo run --bin glimmer-repl --features repl

# Or build and run separately
cargo build --bin glimmer-repl --features repl --release
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

**Example REPL Session:**

```
glimmer[1]> bind x to 42
42
glimmer[2]> x + 10
52
glimmer[3]> chant factorial(n) then
       ...>     should n <= 1 then
       ...>         yield 1
       ...>     otherwise
       ...>         yield n * factorial(n - 1)
       ...>     end
       ...> end
<function>
glimmer[4]> factorial(5)
120
glimmer[5]> :quit
Goodbye!
```

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

# Recursive function
chant fibonacci(n) then
    should n <= 1 then
        yield n
    otherwise
        yield fibonacci(n - 1) + fibonacci(n - 2)
    end
end

# Functions are first-class values
bind my_func to add
bind result to my_func(5, 3)  # 8
```

**Keywords:**
- `chant` - Define a function
- `yield` - Return a value

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

### 8. Pipeline Operator

The pipeline operator (`|`) enables functional composition by threading values through functions:

```glimmer-weave
# Basic pipeline
5 | double | add_one | square  # ((5 * 2) + 1)^2 = 49

# Pipeline with additional arguments
10 | add(5) | multiply(2)  # (10 + 5) * 2 = 30

# Data transformation pipeline
bind data to [-2, -1, 0, 1, 2, 3, 4, 5]

data
  | filter_positive    # [1, 2, 3, 4, 5]
  | double_all         # [2, 4, 6, 8, 10]
  | sum                # 30

# With native functions
[1, 2, 3, 4, 5] | list_length | double  # 10
```

**How it works:**
- `x | f` is equivalent to `f(x)`
- `x | f(y)` is equivalent to `f(x, y)`
- Pipelines evaluate left-to-right

---

### 9. Type Annotations (Optional)

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
```

---

### 10. Built-in Functions

#### List Operations

```glimmer-weave
list_length([1, 2, 3])           # 3
list_push([1, 2], 3)             # [1, 2, 3]
list_pop([1, 2, 3])              # [1, 2]
list_first([1, 2, 3])            # 1
list_last([1, 2, 3])             # 3
```

#### String Operations

```glimmer-weave
to_text(42)                      # "42"
text_length("hello")             # 5
```

#### Iteration

```glimmer-weave
range(1, 6)                      # [1, 2, 3, 4, 5]
iter([1, 2, 3])                  # Create iterator
iter_next(iterator)              # Get next value
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

### Example 2: List Processing

```glimmer-weave
# Filter and transform a list
chant filter_positive(lst) then
    weave result as []
    for each item in lst then
        should item greater than 0 then
            set result to list_push(result, item)
        end
    end
    yield result
end

chant double_all(lst) then
    weave result as []
    for each item in lst then
        set result to list_push(result, item * 2)
    end
    yield result
end

bind numbers to [-2, -1, 0, 1, 2, 3, 4, 5]
bind positive to filter_positive(numbers)  # [1, 2, 3, 4, 5]
bind doubled to double_all(positive)        # [2, 4, 6, 8, 10]
```

### Example 3: Working with Structs

```glimmer-weave
form Account with
    owner as Text
    balance as Number
end

chant deposit(account, amount) then
    yield Account {
        owner: account.owner,
        balance: account.balance + amount
    }
end

chant withdraw(account, amount) then
    should amount greater than account.balance then
        yield Mishap("Insufficient funds")
    otherwise
        yield Triumph(Account {
            owner: account.owner,
            balance: account.balance - amount
        })
    end
end

bind my_account to Account { owner: "Alice", balance: 100 }
bind after_deposit to deposit(my_account, 50)

bind withdrawal to withdraw(after_deposit, 30)
match withdrawal with
    when Triumph(new_account) then
        "New balance: " + to_text(new_account.balance)
    when Mishap(error) then
        "Error: " + error
end
```

---

## Running Programs

### Using the Library

Glimmer-Weave is currently a library. To run programs, use the interpreter:

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
cargo test --test test_pipeline
cargo test --test interpreter_tests

# Run with output
cargo test -- --nocapture
```

### Running Examples

Examples are located in the `examples/` directory:

```bash
# View available examples
ls examples/

# Examples can be tested via integration tests
cargo test --test test_example_pipeline
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
│   ├── bytecode.rs         # Bytecode instruction set
│   ├── bytecode_compiler.rs # Bytecode compiler
│   ├── vm.rs               # Bytecode virtual machine
│   ├── codegen.rs          # Native x86-64 code generator
│   └── runtime.rs          # Runtime functions
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

- Variables (immutable/mutable)
- Control flow (if/else, loops)
- Functions with tail-call optimization
- Pattern matching
- Custom types (structs)
- Pipeline operator
- Error handling
- Built-in collections (lists, maps)

### Planned Features 🚧

- [ ] Generics/parametric polymorphism
- [ ] Traits/interfaces
- [ ] Modules and namespaces
- [ ] Async/await
- [ ] REPL (interactive shell)
- [ ] Package manager
- [ ] LSP (Language Server Protocol) for IDE support

---

## Resources

- **Documentation**: See [CLAUDE.md](CLAUDE.md) for development guide
- **Examples**: Browse [examples/](examples/) directory
- **Tests**: See [tests/](tests/) for comprehensive test suite
- **Issue Tracker**: Use `bd` command-line tool

---

## Keyword Reference

| Keyword | Purpose | Example |
|---------|---------|---------|
| `bind` | Immutable variable | `bind x to 42` |
| `weave` | Mutable variable | `weave counter as 0` |
| `set` | Assignment | `set counter to 10` |
| `chant` | Define function | `chant add(a, b) then...end` |
| `yield` | Return from function | `yield result` |
| `should` | If statement | `should x > 0 then...end` |
| `otherwise` | Else clause | `otherwise...end` |
| `for each` | For-each loop | `for each item in list then...end` |
| `whilst` | While loop | `whilst condition then...end` |
| `match` | Pattern matching | `match x with when 1 then...end` |
| `when` | Match arm | `when 42 then "found"` |
| `form` | Define struct | `form Point with x as Number end` |
| `attempt` | Try block | `attempt then...harmonize...end` |
| `harmonize` | Catch block | `harmonize on "Error" then...end` |
| `Present` | Some/Just value | `Present(42)` |
| `Absent` | None/Nothing | `Absent` |
| `Triumph` | Ok/Success | `Triumph(result)` |
| `Mishap` | Err/Failure | `Mishap("error")` |
| `\|` | Pipeline operator | `x \| f \| g` |

---

## License

[Specify your license here - e.g., MIT, Apache 2.0]

---

## Acknowledgments

Inspired by:
- **Rust** - Memory safety, ownership, pattern matching
- **Elixir** - Pipeline operator, functional patterns
- **Swift** - Natural language syntax, readability

Designed for **AethelOS** but built to run anywhere.

---

**Questions? Feedback?** Please open an issue using the `bd` issue tracker.

Happy coding! ✨
