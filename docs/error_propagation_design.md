# Error Propagation Operator - Design Document

## Overview

Add the `?` operator to Glimmer-Weave for automatic error propagation, following Rust's `?` operator pattern. This reduces boilerplate when working with `Outcome<T, E>` types.

## Design Philosophy

**Ergonomic Error Handling**: The `?` operator makes error handling natural and concise, eliminating the need for repetitive match statements.

## Current State (Without `?`)

```glimmer
chant read_and_process_file(path) then
    bind file_contents to read_file(path)

    match file_contents with
        when Mishap(err) then yield Mishap(err)
        when Triumph(contents) then
            bind parsed to parse_json(contents)

            match parsed with
                when Mishap(err) then yield Mishap(err)
                when Triumph(data) then
                    yield Triumph(process(data))
            end
    end
end
```

## With `?` Operator

```glimmer
chant read_and_process_file(path) then
    bind contents to read_file(path)?
    bind data to parse_json(contents)?
    yield Triumph(process(data))
end
```

## Syntax

### Basic Usage

```glimmer
bind value to risky_operation()?
```

### In Function Calls

```glimmer
bind result to transform(get_data()?)
```

### Chained Operations

```glimmer
bind final to step1()? + step2()? + step3()?
```

## Semantics

The `?` operator:

1. **Checks the type**: Must be applied to an `Outcome<T, E>` value
2. **On Mishap**: Returns `Mishap(error)` from the current function immediately (early return)
3. **On Triumph**: Unwraps the value and continues execution
4. **Context requirement**: Can only be used inside functions that return `Outcome`

## Behavior

### Success Case

```glimmer
chant example() then
    bind x to Triumph(42)?  # x = 42
    yield Triumph(x * 2)     # Returns Triumph(84)
end
```

### Error Case

```glimmer
chant example() then
    bind x to Mishap("error")?  # Early return: Mishap("error")
    yield Triumph(x * 2)         # Never executes
end
```

### Propagation Chain

```glimmer
chant process_chain() then
    bind a to step1()?  # If Mishap, returns immediately
    bind b to step2(a)? # If Mishap, returns immediately
    bind c to step3(b)? # If Mishap, returns immediately
    yield Triumph(c)
end
```

## Implementation Phases

### Phase 1: AST and Parser

**Goal**: Add AST node and parse `?` as postfix operator

**Changes:**
- Add `Try` postfix operator to AST (e.g., `PostfixOp::Try` or `AstNode::Try`)
- Add `Question` token to lexer
- Parse `?` as a postfix operator with high precedence

**Estimated**: ~60 lines

### Phase 2: Evaluator

**Goal**: Implement error propagation semantics

**Approach**: Check if value is Outcome, handle Mishap/Triumph

**Changes:**
- In evaluator, handle `Try` operator
- Check if value is `Outcome`
- On `Mishap`: return the error immediately (using existing Return mechanism)
- On `Triumph`: unwrap and continue
- Track whether we're in a function context

**Estimated**: ~80 lines

### Phase 3: Semantic Analysis

**Goal**: Type-check try operator usage

**Changes:**
- Verify `?` is applied to `Outcome` type
- Verify `?` is used inside function that returns `Outcome`
- Add helpful error messages for misuse

**Estimated**: ~50 lines

### Phase 4: Tests

**Goal**: Comprehensive test coverage

**Test Cases:**
- Basic `?` on Triumph
- Basic `?` on Mishap (early return)
- Chained `?` operations
- `?` in expressions
- `?` with different error types
- Error: `?` on non-Outcome value
- Error: `?` outside function
- Error: `?` in function not returning Outcome
- Integration with pattern matching
- Integration with nested function calls

**Estimated**: ~300 lines (12-15 tests)

## Examples

### Example 1: File Processing

```glimmer
chant process_config_file(path) then
    bind contents to read_file(path)?
    bind config to parse_json(contents)?
    bind validated to validate_config(config)?
    yield Triumph(validated)
end
```

Without `?`:
```glimmer
chant process_config_file(path) then
    match read_file(path) with
        when Mishap(e) then yield Mishap(e)
        when Triumph(contents) then
            match parse_json(contents) with
                when Mishap(e) then yield Mishap(e)
                when Triumph(config) then
                    match validate_config(config) with
                        when Mishap(e) then yield Mishap(e)
                        when Triumph(validated) then
                            yield Triumph(validated)
                    end
            end
    end
end
```

### Example 2: Database Operations

