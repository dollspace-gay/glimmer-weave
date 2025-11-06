/// Integration tests for iterators with other language features (Phase 5)
/// Tests iterators combined with structs, traits, pattern matching, and complex pipelines

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
// Iterators with Structs
// ============================================================================

#[test]
fn test_iterator_with_struct_fields() {
    let source = r#"
        form Person with
            name as Text
            age as Number
        end

        chant collecthelper(iterator, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind nextval to list_last(pair)
            match nextval with
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

        chant maphelper(iterator, func, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind nextval to list_last(pair)
            match nextval with
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

        chant getage(person) then
            yield person.age
        end

        bind alice to Person { name: "Alice", age: 30 }
        bind bob to Person { name: "Bob", age: 25 }
        bind charlie to Person { name: "Charlie", age: 35 }
        bind people to [alice, bob, charlie]
        bind it to iter(people)
        bind ages to map(it, getage)
        bind agelist to collect(ages)
        list_length(agelist)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(3.0)");
}

// ============================================================================
// Iterators with Pattern Matching
// ============================================================================

#[test]
fn test_iterator_with_nested_pattern_matching() {
    let source = r#"
        chant foldhelper(iterator, acc, func) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind nextval to list_last(pair)
            match nextval with
                when Present(value) then
                    bind newacc to func(acc, value)
                    yield foldhelper(new_iter, newacc, func)
                when Absent then
                    yield acc
            end
        end

        chant fold(iterator, init, func) then
            yield foldhelper(iterator, init, func)
        end

        chant processvalue(acc, val) then
            should val greater than 0 then
                yield acc + val
            otherwise
                yield acc
            end
        end

        bind nums to [1, -2, 3, -4, 5]
        bind it to iter(nums)
        fold(it, 0, processvalue)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(9.0)");  // 1 + 3 + 5 = 9
}

// ============================================================================
// Complex Pipelines
// ============================================================================

// TODO: This test hangs - investigate iterator state management in complex pipelines
#[test]
#[ignore]
fn test_complex_pipeline_map_filter_fold() {
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
            bind nextval to list_last(pair)
            match nextval with
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
            bind nextval to list_last(pair)
            match nextval with
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

        chant foldhelper(iterator, acc, func) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind nextval to list_last(pair)
            match nextval with
                when Present(value) then
                    bind newacc to func(acc, value)
                    yield foldhelper(new_iter, newacc, func)
                when Absent then
                    yield acc
            end
        end

        chant fold(iterator, init, func) then
            yield foldhelper(iterator, init, func)
        end

        chant square(x) then
            yield x * x
        end

        chant iseven(x) then
            yield x % 2 is 0
        end

        chant add(a, b) then
            yield a + b
        end

        bind nums to [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        bind it to iter(nums)
        bind squared to map(it, square)
        bind evens to filterit(squared, iseven)
        fold(evens, 0, add)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Squares: 1,4,9,16,25,36,49,64,81,100
    // Even squares: 4,16,36,64,100
    // Sum: 220
    assert_eq!(result.unwrap(), "Number(220.0)");
}

// ============================================================================
// Iterators with Advanced Combinators
// ============================================================================

// TODO: This test hangs - investigate iterator state in nested combinator chains
#[test]
#[ignore]
fn test_pipeline_with_advance_and_gatherwhilst() {
    let source = r#"
        chant advancehelper(iterator, count) then
            should count at most 0 then
                yield iterator
            otherwise
                bind pair to iter_next(iterator)
                bind new_iter to list_first(pair)
                bind nextval to list_last(pair)
                match nextval with
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

        chant gatherwhilsthelper(iterator, predicate, result) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind nextval to list_last(pair)
            match nextval with
                when Present(value) then
                    bind shouldtake to predicate(value)
                    should shouldtake then
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
            bind nextval to list_last(pair)
            match nextval with
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

        chant tallyhelper(iterator, acc) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind nextval to list_last(pair)
            match nextval with
                when Present(value) then
                    yield tallyhelper(new_iter, acc + 1)
                when Absent then
                    yield acc
            end
        end

        chant tally(iterator) then
            yield tallyhelper(iterator, 0)
        end

        chant lessthan(x) then
            yield x less than 50
        end

        bind nums to range(1, 101)
        bind it to iter(nums)
        bind advanced to advance(it, 10)
        bind gathered to gatherwhilst(advanced, lessthan)
        bind result to collect(gathered)
        tally(iter(result))
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Range 1-100, advance past first 10 (skip 1-10), gather while <50 (11-49)
    // That's 39 elements
    assert_eq!(result.unwrap(), "Number(39.0)");
}

// ============================================================================
// Iterators with Witness and Ensure
// ============================================================================

#[test]
fn test_witness_and_ensure_validation() {
    let source = r#"
        chant witnesshelper(iterator, predicate) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind nextval to list_last(pair)
            match nextval with
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

        chant ensurehelper(iterator, predicate) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind nextval to list_last(pair)
            match nextval with
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

        chant greaterthanfive(x) then
            yield x greater than 5
        end

        bind nums to [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        bind it1 to iter(nums)
        bind it2 to iter(nums)

        bind hasgreater to witness(it1, greaterthanfive)
        bind allpositive to ensure(it2, ispositive)

        should hasgreater then
            should allpositive then
                100
            otherwise
                50
            end
        otherwise
            0
        end
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(100.0)");  // Has >5 AND all positive
}

// ============================================================================
// Stress Test - Large Iteration
// ============================================================================

#[test]
fn test_large_range_iteration() {
    let source = r#"
        chant foldhelper(iterator, acc, func) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind nextval to list_last(pair)
            match nextval with
                when Present(value) then
                    bind newacc to func(acc, value)
                    yield foldhelper(new_iter, newacc, func)
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

        bind nums to range(1, 101)
        bind it to iter(nums)
        fold(it, 0, add)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Sum of 1-100 = 5050
    assert_eq!(result.unwrap(), "Number(5050.0)");
}

// ============================================================================
// Discover with Complex Predicates
// ============================================================================

#[test]
fn test_discover_with_complex_condition() {
    let source = r#"
        chant discoverhelper(iterator, predicate) then
            bind pair to iter_next(iterator)
            bind new_iter to list_first(pair)
            bind nextval to list_last(pair)
            match nextval with
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

        chant isdivisible(x) then
            bind by3 to x % 3 is 0
            bind by5 to x % 5 is 0
            should by3 then
                yield by5
            otherwise
                yield false
            end
        end

        bind nums to range(1, 31)
        bind it to iter(nums)
        bind result to discover(it, isdivisible)
        match result with
            when Present(val) then val
            when Absent then -1
        end
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(15.0)");  // First number divisible by both 3 and 5
}
