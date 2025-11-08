# Glimmer-Weave vs Rust: Ownership & Borrowing Comparison

This document helps Rust developers understand Glimmer-Weave's ownership system by comparing it to Rust's.

---

## Quick Reference Table

| Concept | Rust | Glimmer-Weave |
|---------|------|---------------|
| Owned value | `let x = vec![1, 2, 3];` | `bind x to [1, 2, 3]` |
| Mutable value | `let mut x = vec![1, 2, 3];` | `weave x as [1, 2, 3]` |
| Shared borrow | `&x` | `borrow x` |
| Mutable borrow | `&mut x` | `borrow mut x` |
| Lifetime annotation | `'a` | `'span` or `'a` |
| Function with lifetime | `fn first<'a>(list: &'a Vec<i32>)` | `chant first<'span>(borrow 'span list as List<Number>)` |
| Clone | `x.clone()` | `x.replicate()` |
| Copy trait | `#[derive(Copy)]` | Automatic for Number, Truth, Nothing |
| Result | `Result<T, E>` | `Outcome<T, E>` |
| Option | `Option<T>` | `Maybe<T>` |
| Ok variant | `Ok(value)` | `Triumph(value)` |
| Err variant | `Err(error)` | `Mishap(error)` |
| Some variant | `Some(value)` | `Present(value)` |
| None variant | `None` | `Absent` |

---

## Side-by-Side Examples

### Example 1: Basic Ownership

**Rust:**
```rust
let data = vec![1, 2, 3];
let moved = data;
// data is now invalid
println!("{:?}", moved);
```

**Glimmer-Weave:**
```glimmer-weave
bind data to [1, 2, 3]
bind moved to data
# data is now invalid
VGA.write(to_text(moved))
```

### Example 2: Shared Borrowing

**Rust:**
```rust
fn sum(list: &Vec<i32>) -> i32 {
    list.iter().sum()
}

let numbers = vec![1, 2, 3, 4, 5];
let total = sum(&numbers);
println!("Sum: {}, Numbers still valid: {:?}", total, numbers);
```

**Glimmer-Weave:**
```glimmer-weave
chant sum(borrow list as List<Number>) -> Number then
    bind total to 0
    for each n in borrow list then
        set total to total + n
    end
    yield total
end

bind numbers to [1, 2, 3, 4, 5]
bind total to sum(borrow numbers)
VGA.write("Sum: " + to_text(total) + ", Numbers: " + to_text(numbers))
```

### Example 3: Mutable Borrowing

**Rust:**
```rust
fn push_ten(list: &mut Vec<i32>) {
    list.push(10);
}

let mut numbers = vec![1, 2, 3];
push_ten(&mut numbers);
println!("{:?}", numbers);  // [1, 2, 3, 10]
```

**Glimmer-Weave:**
```glimmer-weave
chant push_ten(borrow mut list as List<Number>) then
    list.push(10)
end

bind numbers to [1, 2, 3]
push_ten(borrow mut numbers)
VGA.write(to_text(numbers))  # [1, 2, 3, 10]
```

### Example 4: Lifetimes

**Rust:**
```rust
fn first<'a>(list: &'a Vec<i32>) -> &'a i32 {
    &list[0]
}

let numbers = vec![10, 20, 30];
let first_num = first(&numbers);
println!("First: {}", first_num);
```

**Glimmer-Weave:**
```glimmer-weave
chant first<'span>(borrow 'span list as List<Number>) -> borrow 'span Number then
    yield borrow list[0]
end

bind numbers to [10, 20, 30]
bind first_num to first(borrow numbers)
VGA.write("First: " + to_text(first_num))
```

### Example 5: Multiple Lifetimes

**Rust:**
```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

let s1 = "hello";
let s2 = "world!";
let result = longest(s1, s2);
println!("{}", result);
```

**Glimmer-Weave:**
```glimmer-weave
chant longest<'span>(
    borrow 'span x as Text,
    borrow 'span y as Text
) -> borrow 'span Text then
    should x.length() > y.length() then
        yield x
    otherwise
        yield y
    end
end

bind s1 to "hello"
bind s2 to "world!"
bind result to longest(borrow s1, borrow s2)
VGA.write(result)
```

### Example 6: Structs with Borrowed Data

**Rust:**
```rust
struct Slice<'a> {
    data: &'a Vec<i32>,
    start: usize,
    end: usize,
}

fn make_slice<'a>(data: &'a Vec<i32>, start: usize, end: usize) -> Slice<'a> {
    Slice { data, start, end }
}

let numbers = vec![1, 2, 3, 4, 5];
let slice = make_slice(&numbers, 1, 3);
```

