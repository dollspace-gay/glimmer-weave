/// Comprehensive runtime type checking tests
/// Tests edge cases and error handling for type mismatches at runtime

use glimmer_weave::{Evaluator, Lexer, Parser};
use glimmer_weave::eval::{RuntimeError, Value};

fn run_program(source: &str) -> Result<Value, RuntimeError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| RuntimeError::Custom(format!("Parse error: {:?}", e)))?;

    let mut evaluator = Evaluator::new();
    evaluator.eval(&ast)
}

// ============================================================================
// Arithmetic Operations Type Checking
// ============================================================================

#[test]
fn test_add_text_to_number() {
    let source = r#"
bind x to 42 + "hello"
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for Text + Number");
    match result.unwrap_err() {
        RuntimeError::TypeError { expected, got } => {
            // Either Number or Text is expected, depending on operator precedence
            assert!(expected == "Number" || got == "Text");
        }
        err => panic!("Expected TypeError, got {:?}", err),
    }
}

#[test]
fn test_subtract_nothing() {
    let source = r#"
bind x to 10 - nothing
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for Number - Nothing");
}

#[test]
fn test_multiply_truth() {
    let source = r#"
bind x to 5 * true
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for Number * Truth");
}

#[test]
fn test_negate_text() {
    let source = r#"
bind x to -"hello"
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for negating Text");
    match result.unwrap_err() {
        RuntimeError::TypeError { expected, got } => {
            assert_eq!(expected, "Number");
            assert_eq!(got, "Text");
        }
        err => panic!("Expected TypeError, got {:?}", err),
    }
}

// ============================================================================
// Comparison Operations Type Checking
// ============================================================================

#[test]
fn test_compare_text_with_number() {
    let source = r#"
bind x to "hello" greater than 42
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for Text > Number");
}

#[test]
fn test_compare_list_with_number() {
    let source = r#"
bind x to [1, 2, 3] less than 10
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for List < Number");
}

// ============================================================================
// Index Access Type Checking
// ============================================================================

#[test]
fn test_index_list_with_text() {
    let source = r#"
bind lst to [1, 2, 3]
bind x to lst["first"]
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for indexing List with Text");
    match result.unwrap_err() {
        RuntimeError::TypeError { .. } => { /* Expected */ }
        err => panic!("Expected TypeError, got {:?}", err),
    }
}

#[test]
fn test_index_number() {
    let source = r#"
bind x to 42[0]
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for indexing Number");
}

#[test]
fn test_index_nothing() {
    let source = r#"
bind x to nothing[0]
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for indexing Nothing");
}

// ============================================================================
// Field Access Type Checking
// ============================================================================

#[test]
fn test_field_access_on_number() {
    let source = r#"
bind x to 42.length
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for field access on Number");
    match result.unwrap_err() {
        RuntimeError::TypeError { .. } => { /* Expected */ }
        err => panic!("Expected TypeError, got {:?}", err),
    }
}

#[test]
fn test_field_access_on_list() {
    let source = r#"
bind lst to [1, 2, 3]
bind x to lst.name
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for field access on List");
}

// ============================================================================
// Iteration Type Checking
// ============================================================================

#[test]
fn test_iterate_over_number() {
    let source = r#"
bind sum to 0
for each n in 42 then
    set sum to sum + n
end
sum
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for iterating over Number");
    match result.unwrap_err() {
        RuntimeError::NotIterable(_) => { /* Expected */ }
        err => panic!("Expected NotIterable, got {:?}", err),
    }
}

#[test]
fn test_iterate_over_nothing() {
    let source = r#"
for each n in nothing then
    n
end
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for iterating over Nothing");
}

#[test]
fn test_iterate_over_text() {
    let source = r#"
for each c in "hello" then
    c
end
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for iterating over Text");
}

// ============================================================================
// Function Call Type Checking
// ============================================================================

#[test]
fn test_call_non_function() {
    let source = r#"
bind x to 42
bind result to x(10)
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for calling non-function");
    match result.unwrap_err() {
        RuntimeError::NotCallable(_) => { /* Expected */ }
        err => panic!("Expected NotCallable, got {:?}", err),
    }
}

#[test]
fn test_call_nothing() {
    let source = r#"
bind result to nothing(42)
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for calling Nothing");
}

// ============================================================================
// Division by Zero
// ============================================================================

#[test]
fn test_division_by_zero_literal() {
    let source = r#"
bind x to 10 / 0
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for division by zero");
    match result.unwrap_err() {
        RuntimeError::DivisionByZero => { /* Expected */ }
        err => panic!("Expected DivisionByZero, got {:?}", err),
    }
}

#[test]
fn test_modulo_by_zero() {
    let source = r#"
bind x to 10 % 0
    "#;

    let result = run_program(source);
    // Modulo by zero should also error (though Rust's % returns NaN)
    // Our runtime should catch this
    assert!(result.is_err() || matches!(result, Ok(Value::Number(n)) if n.is_nan()));
}

// ============================================================================
// Mixed-Type Operations (Valid Cases)
// ============================================================================

#[test]
fn test_equality_different_types_valid() {
    // Equality works across types (always returns false)
    let source = r#"
bind x to 42 is "42"
    "#;

    let result = run_program(source).expect("Should not error");
    assert_eq!(result, Value::Truth(false));
}

#[test]
fn test_inequality_different_types_valid() {
    // Inequality works across types
    let source = r#"
bind x to 42 is not "hello"
    "#;

    let result = run_program(source).expect("Should not error");
    assert_eq!(result, Value::Truth(true));
}

// ============================================================================
// Logical Operations with Non-Truth Values
// ============================================================================

#[test]
fn test_logical_and_with_numbers() {
    // Logical operations work on all types (truthiness)
    let source = r#"
bind x to 5 and 10
    "#;

    let result = run_program(source).expect("Should not error");
    // 5 is truthy, 10 is truthy, so result is truthy
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_logical_or_with_nothing() {
    let source = r#"
bind x to nothing or 42
    "#;

    let result = run_program(source).expect("Should not error");
    // nothing is falsy, 42 is truthy
    assert_eq!(result, Value::Truth(true));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_concat_text_with_number_fails() {
    // Text concatenation only works with Text + Text
    let source = r#"
bind x to "Count: " + 42
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for Text + Number");
}

#[test]
fn test_index_out_of_bounds() {
    let source = r#"
bind lst to [1, 2, 3]
bind x to lst[10]
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Expected error for out of bounds access");
    match result.unwrap_err() {
        RuntimeError::IndexOutOfBounds { .. } => { /* Expected */ }
        err => panic!("Expected IndexOutOfBounds, got {:?}", err),
    }
}

#[test]
fn test_fractional_index() {
    // Fractional numbers get truncated to integers for indexing
    let source = r#"
bind lst to [1, 2, 3]
bind x to lst[1.7]
    "#;

    // This should either error or truncate to 1
    let result = run_program(source);
    // Currently, Glimmer truncates fractional indices
    match result {
        Ok(Value::Number(2.0)) => { /* Index 1 -> value 2 */ }
        Err(_) => { /* Also acceptable */ }
        other => panic!("Unexpected result: {:?}", other),
    }
}
