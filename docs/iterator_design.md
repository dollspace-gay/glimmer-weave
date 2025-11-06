# Iterator System - Design Document

## Overview

Implement Iterator as a first-class abstraction enabling functional programming patterns through lazy evaluation, combinators, and efficient iteration over collections.

## Design Philosophy

**Lazy Evaluation**: Iterators compute values on-demand rather than eagerly materializing entire collections, enabling efficient composition and infinite sequences.

**Zero-Cost Abstractions**: Iterator combinators should compile to efficient loops with no runtime overhead compared to manual iteration.

**Trait-Based**: Leverages the trait system (glimmer-weave-3ub) to provide a unified interface for iteration.

## Core Concepts

### Iterator Protocol

An iterator is any type implementing the `Iterator<T>` trait with a `next(self) -> Maybe<T>` method:
- Returns `Present(value)` when there are more elements
- Returns `Absent` when iteration is complete
- Mutates internal state to track position

### Iterable Protocol

An iterable is any type that can produce an iterator via `Iterable<T>` trait's `iter(self) -> Iterator<T>` method.

### Stateful Iteration

Iterators maintain mutable state (current position, remaining elements, etc.) and are consumed as they progress.

## Syntax Design

### Core Traits

```glimmer
# Iterator trait - anything that produces a sequence of values
aspect Iterator<T> then
    chant next(self) -> Maybe<T>
end

# Iterable trait - anything that can be iterated over
aspect Iterable<T> then
    chant iter(self) -> Iterator<T>
end
```

### Implementing Iterator for Built-in Types

```glimmer
# Lists are iterable
embody Iterable<T> for List<T> then
    chant iter(self) -> Iterator<T> then
        yield ListIterator { list: self, index: 0 }
    end
end

# Ranges are iterable
embody Iterable<Number> for Range then
    chant iter(self) -> Iterator<Number> then
        yield RangeIterator { current: self.start, end: self.end }
    end
end
```

### Using Iterators

```glimmer
# Manual iteration
bind iter to [1, 2, 3].iter()
bind first to iter.next()  # Present(1)
bind second to iter.next() # Present(2)

# With combinators
bind nums to [1, 2, 3, 4, 5]
bind result to nums.iter()
    | map(fn(x) then x * 2 end)
    | filter(fn(x) then x greater than 5 end)
    | collect()

# result is [6, 8, 10]
```

## Iterator Combinators

### Transformation Combinators

**map** - Transform each element
```glimmer
chant map<T, U>(iter: Iterator<T>, func: Function<T -> U>) -> Iterator<U> then
    yield MapIterator { inner: iter, func: func }
end

# Usage
bind doubled to [1, 2, 3].iter().map(fn(x) then x * 2 end)
```

**filter** - Keep only matching elements
```glimmer
chant filter<T>(iter: Iterator<T>, predicate: Function<T -> Truth>) -> Iterator<T> then
    yield FilterIterator { inner: iter, predicate: predicate }
end

# Usage
bind evens to [1, 2, 3, 4].iter().filter(fn(x) then x % 2 is 0 end)
```

**flat_map** - Transform and flatten nested iterators
```glimmer
chant flat_map<T, U>(iter: Iterator<T>, func: Function<T -> Iterator<U>>) -> Iterator<U>
```

### Reduction Combinators

**fold** - Accumulate values into single result
```glimmer
chant fold<T, Acc>(iter: Iterator<T>, init: Acc, func: Function<(Acc, T) -> Acc>) -> Acc then
    weave acc as init
    whilst true then
        match iter.next() with
            when Present(value) then
                set acc to func(acc, value)
            when Absent then
                yield acc
        end
    end
end

# Usage: sum
bind sum to [1, 2, 3].iter().fold(0, fn(acc, x) then acc + x end)
```

**reduce** - Like fold but uses first element as initial value
```glimmer
chant reduce<T>(iter: Iterator<T>, func: Function<(T, T) -> T>) -> Maybe<T>
```

**collect** - Materialize iterator into a list
```glimmer
chant collect<T>(iter: Iterator<T>) -> List<T> then
    weave result as []
    whilst true then
        match iter.next() with
            when Present(value) then
                set result to list_append(result, value)
            when Absent then
                yield result
        end
    end
end
```

### Limiting Combinators

**take** - Take first N elements
```glimmer
chant take<T>(iter: Iterator<T>, n: Number) -> Iterator<T> then
    yield TakeIterator { inner: iter, remaining: n }
end
```

