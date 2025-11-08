# Ownership and Borrowing System Design

**Issue:** glimmer-weave-bp9
**Status:** In Design
**Date:** 2025-11-07
**Priority:** P3 (Foundation for memory safety)

---

## Table of Contents

1. [Overview](#overview)
2. [Motivation](#motivation)
3. [Core Principles](#core-principles)
4. [Syntax Design](#syntax-design)
5. [Ownership Rules](#ownership-rules)
6. [Borrowing Rules](#borrowing-rules)
7. [Lifetime System](#lifetime-system)
8. [Implementation Plan](#implementation-plan)
9. [Examples](#examples)
10. [Migration Strategy](#migration-strategy)

---

## Overview

This document defines the ownership and borrowing system for Glimmer-Weave, bringing **Rust-level memory safety** to the language while maintaining its natural language philosophy.

**Goals:**
- Prevent use-after-free, double-free, and dangling pointers at **compile time**
- Zero runtime overhead (all checks done during compilation)
- Memory safety without garbage collection
- Natural, expressive syntax following Glimmer-Weave conventions

**Key Concepts:**
1. **Ownership** - Every value has a single owner that controls its lifetime
2. **Move Semantics** - Transferring ownership invalidates the previous owner
3. **Borrowing** - Temporary access without transferring ownership
4. **Lifetimes** - Tracking how long references remain valid

---

## Motivation

### Current State: No Memory Safety Guarantees

```glimmer-weave
# Current Glimmer-Weave (unsafe patterns possible)

bind data to [1, 2, 3]
bind reference to data     # Copy? Reference? Move? Unclear!

# What happens here?
chant modify(list) then
    # Does this mutate the original or a copy?
    list.push(4)
end

modify(data)
# Is 'data' still valid? Did it change?
```

**Problems:**
- Unclear ownership semantics
- No protection against use-after-free
- No guarantees about mutation
- Potential memory leaks in native runtime

### Proposed: Rust-Level Safety

```glimmer-weave
# With ownership system (safe by default)

bind data to [1, 2, 3]           # data owns the list
bind moved to data               # MOVE: data is now invalid
# data.length()                  # ERROR: value was moved

bind data2 to [4, 5, 6]
chant sum(borrow list) -> Number then
    # list is borrowed (read-only)
    yield list.fold(0, |acc, x| acc + x)
end

bind total to sum(borrow data2)  # Borrow: data2 still valid after call
data2.push(7)                    # OK: data2 still owned here
```

**Benefits:**
- Clear ownership at all times
- Compile-time prevention of memory bugs
- Safe mutation with mutable borrows
- Explicit lifetimes for complex cases

---

## Core Principles

### 1. Ownership

> **Every value has exactly one owner at any time.**

When a value is created, the variable that binds it becomes the owner:

```glimmer-weave
bind owner to [1, 2, 3]  # 'owner' owns the list
```

When the owner goes out of scope, the value is automatically deallocated.

### 2. Move Semantics

> **Assigning or passing a value transfers ownership (moves it).**

```glimmer-weave
bind x to [1, 2, 3]
bind y to x            # MOVE: x is invalidated, y is now owner

# x.length()          # COMPILE ERROR: value was moved out of x
y.length()            # OK: y owns the value
```

### 3. Borrowing

> **Temporary access to a value without taking ownership.**

Two kinds of borrows:

**Shared Borrow** (`borrow`): Read-only access, many allowed simultaneously
```glimmer-weave
bind data to [1, 2, 3]
bind ref1 to borrow data    # Shared borrow
bind ref2 to borrow data    # OK: multiple shared borrows allowed
# data.push(4)              # ERROR: cannot mutate while borrowed
```

**Mutable Borrow** (`borrow mut`): Exclusive write access, only one allowed
```glimmer-weave
bind data to [1, 2, 3]
bind ref to borrow mut data  # Mutable borrow
ref.push(4)                  # OK: can mutate through mutable borrow
# bind ref2 to borrow data   # ERROR: cannot borrow while mutably borrowed
```

### 4. Copy Types

> **Simple types are copied instead of moved.**

**Copy Types** (trivial to duplicate):
- `Number` (64-bit float)
- `Truth` (boolean)
- `Nothing` (unit type)
- Small structs marked with `copy` aspect

```glimmer-weave
bind x to 42
bind y to x    # COPY: both x and y have value 42
x + y          # OK: x is still valid (was copied, not moved)
```

**Move Types** (expensive to duplicate):
- `Text` (heap-allocated string)
- `List<T>` (heap-allocated vector)
- `Map` (heap-allocated hash map)
- Custom structs (unless marked `copy`)

---

## Syntax Design

### Keywords

Following Glimmer-Weave's natural language philosophy:

| Concept | Rust | Glimmer-Weave | Metaphor |
|---------|------|---------------|----------|
| Shared borrow | `&x` | `borrow x` | Lending a book (read-only) |
| Mutable borrow | `&mut x` | `borrow mut x` | Lending for annotation |
| Lifetime annotation | `'a` | `'span` | Duration/timespan |
| Move | implicit | implicit | Ownership transfer |
| Clone | `.clone()` | `.replicate()` | Explicit duplication |

**Design Rationale:**
- `borrow` is natural English for temporary access
- `borrow mut` is explicit about mutability
- `'span` is more intuitive than `'a` for lifetimes
- `.replicate()` emphasizes that copying is intentional

### Function Parameters

**Move (takes ownership):**
```glimmer-weave
chant consume(data as List<Number>) then
    # Function owns 'data', caller loses access
end

bind nums to [1, 2, 3]
consume(nums)
# nums is no longer valid here
```

**Shared Borrow (read-only):**
```glimmer-weave
chant sum(borrow data as List<Number>) -> Number then
    # Function borrows 'data' (cannot modify)
    yield data.fold(0, |acc, x| acc + x)
end

bind nums to [1, 2, 3]
bind total to sum(borrow nums)  # nums still valid after call
```

**Mutable Borrow (exclusive write access):**
```glimmer-weave
chant add_one(borrow mut data as List<Number>) then
    # Function can modify 'data'
    data.push(data.length() + 1)
end

bind nums to [1, 2, 3]
add_one(borrow mut nums)  # nums is modified in place
# nums is now [1, 2, 3, 4]
```

### Return Values

**Return owned value:**
```glimmer-weave
chant create_list() -> List<Number> then
    bind list to [1, 2, 3]
    yield list  # Ownership transferred to caller
end

bind data to create_list()  # data owns the returned list
```

**Return borrowed reference:**
```glimmer-weave
chant first<'span>(borrow 'span list as List<Number>) -> borrow 'span Number then
    yield borrow list[0]  # Return reference with same lifetime as input
end

bind nums to [1, 2, 3]
bind first_elem to first(borrow nums)  # first_elem borrows from nums
```

### Type Annotations

**Borrowed reference types:**
```glimmer-weave
bind x: Number to 42                    # Owned number
bind ref: borrow Number to borrow x     # Shared borrow
bind mutref: borrow mut Number to borrow mut x  # Mutable borrow
```

**Function types with borrows:**
```glimmer-weave
# Function that takes borrowed list and returns owned number
bind func: (borrow List<Number>) -> Number to sum
```

---

## Ownership Rules

### Rule 1: Single Owner

**At any time, a value has exactly one owner.**

```glimmer-weave
bind owner1 to [1, 2, 3]
bind owner2 to owner1       # MOVE: owner1 invalidated
# owner1.length()           # ERROR: value moved out
owner2.length()             # OK
```

**Compiler Error:**
```
Error: Value used after move
  |
4 | owner1.length()
  | ^^^^^^ value moved out in line 2
  |
  = note: Consider using `borrow owner2` if you need shared access
```

### Rule 2: Scoped Lifetime

**When the owner goes out of scope, the value is dropped.**

```glimmer-weave
chant example() then
    bind data to [1, 2, 3]  # data created
    data.length()           # OK
end  # data dropped here (memory freed)

# data does not exist outside the function
```

**Nested scopes:**
```glimmer-weave
bind outer to [1, 2, 3]

should true then
    bind inner to [4, 5, 6]
    inner.length()  # OK
end  # inner dropped here

outer.length()  # OK: outer still in scope
```

### Rule 3: Move on Assignment

**Assignment transfers ownership (except for Copy types).**

```glimmer-weave
# Move types (heap-allocated)
bind s1 to "hello"
bind s2 to s1       # MOVE
# s1.length()       # ERROR

# Copy types (stack-only)
bind n1 to 42
bind n2 to n1       # COPY
n1 + n2             # OK: both valid
```

### Rule 4: Move on Function Call

**Passing to a function transfers ownership (unless borrowing).**

```glimmer-weave
chant takes_ownership(data as List<Number>) then
    # data is owned here
end

bind nums to [1, 2, 3]
takes_ownership(nums)  # nums moved into function
# nums.length()        # ERROR: value was moved
```

**Borrowing avoids the move:**
```glimmer-weave
chant borrows_data(borrow data as List<Number>) then
    # data is borrowed here
end

bind nums to [1, 2, 3]
borrows_data(borrow nums)  # nums is borrowed, not moved
nums.length()              # OK: nums still valid
```

### Rule 5: Explicit Cloning

**Use `.replicate()` to create deep copies.**

```glimmer-weave
bind original to [1, 2, 3]
bind copy to original.replicate()  # Explicit deep copy

original.push(4)  # original is [1, 2, 3, 4]
copy.push(5)      # copy is [1, 2, 3, 5]
# Independent values
```

---

## Borrowing Rules

### Rule 1: Many Shared OR One Mutable

**At any time, you can have EITHER:**
- **One mutable borrow** (exclusive write access)
- **Any number of shared borrows** (read-only)

**Cannot mix mutable and shared borrows.**

```glimmer-weave
bind data to [1, 2, 3]

# OK: Multiple shared borrows
bind ref1 to borrow data
bind ref2 to borrow data
bind ref3 to borrow data

# OK: Single mutable borrow (no other borrows)
bind mutref to borrow mut data

# ERROR: Cannot have mutable + shared
bind ref1 to borrow data
bind mutref to borrow mut data  # ERROR: already borrowed as shared
```

**Compiler Error:**
```
Error: Cannot borrow mutably while shared borrows exist
  |
3 | bind mutref to borrow mut data
  |                ^^^^^^^^^^^^^^^ mutable borrow occurs here
2 | bind ref1 to borrow data
  |              ----------- shared borrow occurs here
  |
  = note: Mutable borrow requires exclusive access
```

### Rule 2: Borrows Must Not Outlive Owner

**A borrow cannot live longer than the value it references.**

```glimmer-weave
bind ref: borrow Number

should true then
    bind value to 42
    ref to borrow value
end  # value dropped here

# ref is now dangling!  # ERROR: ref outlives value
```

**Compiler Error:**
```
Error: Borrowed value does not live long enough
  |
5 | end
  | ^^^ value dropped here while still borrowed
3 | ref to borrow value
  | --- borrow occurs here
  |
  = note: Consider moving the borrow outside the inner scope
```

### Rule 3: No Mutation During Shared Borrow

**While a value is borrowed (shared), the owner cannot modify it.**

```glimmer-weave
bind data to [1, 2, 3]
bind ref to borrow data  # Shared borrow

data.push(4)  # ERROR: cannot mutate while borrowed
```

**Compiler Error:**
```
Error: Cannot mutate while borrowed
  |
4 | data.push(4)
  | ^^^^ cannot mutate borrowed value
3 | bind ref to borrow data
  | --- value borrowed here as shared
```

### Rule 4: Borrow Scope

**Borrows end when they go out of scope (or are last used).**

```glimmer-weave
bind data to [1, 2, 3]

bind ref to borrow data
ref.length()  # Last use of ref

# Borrow ends here, data can be mutated again
data.push(4)  # OK
```

**Non-Lexical Lifetimes (NLL):**
Borrows end at their last use, not the end of the scope:

```glimmer-weave
bind data to [1, 2, 3]

bind ref to borrow data
VGA.write(to_text(ref.length()))  # Last use of ref

# ref is no longer used, borrow ends
data.push(4)  # OK: borrow has ended
```

---

## Lifetime System

### Basic Lifetime Annotations

**Lifetimes track how long references remain valid.**

**Syntax:** `'span` (pronounced "tick-span")

```glimmer-weave
chant first<'span>(borrow 'span list as List<Number>) -> borrow 'span Number then
    yield borrow list[0]
end
```

**Meaning:**
- The returned reference has the same lifetime (`'span`) as the input
- The returned reference is valid as long as `list` is valid

### Lifetime Elision Rules

**For simple cases, lifetimes are inferred automatically.**

**Rule 1:** Each borrowed parameter gets its own lifetime.
```glimmer-weave
# Written:
chant process(borrow data as List<Number>) then ... end

# Inferred:
chant process<'span>(borrow 'span data as List<Number>) then ... end
```

**Rule 2:** If there's exactly one input lifetime, it's assigned to all output lifetimes.
```glimmer-weave
# Written:
chant first(borrow list as List<Number>) -> borrow Number then ... end

# Inferred:
chant first<'span>(borrow 'span list as List<Number>) -> borrow 'span Number then ... end
```

**Rule 3:** If there's a `self` parameter, its lifetime is assigned to outputs.
```glimmer-weave
# Written:
chant get_name(borrow self) -> borrow Text then ... end

# Inferred:
chant get_name<'span>(borrow 'span self) -> borrow 'span Text then ... end
```

### Multiple Lifetimes

**When a function has multiple borrowed inputs with different lifetimes:**

```glimmer-weave
chant longest<'a, 'b>(
    borrow 'a first as Text,
    borrow 'b second as Text
) -> borrow 'a Text then
    should first.length() > second.length() then
        yield first   # Lifetime 'a
    otherwise
        yield second  # ERROR: second has lifetime 'b, not 'a
    end
end
```

**To return either reference, use a common lifetime:**

```glimmer-weave
chant longest<'span>(
    borrow 'span first as Text,
    borrow 'span second as Text
) -> borrow 'span Text then
    should first.length() > second.length() then
        yield first   # OK: both have lifetime 'span
    otherwise
        yield second  # OK
    end
end
```

### Static Lifetime

**`'static` lifetime lasts for the entire program.**

```glimmer-weave
# String literals have 'static lifetime
bind msg: borrow 'static Text to "Hello, World!"

# Can outlive any scope
chant get_message() -> borrow 'static Text then
    yield "Static message"  # OK: string literal is 'static
end
```

---

## Implementation Plan

### Phase 1: AST Extensions (1 week)

**Add ownership annotations to AST nodes.**

**Changes to `src/ast.rs`:**

```rust
/// Borrow mode for parameters and types
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowMode {
    Owned,        // Default: takes ownership
    Borrowed,     // borrow (shared/immutable)
    BorrowedMut,  // borrow mut (exclusive/mutable)
}

/// Lifetime annotation
#[derive(Debug, Clone, PartialEq)]
pub struct Lifetime {
    pub name: String,  // 'span, 'a, 'static, etc.
}

/// Enhanced parameter with borrow mode
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub typ: Option<TypeAnnotation>,
    pub borrow_mode: BorrowMode,  // NEW
    pub lifetime: Option<Lifetime>,  // NEW
    pub is_variadic: bool,
}

/// Enhanced type annotation with borrows
#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    Named(String),
    Generic(String),
    Parametrized { name: String, type_args: Vec<TypeAnnotation> },
    List(Box<TypeAnnotation>),
    Map,
    Function {
        param_types: Vec<TypeAnnotation>,
        return_type: Box<TypeAnnotation>,
    },
    Optional(Box<TypeAnnotation>),

    // NEW: Borrowed types
    Borrowed {
        lifetime: Option<Lifetime>,
        inner: Box<TypeAnnotation>,
        mutable: bool,  // true for borrow mut
    },
}

/// Enhanced function definition with lifetimes
pub enum AstNode {
    // ...
    ChantDef {
        name: String,
        type_params: Vec<String>,
        lifetime_params: Vec<Lifetime>,  // NEW: <'span, 'a>
        params: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: Vec<AstNode>,
    },
    // ...
}
```

**Testing:**
- Update parser tests to handle borrow annotations
- Verify AST correctly represents borrowed types

### Phase 2: Lexer & Parser (1 week)

**Add tokens and parsing for borrow syntax.**

**Changes to `src/token.rs`:**

```rust
pub enum Token {
    // ... existing tokens

    /// `borrow` - Borrow keyword
    Borrow,

    /// `mut` - Mutable keyword (already exists for weave)
    Mut,

    /// `'span`, `'a`, etc. - Lifetime annotation
    Lifetime(String),
}
```

**Changes to `src/parser.rs`:**

```rust
fn parse_parameter(&mut self) -> ParseResult<Parameter> {
    // Check for borrow mode
    let borrow_mode = match self.current() {
        Token::Borrow => {
            self.advance();
            // Check for 'mut'
            if self.current() == &Token::Mut {
                self.advance();
                BorrowMode::BorrowedMut
            } else {
                BorrowMode::Borrowed
            }
        }
        _ => BorrowMode::Owned,
    };

    // Check for lifetime
    let lifetime = match self.current() {
        Token::Lifetime(name) => {
            let lt = Some(Lifetime { name: name.clone() });
            self.advance();
            lt
        }
        _ => None,
    };

    // Parse parameter name
    let name = self.expect_ident()?;

    // ... rest of parameter parsing
}
```

**Testing:**
- Parse `borrow x` correctly
- Parse `borrow mut x` correctly
- Parse `borrow 'span x as List<Number>` correctly
- Parse function with lifetimes: `chant foo<'span>(borrow 'span x as Number)`

### Phase 3: Semantic Analysis - Borrow Checker (2-3 weeks)

**Implement borrow checking in semantic analyzer.**

**New module: `src/borrow_checker.rs`:**

```rust
use crate::ast::*;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Tracks the state of a variable (owned, moved, borrowed)
#[derive(Debug, Clone, PartialEq)]
enum VarState {
    Owned,                 // Variable owns the value
    Moved,                 // Value was moved out
    Borrowed(Vec<BorrowId>),    // Shared borrows active
    BorrowedMut(BorrowId),      // Exclusive borrow active
}

/// Unique identifier for a borrow
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BorrowId(usize);

/// Scope where a borrow is active
#[derive(Debug, Clone)]
struct BorrowScope {
    id: BorrowId,
    var_name: String,
    is_mutable: bool,
    active: bool,  // Ends at last use (NLL)
}

pub struct BorrowChecker {
    /// Variable states in current scope
    var_states: BTreeMap<String, VarState>,

    /// Active borrows
    active_borrows: Vec<BorrowScope>,

    /// Next borrow ID
    next_borrow_id: usize,

    /// Scope stack for nested blocks
    scope_stack: Vec<BTreeMap<String, VarState>>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        BorrowChecker {
            var_states: BTreeMap::new(),
            active_borrows: Vec::new(),
            next_borrow_id: 0,
            scope_stack: Vec::new(),
        }
    }

    /// Check ownership/borrowing rules for a program
    pub fn check(&mut self, nodes: &[AstNode]) -> Result<(), String> {
        for node in nodes {
            self.check_node(node)?;
        }
        Ok(())
    }

    fn check_node(&mut self, node: &AstNode) -> Result<(), String> {
        match node {
            AstNode::BindStmt { name, value, .. } => {
                // Check if value is moved/borrowed
                self.check_expr(value)?;

                // Register new variable as owned
                self.var_states.insert(name.clone(), VarState::Owned);
                Ok(())
            }

            AstNode::SetStmt { name, value } => {
                // Check if variable can be mutated
                self.check_mutable_access(name)?;

                // Check if value is valid
                self.check_expr(value)?;
                Ok(())
            }

            AstNode::Ident(name) => {
                // Check if variable is valid (not moved)
                self.check_access(name)?;

                // If this is a move context, mark as moved
                if self.is_move_context() {
                    self.mark_moved(name)?;
                }
                Ok(())
            }

            // ... handle all AST nodes

            _ => Ok(()),
        }
    }

    /// Check if a variable can be accessed
    fn check_access(&self, name: &str) -> Result<(), String> {
        match self.var_states.get(name) {
            Some(VarState::Owned) => Ok(()),
            Some(VarState::Borrowed(_)) => Ok(()),  // Can read while borrowed
            Some(VarState::BorrowedMut(_)) => Ok(()),
            Some(VarState::Moved) => {
                Err(format!("Value '{}' was moved and cannot be used", name))
            }
            None => {
                Err(format!("Variable '{}' not found", name))
            }
        }
    }

    /// Check if a variable can be mutated
    fn check_mutable_access(&self, name: &str) -> Result<(), String> {
        match self.var_states.get(name) {
            Some(VarState::Owned) => Ok(()),
            Some(VarState::BorrowedMut(_)) => Ok(()),
            Some(VarState::Borrowed(_)) => {
                Err(format!("Cannot mutate '{}' while borrowed", name))
            }
            Some(VarState::Moved) => {
                Err(format!("Cannot mutate '{}' after move", name))
            }
            None => {
                Err(format!("Variable '{}' not found", name))
            }
        }
    }

    /// Mark a variable as moved
    fn mark_moved(&mut self, name: &str) -> Result<(), String> {
        // Cannot move if borrowed
        match self.var_states.get(name) {
            Some(VarState::Borrowed(_)) => {
                return Err(format!("Cannot move '{}' while borrowed", name));
            }
            Some(VarState::BorrowedMut(_)) => {
                return Err(format!("Cannot move '{}' while mutably borrowed", name));
            }
            _ => {}
        }

        self.var_states.insert(name.to_string(), VarState::Moved);
        Ok(())
    }

    /// Create a shared borrow
    fn borrow_shared(&mut self, name: &str) -> Result<BorrowId, String> {
        // Cannot borrow if mutably borrowed
        match self.var_states.get(name) {
            Some(VarState::BorrowedMut(_)) => {
                return Err(format!("Cannot borrow '{}' while mutably borrowed", name));
            }
            Some(VarState::Moved) => {
                return Err(format!("Cannot borrow '{}' after move", name));
            }
            _ => {}
        }

        let borrow_id = BorrowId(self.next_borrow_id);
        self.next_borrow_id += 1;

        // Add to active borrows
        self.active_borrows.push(BorrowScope {
            id: borrow_id,
            var_name: name.to_string(),
            is_mutable: false,
            active: true,
        });

        // Update variable state
        let current = self.var_states.get(name).cloned().unwrap_or(VarState::Owned);
        match current {
            VarState::Borrowed(mut ids) => {
                ids.push(borrow_id);
                self.var_states.insert(name.to_string(), VarState::Borrowed(ids));
            }
            _ => {
                self.var_states.insert(name.to_string(), VarState::Borrowed(vec![borrow_id]));
            }
        }

        Ok(borrow_id)
    }

    /// Create a mutable borrow
    fn borrow_mut(&mut self, name: &str) -> Result<BorrowId, String> {
        // Cannot borrow mutably if any borrows exist
        match self.var_states.get(name) {
            Some(VarState::Borrowed(_)) => {
                return Err(format!("Cannot borrow '{}' mutably while borrowed", name));
            }
            Some(VarState::BorrowedMut(_)) => {
                return Err(format!("Cannot borrow '{}' mutably more than once", name));
            }
            Some(VarState::Moved) => {
                return Err(format!("Cannot borrow '{}' after move", name));
            }
            _ => {}
        }

        let borrow_id = BorrowId(self.next_borrow_id);
        self.next_borrow_id += 1;

        // Add to active borrows
        self.active_borrows.push(BorrowScope {
            id: borrow_id,
            var_name: name.to_string(),
            is_mutable: true,
            active: true,
        });

        // Update variable state
        self.var_states.insert(name.to_string(), VarState::BorrowedMut(borrow_id));

        Ok(borrow_id)
    }

    /// End a borrow (called at last use for NLL)
    fn end_borrow(&mut self, borrow_id: BorrowId) {
        // Mark borrow as inactive
        for borrow in &mut self.active_borrows {
            if borrow.id == borrow_id {
                borrow.active = false;

                // Restore variable to owned state
                let name = borrow.var_name.clone();
                self.var_states.insert(name, VarState::Owned);
                break;
            }
        }
    }
}
```

**Integration with `src/semantic.rs`:**

```rust
pub struct SemanticAnalyzer {
    // ... existing fields
    borrow_checker: BorrowChecker,  // NEW
}

impl SemanticAnalyzer {
    pub fn analyze(&mut self, nodes: &[AstNode]) -> Result<(), String> {
        // ... existing analysis

        // NEW: Run borrow checker
        if self.check_borrowing {
            self.borrow_checker.check(nodes)?;
        }

        Ok(())
    }
}
```

**Testing:**
- Test move semantics: variable invalid after move
- Test shared borrows: multiple readers allowed
- Test mutable borrows: exclusive writer
- Test cannot mutate while borrowed
- Test borrows end at last use (NLL)

### Phase 4: Lifetime Inference (1-2 weeks)

**Implement lifetime inference for simple cases.**

**New module: `src/lifetime_checker.rs`:**

```rust
/// Infer lifetimes for function signatures
pub struct LifetimeInference {
    // ... lifetime analysis
}

impl LifetimeInference {
    /// Apply lifetime elision rules
    pub fn infer_lifetimes(&mut self, func: &mut AstNode) -> Result<(), String> {
        // Rule 1: Each borrowed param gets its own lifetime
        // Rule 2: Single input → all outputs get that lifetime
        // Rule 3: self param → outputs get self's lifetime
        // ...
    }
}
```

**Testing:**
- Infer simple function lifetimes
- Validate complex lifetime relationships
- Error on ambiguous lifetimes

### Phase 5: Error Messages (1 week)

**Provide helpful, actionable error messages.**

**Examples:**

```
Error: Cannot borrow 'data' mutably while already borrowed
  --> example.gw:5:10
   |
 3 | bind ref to borrow data
   |              ---------- first borrow occurs here
 4 |
 5 | bind mut_ref to borrow mut data
   |                 ^^^^^^^^^^^^^^^ mutable borrow occurs here
   |
   = help: Consider dropping 'ref' before creating 'mut_ref'
   = note: Shared borrows prevent mutation to ensure data consistency
```

```
Error: Value 'x' moved out and cannot be used
  --> example.gw:4:1
   |
 2 | bind x to [1, 2, 3]
   |      - value created here
 3 | bind y to x
   |           - value moved here
 4 | x.length()
   | ^ value used here after move
   |
   = help: Use 'borrow x' if you need shared access
   = help: Use 'x.replicate()' if you need an independent copy
   = note: Lists are move types (heap-allocated)
```

### Phase 6: Documentation & Examples (1 week)

**Update CLAUDE.md and create tutorial.**

- Add ownership section to language guide
- Create examples demonstrating patterns
- Document migration from current code

---

## Examples

### Example 1: Basic Ownership

```glimmer-weave
# Create owned value
bind data to [1, 2, 3]  # data owns the list

# Move value
bind moved to data      # data is invalidated
# data.length()         # ERROR: value moved

# Use moved value
moved.push(4)           # OK: moved owns the value
```

### Example 2: Shared Borrowing

```glimmer-weave
chant sum(borrow list as List<Number>) -> Number then
    bind total to 0
    for each item in borrow list then
        set total to total + item
    end
    yield total
end

bind numbers to [1, 2, 3, 4, 5]
bind result to sum(borrow numbers)
VGA.write("Sum: " + to_text(result))
# numbers is still valid here
```

### Example 3: Mutable Borrowing

```glimmer-weave
chant double_all(borrow mut list as List<Number>) then
    for each i in range(0, list.length()) then
        list[i] to list[i] * 2
    end
end

bind numbers to [1, 2, 3]
double_all(borrow mut numbers)
# numbers is now [2, 4, 6]
```

### Example 4: Lifetimes in Return Values

```glimmer-weave
# Return type has same lifetime as input
chant first<'span>(borrow 'span list as List<Number>) -> borrow 'span Number then
    yield borrow list[0]
end

bind numbers to [10, 20, 30]
bind first_num to first(borrow numbers)
VGA.write("First: " + to_text(first_num))
# first_num is valid as long as numbers is valid
```

### Example 5: Multiple Lifetimes

```glimmer-weave
# Inputs have different lifetimes, but output must pick one
chant choose<'a, 'b>(
    borrow 'a first as Text,
    borrow 'b second as Text,
    flag as Truth
) -> borrow 'a Text then  # Returns 'a lifetime
    should flag then
        yield first   # OK: lifetime 'a
    otherwise
        # ERROR: second has lifetime 'b, not 'a
        # yield second

        # Fix: Use common lifetime
        yield first  # Always return first (same lifetime)
    end
end
```

**Fixed version with common lifetime:**

```glimmer-weave
chant choose<'span>(
    borrow 'span first as Text,
    borrow 'span second as Text,
    flag as Truth
) -> borrow 'span Text then
    should flag then
        yield first   # OK
    otherwise
        yield second  # OK: both have 'span lifetime
    end
end

bind s1 to "Hello"
bind s2 to "World"
bind choice to choose(borrow s1, borrow s2, true)
VGA.write(choice)
```

### Example 6: Struct with Borrows

```glimmer-weave
# Struct holding borrowed data
form Slice<'span, T> with
    data: borrow 'span List<T>
    start: Number
    end: Number
end

chant make_slice<'span, T>(
    borrow 'span list as List<T>,
    start as Number,
    end as Number
) -> Slice<'span, T> then
    yield Slice {
        data: borrow list,
        start: start,
        end: end
    }
end

bind numbers to [1, 2, 3, 4, 5]
bind slice to make_slice(borrow numbers, 1, 3)
# slice is valid as long as numbers is valid
```

### Example 7: Iterator Pattern

```glimmer-weave
# Iterator borrows the collection
form Iter<'span, T> with
    data: borrow 'span List<T>
    index: Number
end

aspect Iterator<'span, T> then
    chant next(borrow mut self) -> Maybe<borrow 'span T>
end

embody Iterator<'span, T> for Iter<'span, T> then
    chant next(borrow mut self) -> Maybe<borrow 'span T> then
        should self.index < self.data.length() then
            bind elem to borrow self.data[self.index]
            set self.index to self.index + 1
            yield Present(elem)
        otherwise
            yield Absent
        end
    end
end

# Usage
bind numbers to [1, 2, 3]
weave iter as Iter { data: borrow numbers, index: 0 }

whilst true then
    match iter.next() with
        when Present(value) then
            VGA.write(to_text(value))
        when Absent then
            break
    end
end
```

---

## Migration Strategy

### Opt-In System

**Phase 1:** Ownership checking is **opt-in** (disabled by default)

```glimmer-weave
# At top of file, enable ownership checking
enable ownership_checking

# Code below must follow ownership rules
bind data to [1, 2, 3]
bind moved to data
# data.length()  # ERROR (with ownership checking enabled)
```

**Phase 2:** After testing, make it **default** (opt-out)

```glimmer-weave
# Disable for legacy code
disable ownership_checking

# Code below has old semantics (copies by default)
bind data to [1, 2, 3]
bind copy to data
data.length()  # OK (ownership checking disabled)
```

### Backward Compatibility

**Current code continues to work unchanged:**

```glimmer-weave
# Without ownership checking (default for Phase 1)
bind x to [1, 2, 3]
bind y to x
x.length()  # OK: creates copy (old behavior)
```

**Gradual adoption:**

1. Enable ownership checking in new modules
2. Migrate existing modules one at a time
3. Eventually make ownership checking mandatory

### Migration Tools

**Auto-suggest borrows:**

```glimmer-weave
# Old code
chant process(data as List<Number>) then
    data.length()
end

bind nums to [1, 2, 3]
process(nums)
nums.push(4)  # WARNING: nums moved in process()
```

**Compiler suggestion:**

```
Warning: Value may be moved
  --> example.gw:7:1
   |
 6 | process(nums)
   |         ---- value moved here
 7 | nums.push(4)
   | ^^^^ value used after move
   |
   = help: Change function signature to borrow:
           chant process(borrow data as List<Number>) then
   = help: Or explicitly clone the value:
           process(nums.replicate())
```

---

## Timeline

**Total Estimate: 7-10 weeks**

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1. AST Extensions | 1 week | Borrow annotations in AST |
| 2. Lexer & Parser | 1 week | Parse `borrow` syntax |
| 3. Borrow Checker | 2-3 weeks | Ownership/borrowing validation |
| 4. Lifetime Inference | 1-2 weeks | Automatic lifetime elision |
| 5. Error Messages | 1 week | Helpful diagnostics |
| 6. Documentation | 1 week | Guide & examples |
| **Total** | **7-10 weeks** | Production-ready system |

**Dependencies:**
- Requires existing type system (generics, traits) ✅ DONE
- Blocks smart pointers implementation
- Blocks lifetime system (glimmer-weave-roo)

---

## Open Questions

1. **Copy vs Clone semantics:**
   - Should `Number` always copy, or require explicit `replicate()`?
   - **Decision:** Numbers/Truth/Nothing always copy (no `replicate()` needed)

2. **Interior mutability:**
   - How to handle `Cell<T>` / `RefCell<T>` equivalents?
   - **Decision:** Implement in Phase 3 (after smart pointers)

3. **Drop trait:**
   - Do we need explicit `Drop` aspect for destructors?
   - **Decision:** Yes, for resource management (file handles, etc.)

4. **Pin and unsafe:**
   - Do we need `Pin` for self-referential structs?
   - Do we expose `unsafe` escape hatch?
   - **Decision:** Defer to Phase 4 (advanced features)

---

## Success Criteria

- [ ] All ownership rules enforced at compile time
- [ ] Borrow checker prevents use-after-free
- [ ] Lifetime inference works for 90%+ of cases
- [ ] Clear error messages with actionable suggestions
- [ ] Zero runtime overhead (all checks at compile time)
- [ ] Backward compatible with existing code
- [ ] Comprehensive test coverage (50+ tests)
- [ ] Documentation with 10+ examples

---

**Next Steps:**
1. Review this design with stakeholders
2. Get approval on syntax choices
3. Begin Phase 1 implementation (AST extensions)
4. Create tracking issue for each phase

**Status:** Design complete, awaiting review
**Updated:** 2025-11-07
