# Glimmer-Weave Module System Design

> **Status:** Design Phase
> **Version:** 1.0
> **Date:** 2025-11-07

---

## Overview

Glimmer-Weave's module system provides namespacing and code organization using natural language keywords that align with the language's poetic philosophy.

**Core Metaphor:** "Groves" - Collections of related code, like trees in a grove, each providing shade and shelter.

---

## Syntax Design

### 1. Module Declaration

**Keyword:** `grove` (a collection of related code)

```glimmer-weave
grove Math with
    # Module contents here
    chant sqrt(x) then
        # implementation
    end

    chant pow(base, exp) then
        # implementation
    end
end
```

**Alternative considered:** `realm`, `scroll` - rejected as less evocative of natural growth/organization.

### 2. Exporting Symbols

**Keyword:** `offer` (making functions/types available to other modules)

```glimmer-weave
grove Math with
    chant sqrt(x) then
        yield x * 0.5  # simplified
    end

    chant pow(base, exp) then
        yield base * exp  # simplified
    end

    # Private helper (not exported)
    chant _internal_helper() then
        yield 42
    end

    # Export specific functions
    offer sqrt, pow
end
```

**Alternative considered:** `share`, `provide` - `offer` chosen for its gift-like connotation.

### 3. Importing Modules

**Keyword:** `summon` (bringing a module into scope)

```glimmer-weave
# Import entire module
summon Math from "std/math.gw"

bind result to Math.sqrt(16)
bind power to Math.pow(2, 8)
```

**Alternative considered:** `invoke`, `call`, `gather` - `summon` chosen for magical/intentional connotation.

### 4. Selective Imports

**Keyword:** `gather` (collecting specific items from a module)

```glimmer-weave
# Import specific functions
gather sqrt, pow from Math

bind result to sqrt(16)  # No prefix needed
bind power to pow(2, 8)
```

### 5. Import Aliases

**Keyword:** `as` (renaming imports)

```glimmer-weave
# Alias entire module
summon Math from "std/math.gw" as M
bind result to M.sqrt(16)

# Alias specific imports
gather sqrt as square_root from Math
bind result to square_root(16)
```

### 6. Re-exports

Modules can re-export symbols from other modules:

```glimmer-weave
grove Prelude with
    summon Math from "std/math.gw"
    summon Collections from "std/collections.gw"

    # Re-export everything from Math and Collections
    offer Math, Collections
end

# Usage
summon Prelude from "std/prelude.gw"
bind result to Prelude.Math.sqrt(16)
```

---

## File Organization

### File Extensions

- `.gw` - Glimmer-Weave source files
- One module per file (convention, not enforced)

### Directory Structure

```
project/
├── main.gw              # Entry point (no grove declaration needed)
├── std/                 # Standard library
│   ├── math.gw
│   ├── collections.gw
│   └── prelude.gw
└── lib/                 # User libraries
    ├── utils.gw
    └── helpers.gw
```

### Module Resolution

Import paths are resolved in this order:

1. **Relative paths:** `from "./math.gw"` or `from "../lib/utils.gw"`
2. **Absolute paths from root:** `from "std/math.gw"` searches from project root
3. **Standard library:** `from "std/math.gw"` looks in standard library path

---

## Visibility Rules

### Default Visibility: Private

By default, all items in a module are **private** (not visible outside the module).

```glimmer-weave
grove Math with
    # Private by default
    chant _helper() then
        yield 42
    end

    # Must explicitly offer to make public
    chant sqrt(x) then
        yield x * 0.5
    end

    offer sqrt  # Only sqrt is public
end
```

### Public Items

Only items listed in `offer` are public:

```glimmer-weave
grove Example with
    bind CONSTANT to 42        # Private
    weave counter as 0         # Private

    chant public_func() then
        yield counter
    end

    chant private_func() then
        yield CONSTANT
    end

    offer public_func  # Only this is public
end
```

---

## Entry Point

The entry point file (e.g., `main.gw`) does **not** need a `grove` declaration:

```glimmer-weave
# main.gw - no grove declaration
summon Math from "std/math.gw"

bind result to Math.sqrt(16)
VGA.write(to_text(result))
```

---

## Semantic Rules

### 1. Circular Dependencies

**Rule:** Circular imports are **detected and rejected** at compile time.

```glimmer-weave
# a.gw
grove A with
    summon B from "b.gw"
end

# b.gw
grove B with
    summon A from "a.gw"  # ERROR: Circular dependency A -> B -> A
end
```

