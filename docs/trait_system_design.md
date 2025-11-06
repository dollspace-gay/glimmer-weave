# Trait System - Design Document

## Overview

Add a Rust-like trait system to Glimmer-Weave for polymorphism and code reuse. Traits define shared behavior that types can implement, enabling generic programming with constraints.

## Design Philosophy

**Polymorphism Through Traits**: Traits enable writing generic code that works with any type implementing required behavior, promoting code reuse while maintaining type safety.

## Syntax Design

### Keywords

Following Glimmer-Weave's natural language style:
- **`aspect`**: Define a trait (an aspect of behavior)
- **`embody`**: Implement a trait for a type (the type embodies the aspect)
- **`for`**: Specifies which type implements the trait

### Trait Definition

```glimmer
aspect TraitName then
    chant method_name(self, param: Type) -> ReturnType
    chant another_method(self) -> Type
end
```

Traits can have generic type parameters:

```glimmer
aspect Container<T> then
    chant add(self, item: T)
    chant get(self, index: Number) -> T
end
```

### Trait Implementation

```glimmer
embody TraitName for TypeName then
    chant method_name(self, param: Type) -> ReturnType then
        # Implementation
        yield result
    end
end
```

With generic traits:

```glimmer
embody Container<Number> for NumberList then
    chant add(self, item: Number) then
        # Implementation
    end

    chant get(self, index: Number) -> Number then
        # Implementation
        yield result
    end
end
```

### Trait Bounds on Generics

Constrain generic type parameters to types implementing specific traits:

```glimmer
chant print_all<T: Display>(items: List<T>) then
    for each item in items then
        reveal(item.show())
    end
end
```

Multiple trait bounds:

```glimmer
chant sort_and_display<T: Sortable + Display>(items: List<T>) then
    bind sorted to items.sort()
    for each item in sorted then
        reveal(item.show())
    end
end
```

### Using Trait Methods

Once a trait is implemented for a type, its methods can be called:

```glimmer
bind num to 42
bind text to num.show()  # Calls Display.show() for Number
```

## Semantics

### Trait Definition Rules

1. **Method Signatures Only**: Trait methods are declarations without implementations
2. **self Parameter**: All trait methods must have `self` as first parameter
3. **Type Parameters**: Traits can be generic over type parameters
4. **No Fields**: Traits cannot have fields (only methods)

### Trait Implementation Rules

1. **Complete Implementation**: All trait methods must be implemented
2. **Signature Match**: Method signatures must exactly match the trait definition
3. **One Implementation Per Type**: Each type can implement a trait at most once
4. **Orphan Rule**: Either the trait or the type must be defined in current crate (for package system)

### Method Resolution

When calling a method on a value:
1. Check if the type has a direct method with that name
2. Check all trait implementations for that type
3. Report ambiguity error if multiple traits provide same method name

### Trait Bounds

Generic functions can constrain type parameters:
- **Single bound**: `<T: Display>` - T must implement Display
- **Multiple bounds**: `<T: Display + Clone>` - T must implement both
- **Bound checking**: Compiler verifies all trait methods are available

## Examples

### Example 1: Display Trait

```glimmer
# Define Display aspect
aspect Display then
    chant show(self) -> Text
end

# Implement for Number
embody Display for Number then
    chant show(self) -> Text then
        yield to_text(self)
    end
end

# Implement for Text (identity)
embody Display for Text then
    chant show(self) -> Text then
        yield self
    end
end

# Generic function with trait bound
chant print_value<T: Display>(value: T) then
    reveal(value.show())
end

print_value<Number>(42)        # Prints "42"
print_value<Text>("hello")     # Prints "hello"
```

### Example 2: Comparable Trait

```glimmer
aspect Comparable then
    chant compare(self, other: Self) -> Number
end

embody Comparable for Number then
    chant compare(self, other: Number) -> Number then
        should self less than other then
            yield -1
        otherwise then
            should self greater than other then
                yield 1
            otherwise then
                yield 0
            end
        end
    end
end

chant find_max<T: Comparable>(a: T, b: T) -> T then
    bind cmp to a.compare(b)
    should cmp greater than or equal to 0 then
        yield a
    otherwise then
        yield b
    end
end
```

