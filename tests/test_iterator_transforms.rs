/// Tests for iterator transformation combinators (Phase 2)
/// Tests map, filter, and fold using recursive helper functions

use glimmer_weave::{Evaluator, Lexer, Parser};

fn run_program(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut evaluator = Evaluator::new();
    let result = evaluator.eval(&ast).map_err(|e| format!("Runtime error: {:?}", e))?;

    Ok(format!("{:?}", result))
}

// ============================================================================
// Helper: collect function (converts iterator to list)
// ============================================================================

#[test]
fn test_collect_basic() {
    let source = r#"
        chant collecthelper(iterator, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind new_result to list_push(result, value)
                    yield collecthelper(new_iter, new_result)
                when Absent then
                    yield result
            end
        end

        chant collect(iterator) then
            yield collecthelper(iterator, [])
        end

        bind nums to [1, 2, 3]
        bind it to iter(nums)
        collect(it)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    let output = result.unwrap();
    // Should return a list containing [1, 2, 3]
    assert!(output.contains("List"), "Expected List, got: {}", output);
}

#[test]
fn test_collect_empty() {
    let source = r#"
        chant collecthelper(iterator, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind new_result to list_push(result, value)
                    yield collecthelper(new_iter, new_result)
                when Absent then
                    yield result
            end
        end

        chant collect(iterator) then
            yield collecthelper(iterator, [])
        end

        bind empty to []
        bind it to iter(empty)
        bind result to collect(it)
        list_length(result)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(0.0)");
}

// ============================================================================
// Helper: fold function (reduces iterator to single value)
// ============================================================================

#[test]
fn test_fold_sum() {
    let source = r#"
        chant foldhelper(iterator, acc, func) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind new_acc to func(acc, value)
                    yield foldhelper(new_iter, new_acc, func)
                when Absent then
                    yield acc
            end
        end

        chant fold(iterator, init, func) then
            yield foldhelper(iterator, init, func)
        end

        chant add(a, b) then
            yield a + b
        end

        bind nums to [1, 2, 3, 4, 5]
        bind it to iter(nums)
        fold(it, 0, add)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(15.0)");  // 1+2+3+4+5 = 15
}

#[test]
fn test_fold_product() {
    let source = r#"
        chant foldhelper(iterator, acc, func) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind new_acc to func(acc, value)
                    yield foldhelper(new_iter, new_acc, func)
                when Absent then
                    yield acc
            end
        end

        chant fold(iterator, init, func) then
            yield foldhelper(iterator, init, func)
        end

        chant multiply(a, b) then
            yield a * b
        end

        bind nums to [2, 3, 4]
        bind it to iter(nums)
        fold(it, 1, multiply)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(24.0)");  // 2*3*4 = 24
}

// ============================================================================
// Helper: reduce function (like fold but uses first element as init)
// ============================================================================

#[test]
fn test_reduce_sum() {
    let source = r#"
        chant foldhelper(iterator, acc, func) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind new_acc to func(acc, value)
                    yield foldhelper(new_iter, new_acc, func)
                when Absent then
                    yield acc
            end
        end

        chant fold(iterator, init, func) then
            yield foldhelper(iterator, init, func)
        end

        chant reducehelper(initial, iterator, func) then
            yield fold(iterator, initial, func)
        end

        chant reduce(iterator, func) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind firstval to list_last(pair)
            match firstval with
                when Present(initial) then
                    bind result to reducehelper(initial, new_iter, func)
                    yield Present(result)
                when Absent then
                    yield Absent
            end
        end

        chant add(a, b) then
            yield a + b
        end

        bind nums to [1, 2, 3, 4, 5]
        bind it to iter(nums)
        bind result to reduce(it, add)
        match result with
            when Present(val) then val
            when Absent then -1
        end
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(15.0)");  // 1+2+3+4+5 = 15
}

#[test]
fn test_reduce_empty() {
    let source = r#"
        chant foldhelper(iterator, acc, func) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind new_acc to func(acc, value)
                    yield foldhelper(new_iter, new_acc, func)
                when Absent then
                    yield acc
            end
        end

        chant fold(iterator, init, func) then
            yield foldhelper(iterator, init, func)
        end

        chant reducehelper(initial, iterator, func) then
            yield fold(iterator, initial, func)
        end

        chant reduce(iterator, func) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind firstval to list_last(pair)
            match firstval with
                when Present(initial) then
                    bind result to reducehelper(initial, new_iter, func)
                    yield Present(result)
                when Absent then
                    yield Absent
            end
        end

        chant add(a, b) then
            yield a + b
        end

        bind empty to []
        bind it to iter(empty)
        bind result to reduce(it, add)
        match result with
            when Present(val) then val
            when Absent then -1
        end
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(-1.0)");  // Empty returns Absent
}

#[test]
fn test_reduce_single() {
    let source = r#"
        chant foldhelper(iterator, acc, func) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind new_acc to func(acc, value)
                    yield foldhelper(new_iter, new_acc, func)
                when Absent then
                    yield acc
            end
        end

        chant fold(iterator, init, func) then
            yield foldhelper(iterator, init, func)
        end

        chant reducehelper(initial, iterator, func) then
            yield fold(iterator, initial, func)
        end

        chant reduce(iterator, func) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind firstval to list_last(pair)
            match firstval with
                when Present(initial) then
                    bind result to reducehelper(initial, new_iter, func)
                    yield Present(result)
                when Absent then
                    yield Absent
            end
        end

        chant multiply(a, b) then
            yield a * b
        end

        bind single to [42]
        bind it to iter(single)
        bind result to reduce(it, multiply)
        match result with
            when Present(val) then val
            when Absent then -1
        end
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(42.0)");  // Single element returns itself
}

// ============================================================================
// Map transformation (eager implementation using recursion)
// ============================================================================

#[test]
fn test_map_double() {
    let source = r#"
        chant maphelper(iterator, func, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind transformed to func(value)
                    bind new_result to list_push(result, transformed)
                    yield maphelper(new_iter, func, new_result)
                when Absent then
                    yield iter(result)
            end
        end

        chant map(iterator, func) then
            yield maphelper(iterator, func, [])
        end

        chant collecthelper(iterator, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind new_result to list_push(result, value)
                    yield collecthelper(new_iter, new_result)
                when Absent then
                    yield result
            end
        end

        chant collect(iterator) then
            yield collecthelper(iterator, [])
        end

        chant double(x) then
            yield x * 2
        end

        bind nums to [1, 2, 3]
        bind it to iter(nums)
        bind mapped to map(it, double)
        collect(mapped)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Should return [2, 4, 6]
    let output = result.unwrap();
    assert!(output.contains("2") && output.contains("4") && output.contains("6"),
            "Expected [2, 4, 6], got: {}", output);
}

// ============================================================================
// Filter transformation (eager implementation using recursion)
// ============================================================================

#[test]
fn test_filter_evens() {
    let source = r#"
        chant addif(keep, result, value) then
            should keep then
                yield list_push(result, value)
            otherwise
                yield result
            end
        end

        chant keepif(iterator, predicate, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind keep to predicate(value)
                    bind next_result to addif(keep, result, value)
                    yield keepif(new_iter, predicate, next_result)
                when Absent then
                    yield iter(result)
            end
        end

        chant filterit(iterator, predicate) then
            yield keepif(iterator, predicate, [])
        end

        chant collecthelper(iterator, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind new_result to list_push(result, value)
                    yield collecthelper(new_iter, new_result)
                when Absent then
                    yield result
            end
        end

        chant collect(iterator) then
            yield collecthelper(iterator, [])
        end

        chant iseven(x) then
            yield x % 2 is 0
        end

        bind nums to [1, 2, 3, 4, 5, 6]
        bind it to iter(nums)
        bind filtered to filterit(it, iseven)
        collect(filtered)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Should return [2, 4, 6]
    let output = result.unwrap();
    assert!(output.contains("2") && output.contains("4") && output.contains("6"),
            "Expected [2, 4, 6], got: {}", output);
}

// ============================================================================
// Combined transformations
// ============================================================================

#[test]
fn test_map_then_filter() {
    let source = r#"
        chant addif(keep, result, value) then
            should keep then
                yield list_push(result, value)
            otherwise
                yield result
            end
        end

        chant maphelper(iterator, func, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind transformed to func(value)
                    bind new_result to list_push(result, transformed)
                    yield maphelper(new_iter, func, new_result)
                when Absent then
                    yield iter(result)
            end
        end

        chant map(iterator, func) then
            yield maphelper(iterator, func, [])
        end

        chant keepif(iterator, predicate, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind keep to predicate(value)
                    bind next_result to addif(keep, result, value)
                    yield keepif(new_iter, predicate, next_result)
                when Absent then
                    yield iter(result)
            end
        end

        chant filterit(iterator, predicate) then
            yield keepif(iterator, predicate, [])
        end

        chant collecthelper(iterator, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind new_result to list_push(result, value)
                    yield collecthelper(new_iter, new_result)
                when Absent then
                    yield result
            end
        end

        chant collect(iterator) then
            yield collecthelper(iterator, [])
        end

        chant double(x) then
            yield x * 2
        end

        chant greaterthanfive(x) then
            yield x greater than 5
        end

        bind nums to [1, 2, 3, 4, 5]
        bind it to iter(nums)
        bind mapped to map(it, double)
        bind filtered to filterit(mapped, greaterthanfive)
        collect(filtered)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // [1, 2, 3, 4, 5] -> map(*2) -> [2, 4, 6, 8, 10] -> filter(>5) -> [6, 8, 10]
    let output = result.unwrap();
    assert!(output.contains("6") && output.contains("8") && output.contains("10"),
            "Expected [6, 8, 10], got: {}", output);
}