**Glimmer-Weave:**
```glimmer-weave
form Slice<'span> with
    data: borrow 'span List<Number>
    start: Number
    end: Number
end

chant make_slice<'span>(
    borrow 'span data as List<Number>,
    start as Number,
    end as Number
) -> Slice<'span> then
    yield Slice {
        data: borrow data,
        start: start,
        end: end
    }
end

bind numbers to [1, 2, 3, 4, 5]
bind slice to make_slice(borrow numbers, 1, 3)
```

---

## Key Differences

### 1. Syntax Philosophy

**Rust:** Terse, symbolic syntax (`&`, `&mut`, `'a`)
**Glimmer-Weave:** Natural language keywords (`borrow`, `borrow mut`, `'span`)

**Why?** Glimmer-Weave prioritizes readability and expressiveness over brevity.

### 2. Variable Declaration

**Rust:**
- `let x` - Immutable
- `let mut x` - Mutable

**Glimmer-Weave:**
- `bind x to` - Immutable binding
- `weave x as` - Mutable variable

**Why?** Different keywords emphasize different ownership semantics.

### 3. Lifetime Names

**Rust:** Typically uses `'a`, `'b`, `'static`
**Glimmer-Weave:** Encourages descriptive names like `'span`, `'output`, `'static`

**Why?** More readable for beginners.

### 4. Result/Option Types

**Rust:**
- `Option<T>`: `Some(value)` / `None`
- `Result<T, E>`: `Ok(value)` / `Err(error)`

**Glimmer-Weave:**
- `Maybe<T>`: `Present(value)` / `Absent`
- `Outcome<T, E>`: `Triumph(value)` / `Mishap(error)`

**Why?** More expressive names that convey intent.

---

## Similarities

Both systems enforce the same core rules:

1. ✅ Every value has exactly one owner
2. ✅ Values are moved by default (except Copy types)
3. ✅ At any time: ONE mutable borrow OR many shared borrows
4. ✅ Borrows must not outlive their referents
5. ✅ All checks happen at compile time
6. ✅ Zero runtime overhead

---

## Advanced Features Comparison

### Smart Pointers

| Rust | Glimmer-Weave | Status |
|------|---------------|--------|
| `Box<T>` | `Box<T>` | Planned |
| `Rc<T>` | `Counted<T>` | Planned |
| `Arc<T>` | `Shared<T>` | Planned |
| `RefCell<T>` | `Cell<T>` | Planned |

### Traits vs Aspects

**Rust:**
```rust
trait Printable {
    fn print(&self);
}

impl Printable for MyType {
    fn print(&self) {
        println!("{:?}", self);
    }
}
```

**Glimmer-Weave:**
```glimmer-weave
aspect Printable then
    chant print(borrow self)
end

embody Printable for MyType then
    chant print(borrow self) then
        VGA.write(to_text(self))
    end
end
```

---

## Migration Tips for Rust Developers

### 1. Replace Symbols with Keywords

```rust
// Rust
fn process(data: &Vec<i32>) { }

// Glimmer-Weave
chant process(borrow data as List<Number>) then ... end
```

### 2. Explicit `borrow` Keyword

```rust
// Rust: Implicit borrow
let data = vec![1, 2, 3];
process(&data);

// Glimmer-Weave: Explicit borrow
bind data to [1, 2, 3]
process(borrow data)
```

### 3. Natural Keywords

```rust
// Rust
let mut x = 5;

// Glimmer-Weave
weave x as 5
```

### 4. Pattern Matching

```rust
// Rust
match result {
    Ok(value) => println!("{}", value),
    Err(e) => println!("Error: {}", e),
}

// Glimmer-Weave
match result with
    when Triumph(value) then VGA.write(to_text(value))
    when Mishap(e) then VGA.write("Error: " + e)
end
```

---

## Performance

Both Glimmer-Weave and Rust have:
- ✅ Zero-cost abstractions
- ✅ No garbage collection
- ✅ Compile-time borrow checking
- ✅ Predictable performance

Glimmer-Weave's readability does NOT come at a performance cost!

---

## Conclusion

If you know Rust, you already understand 90% of Glimmer-Weave's ownership system. The main differences are:

1. **Keywords** instead of symbols
2. **Descriptive names** for types (Outcome vs Result)
3. **Natural language** syntax philosophy

The underlying semantics and safety guarantees are virtually identical. Glimmer-Weave brings Rust-level memory safety to a more readable, natural language syntax.

**Happy coding!** 🦀 ➡️ ✨
