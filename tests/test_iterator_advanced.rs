/// Tests for advanced iterator combinators (Phase 4)
/// Tests skip, take_while, zip, chain, any, all, and find

use glimmer_weave::{Evaluator, Lexer, Parser};

fn run_program(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut evaluator = Evaluator::new();
    let result = evaluator.eval(&ast).map_err(|e| format!("Runtime error: {:?}", e))?;

    Ok(format!("{:?}", result))
}

// ============================================================================
// Advance combinator - advance past first N elements
// ============================================================================

#[test]
fn test_advance_basic() {
    let source = r#"
        chant advancehelper(iterator, count) then
            should count at most 0 then
                yield iterator
            otherwise
                bind pair to iter_next(iterator)
                bind new_iter to list_first(pair)
                bind next_val to list_last(pair)
                match next_val with
                    when Present(value) then
                        yield advancehelper(new_iter, count - 1)
                    when Absent then
                        yield iterator
                end
            end
        end

        chant advance(iterator, count) then
            yield advancehelper(iterator, count)
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

        bind nums to [1, 2, 3, 4, 5]
        bind it to iter(nums)
        bind skipped to advance(it, 2)
        collect(skipped)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Should return [3, 4, 5]
    let output = result.unwrap();
    assert!(output.contains("3") && output.contains("4") && output.contains("5"),
            "Expected [3, 4, 5], got: {}", output);
}

#[test]
fn test_advance_zero() {
    let source = r#"
        chant advancehelper(iterator, count) then
            should count at most 0 then
                yield iterator
            otherwise
                bind pair to iter_next(iterator)
                bind new_iter to list_first(pair)
                bind next_val to list_last(pair)
                match next_val with
                    when Present(value) then
                        yield advancehelper(new_iter, count - 1)
                    when Absent then
                        yield iterator
                end
            end
        end

        chant advance(iterator, count) then
            yield advancehelper(iterator, count)
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

        bind nums to [1, 2, 3]
        bind it to iter(nums)
        bind skipped to advance(it, 0)
        collect(skipped)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Should return [1, 2, 3] - nothing skipped
    let output = result.unwrap();
    assert!(output.contains("1") && output.contains("2") && output.contains("3"),
            "Expected [1, 2, 3], got: {}", output);
}

// ============================================================================
// Witness combinator - witness if any element satisfies predicate
// ============================================================================

#[test]
fn test_witness_found() {
    let source = r#"
        chant witnesshelper(iterator, predicate) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind matches to predicate(value)
                    should matches then
                        yield true
                    otherwise
                        yield witnesshelper(new_iter, predicate)
                    end
                when Absent then
                    yield false
            end
        end

        chant witness(iterator, predicate) then
            yield witnesshelper(iterator, predicate)
        end

        chant greaterthanfive(x) then
            yield x greater than 5
        end

        bind nums to [1, 2, 3, 6, 4]
        bind it to iter(nums)
        witness(it, greaterthanfive)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Truth(true)");
}

#[test]
fn test_witness_not_found() {
    let source = r#"
        chant witnesshelper(iterator, predicate) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind matches to predicate(value)
                    should matches then
                        yield true
                    otherwise
                        yield witnesshelper(new_iter, predicate)
                    end
                when Absent then
                    yield false
            end
        end

        chant witness(iterator, predicate) then
            yield witnesshelper(iterator, predicate)
        end

        chant greaterthanfive(x) then
            yield x greater than 5
        end

        bind nums to [1, 2, 3, 4, 5]
        bind it to iter(nums)
        witness(it, greaterthanfive)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Truth(false)");
}

// ============================================================================
// Ensure combinator - ensure all elements satisfy predicate
// ============================================================================

#[test]
fn test_ensure_true() {
    let source = r#"
        chant ensurehelper(iterator, predicate) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind matches to predicate(value)
                    should matches then
                        yield ensurehelper(new_iter, predicate)
                    otherwise
                        yield false
                    end
                when Absent then
                    yield true
            end
        end

        chant ensure(iterator, predicate) then
            yield ensurehelper(iterator, predicate)
        end

        chant ispositive(x) then
            yield x greater than 0
        end

        bind nums to [1, 2, 3, 4, 5]
        bind it to iter(nums)
        ensure(it, ispositive)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Truth(true)");
}

#[test]
fn test_ensure_false() {
    let source = r#"
        chant ensurehelper(iterator, predicate) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind matches to predicate(value)
                    should matches then
                        yield ensurehelper(new_iter, predicate)
                    otherwise
                        yield false
                    end
                when Absent then
                    yield true
            end
        end

        chant ensure(iterator, predicate) then
            yield ensurehelper(iterator, predicate)
        end

        chant ispositive(x) then
            yield x greater than 0
        end

        bind nums to [1, 2, -3, 4, 5]
        bind it to iter(nums)
        ensure(it, ispositive)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Truth(false)");
}

// ============================================================================
// Discover combinator - discover first matching element
// ============================================================================

