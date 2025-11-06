# Loop Control Flow - Design Document

## Overview

Add `break` and `continue` statements to Glimmer-Weave for loop control flow, following the language's natural, readable syntax philosophy.

## Design Philosophy

**Natural Language First**: Loop control should read like natural commands.

### Keyword Choice

After considering alternatives (`exit`, `skip`, `halt`, `next`), we stick with **`break`** and **`continue`** as they are:
- ✅ Universally understood in programming
- ✅ Clear and unambiguous
- ✅ Single-word commands (natural)
- ✅ Already familiar to most programmers

## Syntax

### Break Statement

Exits the innermost enclosing loop immediately:

```glimmer
whilst true then
    should condition then
        break
    end
end
```

### Continue Statement

Skips to the next iteration of the innermost enclosing loop:

```glimmer
for each item in items then
    should skip_this(item) then
        continue
    end
    process(item)
end
```

## Behavior

### Break

- Exits the **innermost** enclosing loop (`for each` or `whilst`)
- Control flow resumes after the loop's `end`
- Using `break` outside a loop is a compile/runtime error

### Continue

- Skips to the next iteration of the **innermost** enclosing loop
- In `for each` loops: moves to next element
- In `whilst` loops: re-evaluates the condition
- Using `continue` outside a loop is a compile/runtime error

### Nested Loops

`break` and `continue` only affect the innermost loop:

```glimmer
for each outer in outer_list then
    for each inner in inner_list then
        should condition then
            break  # Only breaks inner loop
        end
    end
end
```

## Implementation Phases

### Phase 1: AST and Parser ✅ COMPLETE

**Goal**: Add AST nodes and parse break/continue keywords

**Changes:**
- ✅ Add `Break` and `Continue` AST nodes
- ✅ Add `break` and `continue` to lexer keywords
- ✅ Parse statements in loop bodies

**Actual**: ~50 lines (as estimated)

### Phase 2: Evaluator ✅ COMPLETE

**Goal**: Implement control flow semantics

**Approach Used**: Custom Error Type (Option 2)

**Implementation**:
```rust
RuntimeError::BreakOutsideLoop
RuntimeError::ContinueOutsideLoop
```

**Changes:**
- ✅ Modified loop evaluation to catch break/continue errors
- ✅ Converted errors to Rust's break/continue at loop boundaries
- ✅ Added error messages to error_type() and error_value() methods

**Actual**: ~100 lines (as estimated)

### Phase 3: Tests ✅ COMPLETE

**Goal**: Comprehensive test coverage

**Test Cases Implemented (22 tests):**
- ✅ Break in `for each` loop (4 tests)
- ✅ Break in `whilst` loop (3 tests)
- ✅ Continue in `for each` loop (3 tests)
- ✅ Continue in `whilst` loop (2 tests)
- ✅ Nested loops with break/continue (3 tests)
- ✅ Break/continue outside loop (4 tests)
- ✅ Complex scenarios with pattern matching (3 tests)

**Actual**: ~560 lines, 22 tests (exceeded estimate - very thorough coverage)

## Examples

### Example 1: Find First Match

```glimmer
chant find_first(list, predicate) then
    for each item in list then
        should predicate(item) then
            yield item
        end
    end
    yield Absent
end
```

With break:
```glimmer
chant find_first(list, predicate) then
    weave result as Absent

    for each item in list then
        should predicate(item) then
            set result to Present(item)
            break
        end
    end

    yield result
end
```

### Example 2: Skip Invalid Items

```glimmer
for each item in items then
    should not is_valid(item) then
        continue
    end

    process(item)
end
```

### Example 3: Early Loop Exit

```glimmer
whilst true then
    bind input to get_user_input()

    should input is "quit" then
        display("Goodbye!")
        break
    end

    process(input)
end
```

### Example 4: Nested Loop with Break

```glimmer
weave found as false

for each row in matrix then
    for each col in row then
        should col is target then
            set found to true
            break  # Breaks inner loop only
        end
    end

    should found then
        break  # Break outer loop
    end
end
```

### Example 5: Continue in Whilst Loop

```glimmer
weave count as 0

whilst count less than 10 then
    set count to count + 1

    should count mod 2 is 0 then
        continue  # Skip even numbers
    end

    display(count)  # Only prints odd numbers
end
```

## Error Messages

Natural language error messages:

```
Cannot use 'break' outside of a loop
Expected: break statement inside 'for each' or 'whilst' loop

Cannot use 'continue' outside of a loop
Expected: continue statement inside 'for each' or 'whilst' loop
```

## Integration with Existing Features

### With Pattern Matching

```glimmer
for each item in items then
    match item with
        when Skip then continue
        when Stop then break
        when Process(data) then handle(data)
    end
end
```

### With Outcome/Maybe

```glimmer
for each result in results then
    match result with
        when Mishap(err) then
            display("Error: " + err)
            continue
        when Triumph(val) then
            process(val)
    end
end
```

### With Error Propagation

```glimmer
for each file in files then
    bind contents to read_file(file)?

    should is_empty(contents) then
        continue
    end

    process(contents)
end
```

## Runtime Representation

### Control Flow Signal

```rust
// In evaluator
enum ControlFlow {
    None,
    Break,
    Continue,
    Return(Value),
}

// Loop evaluation returns (Value, ControlFlow)
fn eval_loop(&mut self, body: &[AstNode]) -> Result<(Value, ControlFlow), RuntimeError> {
    // ...
}
```

### Loop Context

Track whether we're inside a loop to validate break/continue:

```rust
struct Evaluator {
    // ...
    loop_depth: usize,
}
```

## Test Plan

### Phase 1 Tests (Parsing)
- Parse `break` statement
- Parse `continue` statement
- Parse in `for each` loop body
- Parse in `whilst` loop body
- Parse in nested loops

### Phase 2 Tests (Evaluation)
- Break exits `for each` loop
- Break exits `whilst` loop
- Continue skips to next iteration in `for each`
- Continue re-evaluates condition in `whilst`
- Break only affects innermost loop
- Continue only affects innermost loop
- Error when break outside loop
- Error when continue outside loop

### Phase 3 Tests (Integration)
- Break with pattern matching
- Continue with conditionals
- Break/continue with mutable variables
- Complex nested scenarios

---

*Issue: glimmer-weave-ecp [P1]*
*Status: ✅ COMPLETE*
*Implementation Date: 2025-11-06*
*Total Lines Added: ~710 lines (exceeded estimate due to comprehensive testing)*
*Test Coverage: 22 tests, all passing*

## Summary

All three phases of the break/continue implementation are complete:
- Phase 1: AST and Parser (~50 lines)
- Phase 2: Evaluator Control Flow (~100 lines)
- Phase 3: Comprehensive Tests (~560 lines, 22 tests)

The implementation uses Rust's error mechanism for control flow, where `RuntimeError::BreakOutsideLoop` and `RuntimeError::ContinueOutsideLoop` are thrown by break/continue statements and caught at loop boundaries, where they're converted into actual control flow (Rust's break/continue). This approach elegantly handles the "outside loop" error case while keeping the implementation simple.
