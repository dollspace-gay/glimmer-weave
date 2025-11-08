# Ownership and Borrowing Tutorial

**Learn Glimmer-Weave's memory safety system step by step**

This tutorial teaches you Glimmer-Weave's ownership and borrowing system through practical examples. By the end, you'll understand how to write memory-safe code without a garbage collector.

---

## Table of Contents

1. [Introduction](#introduction)
2. [Lesson 1: Ownership Basics](#lesson-1-ownership-basics)
3. [Lesson 2: Move Semantics](#lesson-2-move-semantics)
4. [Lesson 3: Shared Borrowing](#lesson-3-shared-borrowing)
5. [Lesson 4: Mutable Borrowing](#lesson-4-mutable-borrowing)
6. [Lesson 5: Lifetimes](#lesson-5-lifetimes)
7. [Lesson 6: Common Patterns](#lesson-6-common-patterns)
8. [Troubleshooting](#troubleshooting)
9. [Next Steps](#next-steps)

---

## Introduction

### Why Ownership?

Glimmer-Weave prevents memory bugs like:
- **Use-after-free**: Accessing memory that's been freed
- **Double-free**: Freeing the same memory twice
- **Dangling pointers**: References to freed memory

All checks happen at **compile time** with **zero runtime overhead**.

### Core Rules

1. Every value has exactly one owner
2. When the owner goes out of scope, the value is freed
3. You can borrow a value temporarily (shared or exclusive)

Let's learn by doing!

---

## Lesson 1: Ownership Basics

### Every Value Has an Owner

```glimmer-weave
bind data to [1, 2, 3]   # 'data' owns the list
VGA.write(to_text(data)) # OK: owner can use the value
```

When `data` goes out of scope (end of function/block), the list is automatically freed.

### Scopes and Lifetime

```glimmer-weave
chant example() then
    bind inner to [1, 2, 3]
    VGA.write(to_text(inner))  # OK: inner is in scope
end  # inner dropped here, memory freed

# inner doesn't exist here
```

### Practice Exercise 1

Create a function that creates and returns a list of numbers 1-5:

<details>
<summary>Solution</summary>

```glimmer-weave
chant create_list() -> List<Number> then
    bind numbers to []
    for each i in range(1, 6) then
        numbers.push(i)
    end
    yield numbers  # Transfer ownership to caller
end

bind my_list to create_list()
VGA.write(to_text(my_list))  # [1, 2, 3, 4, 5]
```
</details>

---

## Lesson 2: Move Semantics

### Values Can Move

When you assign or pass a value, ownership transfers (for non-Copy types):

```glimmer-weave
bind data to [1, 2, 3]    # data owns [1,2,3]
bind moved to data         # Ownership transfers to 'moved'
# data is now invalid!

# VGA.write(to_text(data))  # ERROR: value was moved
VGA.write(to_text(moved))   # OK: moved now owns it
```

### Copy Types vs Move Types

**Copy Types** (automatically copied):
- `Number`
- `Truth`
- `Nothing`

**Move Types** (ownership transfers):
- `Text`
- `List<T>`
- `Map`
- Custom structs

```glimmer-weave
# Numbers are copied
bind x to 42
bind y to x
VGA.write(to_text(x + y))  # OK: both valid

# Lists are moved
bind list1 to [1, 2, 3]
bind list2 to list1
# list1 is now invalid
```

### Explicit Cloning

Use `.replicate()` for deep copies:

```glimmer-weave
bind original to [1, 2, 3]
bind copy to original.replicate()

original.push(4)
copy.push(5)

VGA.write(to_text(original))  # [1, 2, 3, 4]
VGA.write(to_text(copy))      # [1, 2, 3, 5]
```

### Practice Exercise 2

Fix this code so both `list1` and `list2` are valid:

```glimmer-weave
bind list1 to [10, 20, 30]
bind list2 to list1
VGA.write(to_text(list1))  # ERROR: how to fix?
VGA.write(to_text(list2))
```

<details>
<summary>Solution</summary>

```glimmer-weave
bind list1 to [10, 20, 30]
bind list2 to list1.replicate()  # Clone instead of move
VGA.write(to_text(list1))  # OK now
VGA.write(to_text(list2))
```
</details>

---

## Lesson 3: Shared Borrowing

### The `borrow` Keyword

Borrow a value temporarily without taking ownership:

```glimmer-weave
chant print_list(borrow list as List<Number>) then
    for each item in borrow list then
        VGA.write(to_text(item))
    end
end

bind numbers to [1, 2, 3]
print_list(borrow numbers)
# numbers is still valid here!
numbers.push(4)
```

### Multiple Shared Borrows

You can have many shared borrows simultaneously:

```glimmer-weave
chant sum(borrow list as List<Number>) -> Number then
    bind total to 0
    for each n in borrow list then
        set total to total + n
    end
    yield total
end

chant length(borrow list as List<Number>) -> Number then
    yield list.length()
end

bind nums to [5, 10, 15]
bind s to sum(borrow nums)
bind len to length(borrow nums)  # OK: multiple shared borrows
VGA.write("Sum: " + to_text(s) + ", Length: " + to_text(len))
```

### Borrowing Rules

While a value is borrowed (shared):
- ✅ You can read it
- ❌ You cannot modify it
- ✅ You can borrow it again (shared)
- ❌ You cannot mutably borrow it

### Practice Exercise 3

Write a function that finds the maximum value in a list without consuming it:

<details>
<summary>Solution</summary>

```glimmer-weave
chant find_max(borrow list as List<Number>) -> Number then
    bind max to list[0]
    for each item in borrow list then
        should item > max then
            set max to item
        end
    end
    yield max
end

bind numbers to [42, 17, 99, 3, 56]
bind maximum to find_max(borrow numbers)
VGA.write("Max: " + to_text(maximum))
# numbers still valid here
```
</details>

---

## Lesson 4: Mutable Borrowing

### The `borrow mut` Keywords

Borrow with exclusive write access:

```glimmer-weave
chant add_ten(borrow mut list as List<Number>) then
    list.push(10)
end

bind numbers to [1, 2, 3]
add_ten(borrow mut numbers)
VGA.write(to_text(numbers))  # [1, 2, 3, 10]
```

### Exclusive Access

Only ONE mutable borrow allowed at a time:

```glimmer-weave
bind data to [1, 2, 3]

# OK: One mutable borrow
bind ref1 to borrow mut data
ref1.push(4)
# ref1 no longer used, borrow ends

# OK: Another mutable borrow (sequential)
bind ref2 to borrow mut data
ref2.push(5)
```

### Cannot Mix Shared and Mutable

```glimmer-weave
bind data to [1, 2, 3]

bind shared to borrow data
# bind mutable to borrow mut data  # ERROR: already borrowed as shared

# Instead, ensure shared borrow ends first:
bind len to shared.length()
# shared no longer used, borrow ends

bind mutable to borrow mut data  # OK now
mutable.push(4)
```

### Practice Exercise 4

Write a function that doubles all values in a list in-place:

<details>
<summary>Solution</summary>

```glimmer-weave
chant double_all(borrow mut list as List<Number>) then
    for each i in range(0, list.length()) then
        set list[i] to list[i] * 2
    end
end

bind numbers to [1, 2, 3, 4, 5]
double_all(borrow mut numbers)
VGA.write(to_text(numbers))  # [2, 4, 6, 8, 10]
```
</details>

---

## Lesson 5: Lifetimes

### Why Lifetimes?

Lifetimes ensure borrowed references don't outlive the data they point to.

### Explicit Lifetime Annotations

Use `'span` or `'a` for lifetime parameters:

```glimmer-weave
chant first<'span>(borrow 'span list as List<Number>) -> borrow 'span Number then
    yield borrow list[0]
end
```

This says: "The returned reference has the same lifetime as the input list"

### Lifetime Elision

Simple cases are inferred automatically:

```glimmer-weave
# You write:
chant get_first(borrow list as List<Number>) -> borrow Number then
    yield borrow list[0]
end

# Compiler infers:
chant get_first<'span>(borrow 'span list as List<Number>) -> borrow 'span Number then
    yield borrow list[0]
end
```

### Multiple Lifetimes

When you have multiple borrowed inputs:

```glimmer-weave
chant longest<'span>(
    borrow 'span a as Text,
    borrow 'span b as Text
) -> borrow 'span Text then
    should a.length() > b.length() then
        yield a
    otherwise
        yield b
    end
end
```

### Static Lifetime

`'static` lives for the entire program:

```glimmer-weave
bind message: borrow 'static Text to "Hello, World!"
# String literals have 'static lifetime
```

### Practice Exercise 5

Write a function that returns a reference to the first element that's greater than 10:

<details>
<summary>Solution</summary>

```glimmer-weave
chant find_gt_10<'span>(borrow 'span list as List<Number>) -> Maybe<borrow 'span Number> then
    for each item in borrow list then
        should item > 10 then
            yield Present(borrow item)
        end
    end
    yield Absent
end

bind nums to [5, 8, 15, 3, 20]
match find_gt_10(borrow nums) with
    when Present(value) then
        VGA.write("Found: " + to_text(value))
    when Absent then
        VGA.write("None found")
end
```
</details>

---

## Lesson 6: Common Patterns

### Pattern 1: Builder (Returns Ownership)

```glimmer-weave
chant build_list(start as Number, end as Number) -> List<Number> then
    bind result to []
    for each i in range(start, end + 1) then
        result.push(i)
    end
    yield result
end
```

### Pattern 2: Transform (Consumes and Returns New)

```glimmer-weave
chant map_square(list as List<Number>) -> List<Number> then
    bind result to []
    for each item in list then
        result.push(item * item)
    end
    yield result
end
```

### Pattern 3: In-Place Modify (Mutable Borrow)

```glimmer-weave
chant increment(borrow mut list as List<Number>) then
    for each i in range(0, list.length()) then
        set list[i] to list[i] + 1
    end
end
```

### Pattern 4: Compute (Shared Borrow)

```glimmer-weave
chant average(borrow list as List<Number>) -> Number then
    bind sum to 0
    for each item in borrow list then
        set sum to sum + item
    end
    yield sum / list.length()
end
```

### Pattern 5: Chain Transformations

```glimmer-weave
bind nums to build_list(1, 5)       # [1,2,3,4,5]
bind squared to map_square(nums)    # [1,4,9,16,25]
increment(borrow mut squared)       # [2,5,10,17,26]
bind avg to average(borrow squared)
VGA.write("Average: " + to_text(avg))
```

---

## Troubleshooting

### Error: Value used after move

**Problem:**
```glimmer-weave
bind data to [1, 2, 3]
bind moved to data
data.length()  # ERROR
```

**Solution:** Use the moved value or clone before moving:
```glimmer-weave
# Option 1: Use moved value
moved.length()

# Option 2: Clone before move
bind copy to data.replicate()
bind moved to data
copy.length()  # OK
```

### Error: Cannot borrow mutably while shared borrow exists

**Problem:**
```glimmer-weave
bind data to [1, 2, 3]
bind ref to borrow data
modify(borrow mut data)  # ERROR
```

**Solution:** Ensure shared borrow ends first:
```glimmer-weave
bind len to ref.length()
# ref no longer used, borrow ends
modify(borrow mut data)  # OK now
```

### Error: Multiple mutable borrows

**Problem:**
```glimmer-weave
bind ref1 to borrow mut data
bind ref2 to borrow mut data  # ERROR
```

**Solution:** Use borrows sequentially:
```glimmer-weave
bind ref1 to borrow mut data
ref1.push(1)
# ref1 no longer used

bind ref2 to borrow mut data  # OK now
ref2.push(2)
```

---

## Next Steps

You now understand Glimmer-Weave's ownership system!

**Practice Projects:**
1. Write a linked list implementation
2. Build a simple memory allocator
3. Create an iterator pattern

**Further Reading:**
- [Ownership Design Document](ownership_borrowing_design.md)
- [Example Programs](../examples/)
- [CLAUDE.md - Ownership Section](../CLAUDE.md#ownership--borrowing-system)

**Advanced Topics:**
- Smart pointers (Box, Rc, Arc)
- Interior mutability (RefCell)
- Trait objects with lifetimes
- Lifetime subtyping

Happy coding with memory safety! 🎉