**Error Message:**
```
Semantic Error: Circular module dependency detected
  a.gw -> b.gw -> a.gw

Help: Refactor shared code into a third module that both can import
```

### 2. Name Conflicts

**Rule:** Imported names must not conflict with local names.

```glimmer-weave
summon Math from "std/math.gw"

chant sqrt(x) then  # ERROR: Name 'sqrt' conflicts with Math.sqrt
    yield x
end
```

**Error Message:**
```
Semantic Error: Name 'sqrt' conflicts with imported symbol from Math

Help: Use an alias: gather sqrt as math_sqrt from Math
```

### 3. Undefined Imports

**Rule:** All imported modules must exist and be accessible.

```glimmer-weave
summon NonExistent from "foo.gw"  # ERROR: Module not found
```

**Error Message:**
```
Semantic Error: Module not found: "foo.gw"

Searched in:
  - ./foo.gw
  - std/foo.gw

Help: Check the file path and ensure the module exists
```

---

## AST Representation

### Module Declaration

```rust
pub enum AstNode {
    // ... existing variants

    /// Module declaration: grove Name with body end
    ModuleDecl {
        name: String,
        body: Vec<Box<AstNode>>,
        exports: Vec<String>,  // Items listed in 'offer'
    },

    /// Import statement: summon Module from "path.gw"
    Import {
        module_name: String,
        path: String,
        items: Option<Vec<String>>,  // None = import all, Some = specific items
        alias: Option<String>,        // Optional 'as' alias
    },

    /// Export statement: offer item1, item2, ...
    Export {
        items: Vec<String>,
    },
}
```

### Module Qualified Access

```rust
pub enum AstNode {
    // ... existing variants

    /// Module-qualified name: Math.sqrt
    ModuleAccess {
        module: String,
        member: String,
    },
}
```

---

## Implementation Phases

### Phase 1: Parser Support (Foundation)

**Goal:** Parse module syntax without execution

**Tasks:**
1. Add tokens: `Grove`, `Offer`, `Summon`, `Gather`, `From`, `As`
2. Implement `parse_module_decl()` for `grove...end`
3. Implement `parse_import()` for `summon...from`
4. Implement `parse_export()` for `offer`
5. Implement `parse_module_access()` for `Module.member`
6. Add AST nodes: `ModuleDecl`, `Import`, `Export`, `ModuleAccess`
7. Write parser tests

### Phase 2: Module Resolver (Infrastructure)

**Goal:** Load and resolve module dependencies

**Tasks:**
1. Create `ModuleResolver` struct
2. Implement file path resolution (relative, absolute, std)
3. Implement module loading from files
4. Build module dependency graph
5. Implement circular dependency detection
6. Cache loaded modules
7. Write resolver tests

### Phase 3: Semantic Analysis (Validation)

**Goal:** Validate module usage and detect errors

**Tasks:**
1. Extend `SemanticAnalyzer` with module scope tracking
2. Implement export validation (offered items exist)
3. Implement import validation (modules exist)
4. Implement name conflict detection
5. Implement visibility checking (private vs public)
6. Implement qualified name resolution (`Module.member`)
7. Write semantic tests

### Phase 4: Interpreter Support (Execution)

**Goal:** Execute modular programs in interpreter

**Tasks:**
1. Extend `Environment` with module scopes
2. Implement module loading in `Evaluator`
3. Implement qualified access (`Module.member`)
4. Implement selective imports (`gather`)
5. Handle module initialization order
6. Write interpreter integration tests

### Phase 5: Bytecode VM Support

**Goal:** Compile and execute modules in bytecode VM

**Tasks:**
1. Extend bytecode compiler for module boundaries
2. Implement module-level compilation units
3. Add module metadata to bytecode chunks
4. Implement qualified access in VM
5. Handle cross-module function calls
6. Write VM integration tests

### Phase 6: Native Codegen Support

**Goal:** Generate native code for modular programs

**Tasks:**
1. Design module linking strategy for native code
2. Implement symbol export/import in assembly
3. Generate module-local vs global symbols
4. Handle cross-module calls in codegen
5. Write native codegen tests (or document limitations)

---

## Example Programs

### Example 1: Math Library

```glimmer-weave
# std/math.gw
grove Math with
    chant sqrt(x) then
        # Newton's method approximation
        weave guess as x / 2.0
        weave prev as 0.0

        whilst guess is not prev then
            set prev to guess
            set guess to (guess + x / guess) / 2.0
        end

        yield guess
    end

    chant pow(base, exp) then
        weave result as 1.0
        weave i as 0

        whilst i less than exp then
            set result to result * base
            set i to i + 1
        end

        yield result
    end

    offer sqrt, pow
end
```