### Example 3: Container Trait

```glimmer
aspect Container<T> then
    chant add(self, item: T)
    chant size(self) -> Number
    chant get(self, index: Number) -> Maybe<T>
end

# Implement for List
embody Container<T> for List<T> then
    chant add(self, item: T) then
        bind new_list to list_append(self, item)
        yield new_list
    end

    chant size(self) -> Number then
        yield list_length(self)
    end

    chant get(self, index: Number) -> Maybe<T> then
        should index less than list_length(self) then
            yield Present(list_get(self, index))
        otherwise then
            yield Absent
        end
    end
end
```

### Example 4: Iterator Trait (Foundation for glimmer-weave-7n0)

```glimmer
aspect Iterator<T> then
    chant next(self) -> Maybe<T>
end

aspect Iterable<T> then
    chant iter(self) -> Iterator<T>
end

# Generic iterator combinators
chant map<T, U>(iter: Iterator<T>, func: Function<T -> U>) -> Iterator<U> then
    # Implementation using iterator protocol
    yield mapped_iterator
end

chant filter<T>(iter: Iterator<T>, predicate: Function<T -> Truth>) -> Iterator<T> then
    # Implementation using iterator protocol
    yield filtered_iterator
end
```

### Example 5: Clone Trait

```glimmer
aspect Clone then
    chant clone(self) -> Self
end

embody Clone for Number then
    chant clone(self) -> Number then
        yield self  # Numbers are copied
    end
end

embody Clone for List<T: Clone> then
    chant clone(self) -> List<T> then
        weave result as []
        for each item in self then
            bind cloned to item.clone()
            set result to list_append(result, cloned)
        end
        yield result
    end
end
```

## Implementation Phases

### Phase 1: AST and Parser

**Goal**: Add AST nodes and parse trait definitions and implementations

**AST Changes** (~100 lines):
```rust
// Add to AstNode enum:
AspectDef {
    name: String,
    type_params: Vec<String>,
    methods: Vec<TraitMethod>,
}

EmbodyStmt {
    aspect_name: String,
    type_args: Vec<TypeAnnotation>,  // For generic traits
    target_type: TypeAnnotation,
    methods: Vec<AstNode>,  // ChantDef nodes
}

// New struct:
struct TraitMethod {
    name: String,
    params: Vec<Parameter>,  // First must be 'self'
    return_type: Option<TypeAnnotation>,
}
```

**Token Changes** (~10 lines):
- Add `Aspect` keyword
- Add `Embody` keyword

**Parser Changes** (~200 lines):
- Parse `aspect Name then ... end`
- Parse `embody Trait for Type then ... end`
- Parse trait bounds in generic parameters: `<T: Display>`
- Parse multiple bounds: `<T: Display + Clone>`

**Estimated**: ~310 lines

### Phase 2: Semantic Analysis

**Goal**: Type-check trait definitions, implementations, and bounds

**Changes** (~300 lines):
- Track trait definitions in semantic analyzer
- Track trait implementations (trait → type → method mapping)
- Verify trait implementations are complete
- Verify method signatures match trait definition
- Check trait bounds on generic functions
- Resolve trait methods in method calls
- Verify self parameter is correct type
- Check for duplicate implementations

**Type System Integration**:
```rust
// In semantic.rs Type enum:
Type::TraitBound {
    name: String,
    type_args: Vec<Type>,
}

// Add trait registry:
struct TraitRegistry {
    traits: BTreeMap<String, TraitInfo>,
    implementations: BTreeMap<(String, Type), ImplInfo>,
}
```

**Estimated**: ~300 lines

### Phase 3: Evaluator/Runtime

**Goal**: Execute trait methods at runtime

**Changes** (~150 lines):
- Store trait implementations in evaluator environment
- Resolve trait method calls at runtime
- Handle self parameter binding
- Support calling trait methods through trait bounds

