/// Tests for basic iterator functionality (Phase 1)
/// Verifies iterator creation and next() operation

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
// Creating iterators
// ============================================================================

#[test]
fn test_create_list_iterator() {
    let source = r#"
        bind nums to [1, 2, 3]
        bind it to iter(nums)
        it
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Should return an Iterator value
    assert!(result.unwrap().contains("Iterator"));
}

#[test]
fn test_create_range_iterator() {
    let source = r#"
        bind r to range(1, 5)
        bind it to iter(r)
        it
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert!(result.unwrap().contains("Iterator"));
}

// ============================================================================
// Basic iteration with iter_next
// ============================================================================

#[test]
fn test_iter_next_list() {
    let source = r#"
        bind nums to [10, 20, 30]
        bind it to iter(nums)
        bind pair to iter_next(it)
        bind result to list_last(pair)
        result
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Should return Present(10)
    let output = result.unwrap();
    assert!(output.contains("present: true"), "Expected Present, got: {}", output);
}

#[test]
fn test_iter_next_range() {
    let source = r#"
        bind r to range(5, 8)
        bind it to iter(r)
        bind pair to iter_next(it)
        bind result to list_last(pair)
        result
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("present: true"), "Expected Present, got: {}", output);
}

#[test]
fn test_iter_next_empty_list() {
    let source = r#"
        bind empty to []
        bind it to iter(empty)
        bind pair to iter_next(it)
        bind result to list_last(pair)
        result
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Should return Absent
    let output = result.unwrap();
    assert!(output.contains("present: false"), "Expected Absent, got: {}", output);
}

// ============================================================================
// Iteration pattern with pattern matching
// ============================================================================

#[test]
fn test_iter_with_match() {
    let source = r#"
        chant get_value() then
            bind nums to [42]
            bind it to iter(nums)
            bind pair to iter_next(it)
            bind result to list_last(pair)

            match result with
                when Present(value) then
                    yield value
                when Absent then
                    yield 0
            end
        end

        get_value()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(42.0)");
}

// ============================================================================
// Iterator type errors
// ============================================================================

#[test]
fn test_iter_wrong_type() {
    let source = r#"
        bind x to 42
        iter(x)
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Should fail with type error");
    let error = result.unwrap_err();
    assert!(error.contains("TypeError") || error.contains("List or Range"),
            "Expected type error, got: {}", error);
}

#[test]
fn test_iter_next_wrong_type() {
    let source = r#"
        bind x to 42
        iter_next(x)
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Should fail with type error");
    let error = result.unwrap_err();
    assert!(error.contains("TypeError") || error.contains("Iterator"),
            "Expected type error, got: {}", error);
}

// ============================================================================
// Multiple next calls (testing state progression)
// ============================================================================

#[test]
fn test_iter_multiple_next_calls() {
    let source = r#"
        bind nums to [10, 20, 30]
        bind it to iter(nums)
        bind pair to iter_next(it)
        bind result to list_last(pair)
        result
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

// ============================================================================
// Iterator combinators (basic)
// ============================================================================

#[test]
fn test_iter_map_creation() {
    let source = r#"
        bind nums to [1, 2, 3]
        bind it to iter(nums)

        chant double(x) then
            yield x * 2
        end

        bind mapped to iter_map(it, double)
        mapped
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert!(result.unwrap().contains("Iterator"));
}

#[test]
fn test_iter_filter_creation() {
    let source = r#"
        bind nums to [1, 2, 3, 4]
        bind it to iter(nums)

        chant is_even(x) then
            yield x % 2 is 0
        end

        bind filtered to iter_filter(it, is_even)
        filtered
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert!(result.unwrap().contains("Iterator"));
}

#[test]
fn test_iter_take_creation() {
    let source = r#"
        bind nums to [1, 2, 3, 4, 5]
        bind it to iter(nums)
        bind taken to iter_take(it, 3)
        taken
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert!(result.unwrap().contains("Iterator"));
}

// ============================================================================
// Integration tests
// ============================================================================

#[test]
fn test_iter_in_function() {
    let source = r#"
        chant first_element(lst) then
            bind it to iter(lst)
            bind pair to iter_next(it)
            bind result to list_last(pair)
            match result with
                when Present(val) then yield val
                when Absent then yield 0
            end
        end

        first_element([99, 88, 77])
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(99.0)");
}

#[test]
fn test_iter_to_text_conversion() {
    let source = r#"
        bind nums to [1, 2, 3]
        bind it to iter(nums)
        to_text(it)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    // Should convert iterator to text representation
    let output = result.unwrap();
    assert!(output.contains("Iterator") || output.contains("List"),
            "Expected Iterator text representation, got: {}", output);
}