**skip** - Skip first N elements
```glimmer
chant skip<T>(iter: Iterator<T>, n: Number) -> Iterator<T> then
    # Skip n elements
    weave count as 0
    whilst count less than n then
        match iter.next() with
            when Present(_) then set count to count + 1
            when Absent then yield EmptyIterator {}
        end
    end
    yield iter
end
```

**take_while** - Take while predicate is true
```glimmer
chant take_while<T>(iter: Iterator<T>, predicate: Function<T -> Truth>) -> Iterator<T>
```

### Combining Combinators

**zip** - Combine two iterators element-wise
```glimmer
chant zip<T, U>(iter1: Iterator<T>, iter2: Iterator<U>) -> Iterator<Pair<T, U>> then
    yield ZipIterator { first: iter1, second: iter2 }
end
```

**chain** - Concatenate two iterators
```glimmer
chant chain<T>(iter1: Iterator<T>, iter2: Iterator<T>) -> Iterator<T>
```

### Testing Combinators

**any** - Check if any element satisfies predicate
```glimmer
chant any<T>(iter: Iterator<T>, predicate: Function<T -> Truth>) -> Truth then
    whilst true then
        match iter.next() with
            when Present(value) then
                should predicate(value) then yield true end
            when Absent then yield false
        end
    end
end
```

**all** - Check if all elements satisfy predicate
```glimmer
chant all<T>(iter: Iterator<T>, predicate: Function<T -> Truth>) -> Truth
```

**find** - Find first matching element
```glimmer
chant find<T>(iter: Iterator<T>, predicate: Function<T -> Truth>) -> Maybe<T>
```

## Implementation Strategy

### Phase 1: Core Iterator Infrastructure (~150 lines)

**Goal**: Establish iterator representation and basic iteration

1. Define `Iterator<T>` and `Iterable<T>` traits
2. Create iterator state representation in runtime
3. Implement `iter()` for List and Range
4. Add tests for basic iteration

### Phase 2: Transformation Combinators (~200 lines)

**Goal**: Implement map, filter, flat_map

1. Create combinator iterator types (MapIterator, FilterIterator)
2. Implement combinator functions
3. Add chaining syntax support
4. Write tests for transformations

### Phase 3: Reduction and Collection (~150 lines)

**Goal**: Implement fold, reduce, collect

1. Implement fold and reduce
2. Implement collect to materialize iterators
3. Add count, sum, product helpers
4. Write tests for reductions

### Phase 4: Advanced Combinators (~200 lines)

**Goal**: Implement take, skip, zip, chain, and testing combinators

1. Implement limiting combinators (take, skip, take_while)
2. Implement combining combinators (zip, chain)
3. Implement testing combinators (any, all, find)
4. Write comprehensive tests

### Phase 5: Integration and Performance (~100 lines)

**Goal**: Integration with existing language features

1. Test iterator with pattern matching
2. Test iterator with error propagation (?)
3. Benchmark iterator performance
4. Document usage patterns

## Runtime Representation

### Iterator State Values

Iterators are represented as special runtime values carrying state:

```rust
// In eval.rs Value enum:
pub enum Value {
    // ... existing variants ...

    /// Iterator state - generic iterator over type T
    Iterator {
        iterator_type: String,  // "List", "Range", "Map", "Filter", etc.
        state: Box<IteratorState>,
    },
}

#[derive(Debug, Clone)]
pub enum IteratorState {
    List {
        elements: Vec<Value>,
        index: usize,
    },
    Range {
        current: f64,
        end: f64,
    },
    Map {
        inner: Box<Value>,  // Inner iterator
        func: Box<Value>,   // Function to apply
    },
    Filter {
        inner: Box<Value>,
        predicate: Box<Value>,
    },
    Take {
        inner: Box<Value>,
        remaining: usize,
    },
    // ... more iterator types
}
```

### Iterator Trait Implementation

```rust
// Iterator methods dispatch to appropriate state handlers
fn eval_iterator_next(state: &mut IteratorState) -> Result<Value, RuntimeError> {
    match state {
        IteratorState::List { elements, index } => {
            if *index < elements.len() {
                let value = elements[*index].clone();
                *index += 1;
                Ok(Value::Maybe { present: true, value: Some(Box::new(value)) })
            } else {
                Ok(Value::Maybe { present: false, value: None })
            }
        }
        IteratorState::Map { inner, func } => {
            // Get next from inner iterator
            // Apply function
            // Return result
        }
        // ... handle other iterator types
    }
}
```

## Examples

### Example 1: Simple Iteration