### Example 2: Using Math Library

```glimmer-weave
# main.gw
summon Math from "std/math.gw"

bind root to Math.sqrt(16)
VGA.write("Square root of 16: " + to_text(root))

bind power to Math.pow(2, 10)
VGA.write("2^10 = " + to_text(power))
```

### Example 3: Selective Import

```glimmer-weave
# main.gw
gather sqrt, pow from "std/math.gw"

bind root to sqrt(25)      # No Math. prefix needed
bind power to pow(3, 4)

VGA.write("Results: " + to_text(root) + ", " + to_text(power))
```

### Example 4: Module with Structs

```glimmer-weave
# std/geometry.gw
grove Geometry with
    form Point with
        x as Number
        y as Number
    end

    chant distance(p1, p2) then
        bind dx to p2.x - p1.x
        bind dy to p2.y - p1.y
        yield sqrt(dx * dx + dy * dy)
    end

    offer Point, distance
end
```

---

## Design Decisions

### Why "Grove"?

- **Natural metaphor:** Groves are organized collections in nature
- **Growth connotation:** Code grows and evolves like trees
- **Protection:** Groves provide shelter, like modules provide encapsulation
- **Fits aesthetic:** Aligns with Glimmer-Weave's poetic naming

### Why "Offer"?

- **Gift-like:** Offering something to others has positive connotations
- **Intentional:** Must explicitly offer, default is private
- **Clear:** Obvious what's being exported
- **Natural:** "I offer these functions for your use"

### Why "Summon"?

- **Magical:** Fits the "weaving" metaphor
- **Intentional:** Summoning is deliberate, not accidental
- **Powerful:** Brings distant code into current scope
- **Memorable:** Distinctive and evocative

### Why File-Per-Module?

- **Simplicity:** Easy to understand and implement
- **Discoverability:** Module name = file name
- **Tooling:** Easier for editors and IDEs
- **Convention:** Matches Rust, Python, JavaScript practice

### Why Explicit Exports?

- **Safety:** Default private prevents accidental API exposure
- **Clarity:** Clear API boundaries
- **Evolution:** Can add private helpers without breaking users
- **Best practice:** Matches Rust, encourages good design

---

## Future Enhancements

### Standard Library Organization

Once modules are implemented, reorganize stdlib:

```
std/
├── prelude.gw       # Auto-imported basics
├── math.gw          # Mathematical functions
├── collections.gw   # List, Map operations
├── io.gw            # Input/output
├── text.gw          # String operations
└── iter.gw          # Iterator utilities
```

### Package Manager Integration

Future package manager (`glimmer-weave-cqf` issue) will use this module system:

```glimmer-weave
# Import from external package
summon Http from "packages/http/client.gw"
summon Json from "packages/json/parser.gw"
```

---

## Open Questions

1. **Nested modules?** Should we support `grove A with grove B with...end end`?
   - **Decision:** Not in v1. Keep it simple. Use separate files.

2. **Wildcard imports?** `gather * from Math`?
   - **Decision:** Not in v1. Explicit is better. Prevents namespace pollution.

3. **Conditional imports?** Platform-specific modules?
   - **Decision:** Not in v1. Can add later with `should` blocks.

4. **Module-level variables?** Shared state across imports?
   - **Decision:** Yes, but must be explicitly offered. Same rules as functions.

5. **Prelude auto-import?** Always import `std/prelude.gw`?
   - **Decision:** Not in v1. Explicit imports only. Can add later.

---

## Testing Strategy

### Parser Tests

- Parse valid module declarations
- Parse import statements (summon, gather)
- Parse export statements (offer)
- Parse qualified access (Module.member)
- Reject invalid syntax

### Resolver Tests

- Resolve relative paths
- Resolve absolute paths
- Detect circular dependencies
- Handle missing modules
- Cache loaded modules

### Semantic Tests

- Validate exports exist
- Detect name conflicts
- Enforce visibility rules
- Validate qualified access

### Integration Tests

- Multi-file programs
- Standard library imports
- Complex dependency chains
- Error messages are helpful

---

## References

- Rust module system: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html
- Python imports: https://docs.python.org/3/reference/import.html
- Glimmer-Weave design philosophy: See CLAUDE.md

---

**End of Design Document**
