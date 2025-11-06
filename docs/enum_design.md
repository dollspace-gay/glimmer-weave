# User-Defined Enum Types - Design Document

## Overview

Glimmer-Weave's enum types enable algebraic data types for domain modeling, following the language's natural, readable syntax philosophy.

## Design Philosophy

**Natural Language First**: Enums should read like natural descriptions of possibilities.

### Keyword Choice: "variant"

After considering alternatives (`form`, `choice`, `enum`), **`variant`** best captures the concept in natural language:

```glimmer
variant Color then
    Red
    Green
    Blue
end
```

Reads as: "Variant Color, then Red, Green, Blue"

## Syntax

### Simple Enums (Unit Variants)

```glimmer
variant Direction then
    North
    South
    East
    West
end

# Usage
bind heading to North
```

### Enums with Data (Tagged Unions)

```glimmer
variant Message then
    Quit
    Move(x: Number, y: Number)
    Write(text: Text)
    ChangeColor(r: Number, g: Number, b: Number)
end

# Usage
bind msg1 to Quit
bind msg2 to Move(10, 20)
bind msg3 to Write("Hello")
```

### Generic Enums

```glimmer
variant Option<T> then
    Some(value: T)
    None
end

variant Result<T, E> then
    Ok(value: T)
    Err(error: E)
end

# Usage
bind maybe_value to Some<Number>(42)
bind result to Ok<Text, Text>("success")
```

## Pattern Matching

Enums naturally integrate with `match` expressions:

```glimmer
variant TrafficLight then
    Red
    Yellow
    Green
end

bind light to Red

match light with
    when Red then
        display("Stop!")
    when Yellow then
        display("Caution!")
    when Green then
        display("Go!")
end
```

### Matching with Data

```glimmer
variant Shape then
    Circle(radius: Number)
    Rectangle(width: Number, height: Number)
    Triangle(base: Number, height: Number)
end

bind shape to Circle(5.0)

match shape with
    when Circle(r) then
        bind area to 3.14 * r * r
        area
    when Rectangle(w, h) then
        w * h
    when Triangle(b, h) then
        (b * h) / 2
end
```

## Natural Language Helper Methods

Following Outcome/Maybe patterns, enums get natural helper methods:

### Inspection

```glimmer
# Check which variant
is_variant(enum_value, "VariantName") -> Truth

# Example
bind light to Red
is_variant(light, "Red")  # yields true
```

### Extraction

```glimmer
# Get variant data or panic
expect_variant(enum_value, "VariantName", "message") -> data

# Get variant data or default
variant_or(enum_value, "VariantName", default) -> data
```

### Transformation

```glimmer
# Transform if matches variant
refine_variant(enum_value, "VariantName", chant(data) -> result) -> Option<result>
```

## Comparison with Outcome/Maybe

Glimmer-Weave already has built-in enums:

```glimmer
# Built-in (special syntax):
Outcome<T, E> ::= Triumph(T) | Mishap(E)
Maybe<T> ::= Present(T) | Absent

# User-defined (new syntax):
variant Option<T> then
    Some(value: T)
    None
end

variant Result<T, E> then
    Ok(value: T)
    Err(error: E)
end
```

Both follow the same pattern matching and helper method conventions.

## Implementation Phases

### Phase 1: Simple Enums ✅ COMPLETE

**Goal**: Support unit variants (no data)

```glimmer
variant Color then
    Red
    Green
    Blue
end
```

**Changes:**
- ✅ AST: Added `VariantDef` node
- ✅ Parser: Parse `variant ... then ... end`
- ✅ Evaluator: Store enum definitions and create variant constructors
- ✅ Pattern matching: Match variant names

**Tests**: 35 tests (12 parsing + 10 evaluation + 13 pattern matching)
**Lines Added**: ~400 lines

### Phase 2: Enums with Data ✅ COMPLETE

**Goal**: Support variants with associated data

```glimmer
variant Message then
    Quit
    Move(x: Number, y: Number)
    Write(text: Text)
end
```

**Changes:**
- ✅ Value: Added `VariantConstructor` type for callable variant builders
- ✅ Parser: Extended pattern parsing to handle `Ident(pattern, ...)` syntax
- ✅ Evaluator: Create function constructors for data variants
- ✅ Pattern matching: Extract and bind field values

**Tests**: 19 tests (8 construction + 11 pattern matching)
**Lines Added**: ~300 lines

### Phase 3: Generic Enums ✅ COMPLETE

**Goal**: Support type parameters