```glimmer
bind nums to [1, 2, 3, 4, 5]
bind iter to nums.iter()

weave sum as 0
whilst true then
    match iter.next() with
        when Present(n) then
            set sum to sum + n
        when Absent then
            break
    end
end

reveal(sum)  # Prints 15
```

### Example 2: Map and Filter

```glimmer
bind nums to [1, 2, 3, 4, 5, 6]
bind result to nums.iter()
    .map(fn(x) then x * 2 end)
    .filter(fn(x) then x greater than 5 end)
    .collect()

reveal(result)  # Prints [6, 8, 10, 12]
```

### Example 3: Fold (Sum)

```glimmer
bind nums to [1, 2, 3, 4, 5]
bind sum to nums.iter().fold(0, fn(acc, x) then acc + x end)

reveal(sum)  # Prints 15
```

### Example 4: Lazy Evaluation

```glimmer
# This creates an iterator but doesn't compute anything yet
bind lazy to range(1, 1000000)
    .map(fn(x) then x * x end)
    .filter(fn(x) then x % 2 is 0 end)
    .take(5)

# Only when we collect do we actually compute (and only 5 elements!)
bind result to lazy.collect()

reveal(result)  # Prints first 5 even squares
```

### Example 5: Complex Pipeline

```glimmer
form Person with name as Text age as Number end

bind people to [
    Person { name: "Alice", age: 30 },
    Person { name: "Bob", age: 25 },
    Person { name: "Charlie", age: 35 }
]

bind adult_names to people.iter()
    .filter(fn(p) then p.age at least 30 end)
    .map(fn(p) then p.name end)
    .collect()

reveal(adult_names)  # Prints ["Alice", "Charlie"]
```

### Example 6: Zip

```glimmer
bind names to ["Alice", "Bob", "Charlie"]
bind ages to [30, 25, 35]

bind pairs to zip(names.iter(), ages.iter()).collect()

# pairs is [("Alice", 30), ("Bob", 25), ("Charlie", 35)]
```

## Integration with Existing Features

### With Pattern Matching

```glimmer
bind nums to [1, 2, 3, 4, 5]
bind iter to nums.iter()

match iter.next() with
    when Present(first) then reveal(first)
    when Absent then reveal("Empty!")
end
```

### With Error Propagation

```glimmer
chant parse_numbers(lines: List<Text>) -> Outcome<List<Number>, Text> then
    bind results to lines.iter()
        .map(fn(line) then parse_number(line)? end)
        .collect()

    yield Triumph(results)
end
```

### With For-Each Loops

```glimmer
# Existing for-each syntax could desugar to iterator protocol
for each x in [1, 2, 3] then
    reveal(x)
end

# Equivalent to:
bind iter to [1, 2, 3].iter()
whilst true then
    match iter.next() with
        when Present(x) then reveal(x)
        when Absent then break
    end
end
```

## Performance Considerations

### Optimization Opportunities

1. **Iterator Fusion**: Combine adjacent map/filter operations into single pass
2. **Inline Small Functions**: Inline lambda functions in tight loops
3. **Avoid Allocations**: Reuse iterator state where possible
4. **Specialize Common Patterns**: Optimize sum, count, etc.

### Benchmarks

Track performance for common operations:
- Simple iteration (vs manual loops)
- Map + filter chains
- Fold operations
- Collection to list

## Error Messages

```
Cannot call 'next' on non-iterator value
Expected Iterator<T>, got List

Iterator exhausted
Attempted to call 'next' on consumed iterator

Type mismatch in map function
Expected function (Number -> Text), got (Number -> Number)

Iterator trait not implemented for type 'Person'
Consider implementing Iterable<T> for Person
```

## Future Enhancements

### Parallel Iterators

```glimmer
bind result to large_list.par_iter()
    .map(expensive_computation)
    .collect()
```

### Async Iterators (Streams)

```glimmer
aspect AsyncIterator<T> then
    chant next(self) -> Future<Maybe<T>>
end
```

### Custom Iterator Implementations

```glimmer
form Fibonacci with a as Number b as Number end

embody Iterator<Number> for Fibonacci then
    chant next(self) -> Maybe<Number> then
        bind result to self.a
        bind next_a to self.b
        bind next_b to self.a + self.b
        set self.a to next_a
        set self.b to next_b
        yield Present(result)
    end
end
```

---

*Issue: glimmer-weave-7n0 [P1]*
*Status: 🚧 IN PROGRESS - Design Phase*
*Dependencies: glimmer-weave-bdw (Generics) ✅, glimmer-weave-3ub (Traits) ✅*
*Estimated Implementation: ~800 lines across 5 phases*