#[test]
fn test_discover_found() {
    let source = r#"
        chant discoverhelper(iterator, predicate) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind matches to predicate(value)
                    should matches then
                        yield Present(value)
                    otherwise
                        yield discoverhelper(new_iter, predicate)
                    end
                when Absent then
                    yield Absent
            end
        end

        chant discover(iterator, predicate) then
            yield discoverhelper(iterator, predicate)
        end

        chant iseven(x) then
            yield x % 2 is 0
        end

        bind nums to [1, 3, 5, 6, 8]
        bind it to iter(nums)
        bind result to discover(it, iseven)
        match result with
            when Present(val) then val
            when Absent then -1
        end
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(6.0)");  // First even number
}

#[test]
fn test_discover_not_found() {
    let source = r#"
        chant discoverhelper(iterator, predicate) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind matches to predicate(value)
                    should matches then
                        yield Present(value)
                    otherwise
                        yield discoverhelper(new_iter, predicate)
                    end
                when Absent then
                    yield Absent
            end
        end

        chant discover(iterator, predicate) then
            yield discoverhelper(iterator, predicate)
        end

        chant iseven(x) then
            yield x % 2 is 0
        end

        bind nums to [1, 3, 5, 7, 9]
        bind it to iter(nums)
        bind result to discover(it, iseven)
        match result with
            when Present(val) then val
            when Absent then -1
        end
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(-1.0)");  // Not found
}

// ============================================================================
// Tally combinator - tally elements in iterator
// ============================================================================

#[test]
fn test_tally_basic() {
    let source = r#"
        chant tallyhelper(iterator, acc) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    yield tallyhelper(new_iter, acc + 1)
                when Absent then
                    yield acc
            end
        end

        chant tally(iterator) then
            yield tallyhelper(iterator, 0)
        end

        bind nums to [1, 2, 3, 4, 5]
        bind it to iter(nums)
        tally(it)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(5.0)");
}

#[test]
fn test_tally_empty() {
    let source = r#"
        chant tallyhelper(iterator, acc) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    yield tallyhelper(new_iter, acc + 1)
                when Absent then
                    yield acc
            end
        end

        chant tally(iterator) then
            yield tallyhelper(iterator, 0)
        end

        bind empty to []
        bind it to iter(empty)
        tally(it)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(0.0)");
}

// ============================================================================
// Gatherwhilst combinator - gather elements whilst predicate is true
// ============================================================================

#[test]
fn test_gatherwhilst_basic() {
    let source = r#"
        chant gatherwhilsthelper(iterator, predicate, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind should_take to predicate(value)
                    should should_take then
                        bind new_result to list_push(result, value)
                        yield gatherwhilsthelper(new_iter, predicate, new_result)
                    otherwise
                        yield iter(result)
                    end
                when Absent then
                    yield iter(result)
            end
        end

        chant gatherwhilst(iterator, predicate) then
            yield gatherwhilsthelper(iterator, predicate, [])
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

        chant lessthanfive(x) then
            yield x less than 5
        end

        bind nums to [1, 2, 3, 4, 5, 6, 7]
        bind it to iter(nums)
        bind taken to gatherwhilst(it, lessthanfive)
        collect(taken)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Should return [1, 2, 3, 4] - stops at 5
    let output = result.unwrap();
    assert!(output.contains("1") && output.contains("2") && output.contains("3") && output.contains("4"),
            "Expected [1, 2, 3, 4], got: {}", output);
    assert!(!output.contains("5"), "Should not contain 5, got: {}", output);
}

// ============================================================================
// Combined advanced combinators
// ============================================================================

#[test]
fn test_advance_then_gatherwhilst() {
    let source = r#"
        chant skiphelper(iterator, count) then
            should count at most 0 then
                yield iterator
            otherwise
                bind pair to iter_next(iterator)
                bind new_iter to list_first(pair)
                bind next_val to list_last(pair)
                match next_val with
                    when Present(value) then
                        yield skiphelper(new_iter, count - 1)
                    when Absent then
                        yield iterator
                end
            end
        end

        chant skipit(iterator, count) then
            yield skiphelper(iterator, count)
        end

        chant gatherwhilsthelper(iterator, predicate, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind next_val to list_last(pair)
            match next_val with
                when Present(value) then
                    bind should_take to predicate(value)
                    should should_take then
                        bind new_result to list_push(result, value)
                        yield gatherwhilsthelper(new_iter, predicate, new_result)
                    otherwise
                        yield iter(result)
                    end
                when Absent then
                    yield iter(result)
            end
        end

        chant gatherwhilst(iterator, predicate) then
            yield gatherwhilsthelper(iterator, predicate, [])
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

        chant lessthanfive(x) then
            yield x less than 5
        end

        bind nums to [1, 2, 3, 4, 5, 6]
        bind it to iter(nums)
        bind skipped to skipit(it, 1)
        bind taken to gatherwhilst(skipped, lessthanfive)
        collect(taken)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Skip 1 -> [2,3,4,5,6], take while <5 -> [2,3,4]
    let output = result.unwrap();
    assert!(output.contains("2") && output.contains("3") && output.contains("4"),
            "Expected [2, 3, 4], got: {}", output);
    assert!(!output.contains("1") && !output.contains("5"),
            "Should not contain 1 or 5, got: {}", output);
}