**Runtime Representation**:
```rust
// In eval.rs:
struct TraitImpl {
    aspect_name: String,
    target_type: Type,
    methods: BTreeMap<String, Function>,
}
```

**Method Call Resolution**:
1. Check if type has direct method
2. Search trait implementations for type
3. Call appropriate method with self bound

**Estimated**: ~150 lines

### Phase 4: Tests

**Goal**: Comprehensive test coverage

**Test Cases** (~800 lines, 25+ tests):

**Basic Trait Definition and Implementation**:
- Define simple trait with one method
- Implement trait for primitive type
- Call trait method on value
- Trait with multiple methods

**Generic Traits**:
- Define trait with type parameters
- Implement generic trait for specific type
- Implement generic trait with generic type
- Call methods on generic trait impl

**Trait Bounds**:
- Generic function with single trait bound
- Generic function with multiple trait bounds
- Call generic function with valid type
- Error: call generic function with type missing trait

**Method Resolution**:
- Call trait method on implementing type
- Multiple traits with different methods
- Error: ambiguous method call (same name in multiple traits)
- Error: method not found

**Semantic Errors**:
- Error: incomplete trait implementation
- Error: method signature mismatch
- Error: trait method without self parameter
- Error: duplicate trait implementation
- Error: trait bound on non-existent trait

**Integration Tests**:
- Trait with generic struct
- Trait with enum variants
- Trait in pattern matching
- Nested trait bounds
- Trait with Outcome/Maybe return types

**Complex Scenarios**:
- Multiple traits with shared behavior
- Trait implementation calling another trait method
- Generic function returning trait-constrained type
- Chained trait method calls

**Estimated**: ~800 lines

## Design Decisions

### Why "aspect" and "embody"?

- **aspect**: Conveys that traits define one aspect of a type's behavior
- **embody**: Natural language expressing that a type embodies/fulfills an aspect
- Both fit Glimmer-Weave's natural, readable syntax style

### Why Require `self` Parameter?

- Makes trait methods explicitly method-like
- Clarifies that trait defines instance behavior
- Matches Rust's explicit self (vs Python's implicit self)
- Enables future static methods (no self) vs instance methods (self)

### Why No Default Implementations (Yet)?

**Phase 1 Simplicity**:
- Default implementations add complexity
- Want to get basic trait system working first
- Can add in Phase 2 as enhancement

**Future Enhancement**:
```glimmer
aspect Display then
    chant show(self) -> Text

    chant show_debug(self) -> Text then
        # Default implementation
        yield "[" + self.show() + "]"
    end
end
```

### Associated Types vs Generic Parameters?

**Starting with Generic Parameters**:
- More flexible for initial implementation
- Easier to understand
- Matches current generic system

**Future Associated Types**:
```glimmer
aspect Iterator then
    type Item  # Associated type
    chant next(self) -> Maybe<Self.Item>
end
```

### Static vs Dynamic Dispatch?

**Phase 1: Static Dispatch Only**:
- Monomorphization-based (like current generics)
- No runtime overhead
- All trait bounds resolved at compile time

**Future Dynamic Dispatch**:
```glimmer
# Trait object syntax (future)
bind displayer: aspect Display to get_some_displayable()
reveal(displayer.show())  # Dynamic dispatch
```

## Integration with Existing Features

### With Generics (glimmer-weave-bdw) ✅

Traits build on the existing generic system:
```glimmer
chant identity<T>(x: T) -> T then yield x end

# Now with trait bounds:
chant print_identity<T: Display>(x: T) -> T then
    reveal(x.show())
    yield x
end
```

### With Iterator System (glimmer-weave-7n0)

Traits are **required** for proper iterator implementation:
```glimmer
aspect Iterator<T> then
    chant next(self) -> Maybe<T>
end

chant map<T, U>(iter: Iterator<T>, f: Function<T -> U>) -> Iterator<U> then
    # ...
end
```

### With Outcome and Maybe