```glimmer
chant get_user_email(user_id) then
    bind db to connect_to_database()?
    bind user to db.find_user(user_id)?
    bind email to user.get_email()?
    yield Triumph(email)
end
```

### Example 3: Mathematical Operations

```glimmer
chant safe_divide_and_add(a, b, c, d) then
    bind quotient1 to divide(a, b)?
    bind quotient2 to divide(c, d)?
    yield Triumph(quotient1 + quotient2)
end
```

### Example 4: Nested Function Calls

```glimmer
chant transform_data(input) then
    bind result to process(parse(input)?)
    yield Triumph(result)
end
```

### Example 5: Conditional with `?`

```glimmer
chant conditional_process(flag, data) then
    should flag then
        bind processed to risky_operation(data)?
        yield Triumph(processed)
    otherwise
        yield Triumph(data)
    end
end
```

## Error Messages

Natural language error messages:

```
Cannot use '?' operator on type 'Number'
Expected: Outcome<T, E>

Cannot use '?' operator outside of a function

Function must return Outcome to use '?' operator
Current return type: Number
```

## Integration with Existing Features

### With Pattern Matching

```glimmer
chant process_with_match(data) then
    bind result to get_outcome()?

    match result with
        when Some(value) then yield Triumph(value * 2)
        when None then yield Mishap("No value")
    end
end
```

### With Nested Functions

```glimmer
chant outer() then
    chant inner() then
        bind x to risky()?
        yield Triumph(x)
    end

    bind result to inner()?
    yield Triumph(result + 1)
end
```

### With Loops

```glimmer
chant process_list(items) then
    weave results as []

    for each item in items then
        bind processed to process_item(item)?
        set results to list_append(results, processed)
    end

    yield Triumph(results)
end
```

## Implementation Notes

### Precedence

The `?` operator should have high precedence (postfix operators typically do):
- Higher than binary operators: `a + b?` means `a + (b?)`
- Lower than field access: `obj.field?` means `(obj.field)?`
- Can be chained: `func()?.field` (if we add optional chaining later)

### Return Mechanism

We can reuse the existing `RuntimeError::Return` mechanism:
- When `?` encounters a `Mishap(e)`, throw `RuntimeError::Return(Mishap(e))`
- This will propagate up to the function boundary
- The function will return the Mishap value

### Type System Integration

The semantic analyzer should:
1. Check that `?` is applied to `Outcome<T, E>` type
2. Infer that the unwrapped type is `T`
3. Verify the enclosing function returns `Outcome<_, E>` (same error type)

## Alternative Syntax Considered

- **`try`**: Too verbose, keyword-heavy
- **`!`**: Confusing, looks like negation
- **`.unwrap_or_return()`**: Too verbose, not operator-like
- **`?` (chosen)**: Concise, familiar to Rust programmers, visually distinct

## Design Decisions

### Why Postfix?

Postfix `?` is more readable than prefix:
- `read_file(path)?` - clear that `?` applies to the result
- `? read_file(path)` - ambiguous, looks like ternary

### Why Only Outcome?

Currently limiting to `Outcome` types:
- ✅ Most common use case (error handling)
- ✅ Clear semantics
- ✅ Easier to implement initially
- ⚠️ Future: Could extend to `Maybe` types with similar semantics

### Function Boundary Only?

The `?` operator only works in functions because:
- Clear boundary for early return
- Matches Rust's semantics
- Prevents confusion about where errors propagate to

## Future Enhancements

Potential future additions:
1. **Maybe support**: `get_optional()?` returns Absent early
2. **Custom conversions**: Allow user types to define `?` behavior
3. **Try blocks**: Rust-like `try { ... }` expressions
4. **Error conversion**: Automatic `.into()` for error type conversion

---

*Issue: glimmer-weave-t9b [P1]*
*Status: ✅ COMPLETE*
*Implementation Date: 2025-11-06*
*Total Lines Added: ~670 lines*
*Test Coverage: 15 tests, all passing*

## Summary

All four phases of the error propagation operator are complete:
- Phase 1: AST and Parser (~60 lines)
- Phase 2: Evaluator (~25 lines)
- Phase 3: Semantic Analysis (~7 lines)
- Phase 4: Comprehensive Tests (~575 lines, 15 tests)

The `?` operator is now fully functional and allows for clean, ergonomic error propagation in Glimmer-Weave. It integrates seamlessly with the existing Outcome type system and works in all contexts including nested functions, loops, and conditionals.