```glimmer
variant Option<T> then
    Some(value: T)
    Empty
end

bind maybe_num to Some<Number>(42)
bind maybe_text to Some<Text>("hello")
```

**Changes:**
- ✅ Value: Extended `VariantDef`, `VariantValue`, and `VariantConstructor` to store type parameters/arguments
- ✅ Evaluator: Register generic enum definitions with type parameters
- ✅ Parser: Generic syntax already supported (from Phase 1)
- ✅ Call: Extract type arguments from Call nodes and pass to constructors
- ✅ Pattern matching: Works with generic variants (type arguments ignored at runtime)

**Tests**: 9 tests (definition, construction with Number/Text, two type params, in functions, pattern matching, nested, type arg validation)
**Lines Added**: ~100 lines

### Phase 4: Helper Methods ✅ COMPLETE

**Goal**: Natural language helper functions

**Functions:**
- `is_variant(enum_value, "Name") -> Truth` - Check if value matches variant
- `expect_variant(enum_value, "Name", msg) -> List` - Extract fields or panic with message
- `variant_or(enum_value, "Name", default) -> data` - Extract fields or return default
- `refine_variant(enum_value, "Name", fn) -> Maybe<result>` - Transform variant if matches

**Changes:**
- ✅ Runtime: Added four helper functions to builtins
- ✅ is_variant: Returns Truth indicating variant match
- ✅ expect_variant: Returns List of fields or panics with custom error
- ✅ variant_or: Returns List of fields or default value
- ✅ refine_variant: Returns Maybe containing transformed result or Absent

**Tests**: 21 tests (4 is_variant + 5 expect_variant + 4 variant_or + 5 refine_variant + 3 integration)
**Lines Added**: ~150 lines

## Syntax Details

### Full Grammar

```
VariantDef ::= "variant" Identifier TypeParams? "then"
                 VariantCase ("," VariantCase)* ","?
               "end"

TypeParams ::= "<" Identifier ("," Identifier)* ">"

VariantCase ::= Identifier FieldList?

FieldList ::= "(" Field ("," Field)* ")"

Field ::= Identifier ":" TypeAnnotation
```

### Examples

**Simple:**
```glimmer
variant Status then
    Pending,
    Active,
    Completed
end
```

**With Data:**
```glimmer
variant Event then
    Click(x: Number, y: Number),
    KeyPress(key: Text),
    Resize(width: Number, height: Number)
end
```

**Generic:**
```glimmer
variant Either<L, R> then
    Left(value: L),
    Right(value: R)
end
```

**Recursive:**
```glimmer
variant List<T> then
    Cons(head: T, tail: List<T>),
    Nil
end
```

## Type Checking

Enums integrate with the type system:

```glimmer
variant Color then Red, Green, Blue end

# Type: Color
bind my_color to Red

# Type error - not a Color variant
bind invalid to Purple  # Error: Undefined variant 'Purple'
```

### With Type Inference

```glimmer
variant Option<T> then Some(T), None end

bind x to Some(42)
# Type inference: x :: Option<Number>

bind y to Some("hello")
# Type inference: y :: Option<Text>
```

## Runtime Representation

### Simple Enums

Represented as tagged values:

```rust
Value::Variant {
    enum_name: "Color",
    variant_name: "Red",
    data: None,
}
```

### Enums with Data

```rust
Value::Variant {
    enum_name: "Message",
    variant_name: "Move",
    data: Some(vec![Value::Number(10.0), Value::Number(20.0)]),
}
```

## Error Messages

Natural language error messages:

```
Undefined variant 'Purple' for enum 'Color'
Available variants: Red, Green, Blue

Pattern matching on 'Color' is not exhaustive
Missing cases: Green, Blue

Type mismatch in variant constructor:
  Expected: Number
  Got: Text
  In variant: Message::Move
```

## Integration with Existing Features

### With Outcome/Maybe

```glimmer
variant Result<T, E> then
    Ok(value: T)
    Err(error: E)
end

# Convert to Outcome
chant result_to_outcome(result: Result<T, E>) -> Outcome<T, E> then
    match result with
        when Ok(val) then yield Triumph(val)
        when Err(e) then yield Mishap(e)
    end
end
```

### With Pattern Matching

```glimmer
variant Command then
    Quit
    Help
    Run(args: List<Text>)
end

match cmd with
    when Quit then exit()
    when Help then show_help()
    when Run(args) then execute(args)
end
```

### With Type Inference

Type inference automatically deduces enum types:

```glimmer
variant Status then Active, Inactive end

bind s to Active
# Inferred: s :: Status

chant is_active(status) then
    match status with
        when Active then true
        when Inactive then false
    end
end
# Inferred: is_active :: Status -> Truth
```

## Examples

### Traffic Light State Machine

```glimmer
variant TrafficLight then
    Red
    Yellow
    Green
end

chant next_light(current: TrafficLight) -> TrafficLight then
    match current with
        when Red then yield Green
        when Yellow then yield Red
        when Green then yield Yellow
    end
end

bind light to Red
bind next to next_light(light)  # Green
```

### Optional Values

```glimmer
variant Option<T> then
    Some(value: T)
    None
end

chant divide(a: Number, b: Number) -> Option<Number> then
    should b is 0 then
        yield None
    otherwise
        yield Some(a / b)
    end
end

bind result to divide(10, 2)  # Some(5)
bind error to divide(10, 0)   # None
```

### Error Handling

```glimmer
variant FileError then
    NotFound(path: Text)
    PermissionDenied(path: Text)
    IoError(message: Text)
end

variant Result<T, E> then
    Ok(value: T)
    Err(error: E)
end

chant read_file(path: Text) -> Result<Text, FileError> then
    # Implementation
end

bind contents to read_file("data.txt")

match contents with
    when Ok(text) then
        display(text)
    when Err(NotFound(p)) then
        display("File not found: " + p)
    when Err(PermissionDenied(p)) then
        display("Permission denied: " + p)
    when Err(IoError(msg)) then
        display("IO Error: " + msg)
end
```

### Linked List

```glimmer
variant List<T> then
    Cons(head: T, tail: List<T>)
    Nil
end

chant length<T>(list: List<T>) -> Number then
    match list with
        when Nil then yield 0
        when Cons(_, tail) then yield 1 + length(tail)
    end
end

bind my_list to Cons(1, Cons(2, Cons(3, Nil)))
bind len to length(my_list)  # 3
```

## Design Decisions

### Why "variant" over "enum"?

**Considered:**
- `enum` - Standard but technical jargon
- `choice` - Good but ambiguous with conditionals
- `form` - Already used for structs
- `variant` - Clear, natural, reads well

**Decision**: Use `variant` because:
- ✅ Natural language: "variant Color then..."
- ✅ Describes the concept: multiple variants of a type
- ✅ Not overloaded with other meanings
- ✅ Reads smoothly in declarations

### Why "then" ... "end"?

Maintains consistency with existing Glimmer-Weave block syntax:
- `chant ... then ... end`
- `should ... then ... end`
- `variant ... then ... end`

### Comma vs Pipe Separators?

**Option 1: Commas** (chosen)
```glimmer
variant Color then Red, Green, Blue end
```

**Option 2: Pipes**
```glimmer
variant Color then Red | Green | Blue end
```

**Decision**: Commas because:
- ✅ Consistent with parameter lists
- ✅ Less visual noise
- ✅ Clearer for variants with data

### Constructor Syntax?

Use direct constructors like Outcome/Maybe:

```glimmer
bind color to Red
bind msg to Move(10, 20)
```

No `Color::Red` or `Color.Red` needed - keeps it simple and natural.

## Migration Path

Existing Outcome/Maybe code remains unchanged:

```glimmer
# Still works
bind result to Triumph(42)
bind maybe to Present("hello")

# New equivalent enums available
variant Option<T> then Some(T), None end
variant Result<T, E> then Ok(T), Err(E) end
```

Users can gradually adopt user-defined enums alongside built-in types.

## Test Plan

### Phase 1 Tests (Simple Enums)
- Define simple enum
- Construct enum values
- Pattern match on enums
- Exhaustiveness checking

### Phase 2 Tests (With Data)
- Variants with single field
- Variants with multiple fields
- Extract data in patterns
- Type checking variant data

### Phase 3 Tests (Generic)
- Generic enum definition
- Type instantiation
- Type inference with generic enums
- Monomorphization

### Phase 4 Tests (Helpers)
- `is_variant` checks
- `expect_variant` extraction
- `variant_or` defaults
- Error messages

---

*Last Updated: 2025-11-06*
*Issue: glimmer-weave-5nx [P1]*
*Status: 🎉 ALL PHASES COMPLETE (1-4) - 84 tests passing*
*Total Implementation: ~950 lines across all phases*
*Phase 1 (Simple Enums): 35 tests*
*Phase 2 (Enums with Data): 19 tests*
*Phase 3 (Generic Enums): 9 tests*
*Phase 4 (Helper Methods): 21 tests*