Trait methods can return Outcome/Maybe:
```glimmer
aspect Parseable then
    chant parse(text: Text) -> Outcome<Self, Text>
end

embody Parseable for Number then
    chant parse(text: Text) -> Outcome<Number, Text> then
        # Parse implementation
        should success then
            yield Triumph(parsed_number)
        otherwise then
            yield Mishap("Parse error")
        end
    end
end
```

### With Pattern Matching

Trait bounds in match expressions:
```glimmer
chant process<T: Display>(value: Maybe<T>) then
    match value with
        when Present(v) then reveal(v.show())
        when Absent then reveal("No value")
    end
end
```

### With Structs and Enums

Implement traits for user-defined types:
```glimmer
form Point with x as Number y as Number end

embody Display for Point then
    chant show(self) -> Text then
        yield "(" + to_text(self.x) + ", " + to_text(self.y) + ")"
    end
end
```

## Error Messages

Natural language error messages:

```
Cannot find aspect 'Display'
Did you mean: 'Displayable'?

Type 'Number' does not embody aspect 'Comparable'
Required by trait bound on parameter 'T' in function 'sort'

Incomplete embodiment of aspect 'Display' for type 'Point'
Missing method: show(self) -> Text

Method signature mismatch in embodiment of aspect 'Display' for type 'Point'
Expected: show(self) -> Text
Found:    show(self) -> Number

Trait method 'show' must have 'self' as first parameter

Ambiguous method call: 'show'
Multiple aspects provide this method:
  - Display
  - Debug
Specify which aspect to use
```

## Implementation Notes

### Trait Registry Structure

```rust
struct SemanticAnalyzer {
    // Existing fields...

    trait_defs: BTreeMap<String, TraitDefinition>,
    trait_impls: BTreeMap<TraitImplKey, TraitImplementation>,
}

struct TraitDefinition {
    name: String,
    type_params: Vec<String>,
    methods: Vec<TraitMethodSignature>,
}

struct TraitImplementation {
    aspect_name: String,
    type_args: Vec<Type>,
    target_type: Type,
    method_bodies: BTreeMap<String, AstNode>,
}

#[derive(Hash, Eq, PartialEq)]
struct TraitImplKey {
    aspect_name: String,
    target_type: Type,  // Normalized
}
```

### Method Resolution Algorithm

```
When resolving method call `value.method(args)`:

1. Get type of `value`
2. Check if type has inherent method `method`
3. If not, search trait implementations:
   a. Find all traits implemented for type
   b. Filter traits that have method `method`
   c. If exactly one: use it
   d. If zero: error "method not found"
   e. If multiple: error "ambiguous method"
4. Bind `self` to `value`
5. Call method with `args`
```

### Type Checking Trait Bounds

```
When checking generic function call `f<T>(value)` where `f<T: Trait>`:

1. Infer or extract type argument T
2. Get trait bound `Trait` from function definition
3. Check if T implements Trait:
   a. Look up (Trait, T) in trait_impls
   b. If found: verify all methods implemented
   c. If not found: error "type does not implement trait"
4. Proceed with normal type checking
```

## Future Enhancements

### Phase 2 Features

1. **Default Method Implementations**: Provide default bodies in trait definitions
2. **Associated Types**: Type members in traits (e.g., `Iterator::Item`)
3. **Supertraits**: Trait inheritance (e.g., `aspect Ord: Eq`)
4. **Derivable Traits**: Automatic implementation (e.g., `#[derive(Clone)]`)

### Phase 3 Features

1. **Trait Objects**: Dynamic dispatch through `aspect Type` syntax
2. **Trait Aliases**: Type aliases for complex trait bounds
3. **Higher-Ranked Trait Bounds**: For-all quantification (advanced)
4. **Const Generics in Traits**: Generic over constants

---

*Issue: glimmer-weave-3ub [P1]*
*Status: 🚧 IN PROGRESS - Design Phase*
*Dependencies: glimmer-weave-bdw (Generics) ✅ Complete*
*Unblocks: glimmer-weave-7n0 (Iterator trait)*
*Estimated Implementation: ~1560 lines (310 AST + 300 Semantic + 150 Runtime + 800 Tests)*
