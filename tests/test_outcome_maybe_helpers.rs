/// Tests for Outcome<T, E> and Maybe<T> helper functions with natural language branding
///
/// These tests verify the builtin helper functions for working with
/// Outcome and Maybe types in a natural, readable way.

use glimmer_weave::{Lexer, Parser, Evaluator};
use glimmer_weave::eval::Value;

fn eval_program(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut evaluator = Evaluator::new();
    evaluator.eval(&ast).map_err(|e| format!("Runtime error: {:?}", e))
}

// ============================================================================
// OUTCOME INSPECTION TESTS
// ============================================================================

#[test]
fn test_is_triumph_on_triumph() {
    let source = r#"
        bind result to Triumph(42)
        is_triumph(result)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_is_triumph_on_mishap() {
    let source = r#"
        bind result to Mishap("error")
        is_triumph(result)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(false));
}

#[test]
fn test_is_mishap_on_mishap() {
    let source = r#"
        bind result to Mishap("error")
        is_mishap(result)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_is_mishap_on_triumph() {
    let source = r#"
        bind result to Triumph(42)
        is_mishap(result)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(false));
}

// ============================================================================
// MAYBE INSPECTION TESTS
// ============================================================================

#[test]
fn test_is_present_on_present() {
    let source = r#"
        bind maybe to Present(42)
        is_present(maybe)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_is_present_on_absent() {
    let source = r#"
        bind maybe to Absent
        is_present(maybe)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(false));
}

#[test]
fn test_is_absent_on_absent() {
    let source = r#"
        bind maybe to Absent
        is_absent(maybe)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_is_absent_on_present() {
    let source = r#"
        bind maybe to Present(42)
        is_absent(maybe)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(false));
}

// ============================================================================
// OUTCOME EXTRACTION TESTS
// ============================================================================

#[test]
fn test_expect_triumph_success() {
    let source = r#"
        bind result to Triumph(42)
        expect_triumph(result, "Should have value")
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_expect_triumph_failure() {
    let source = r#"
        bind result to Mishap("error")
        expect_triumph(result, "Expected success")
    "#;

    let result = eval_program(source);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Expected success"));
}

#[test]
fn test_triumph_or_with_triumph() {
    let source = r#"
        bind result to Triumph(42)
        triumph_or(result, 0)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_triumph_or_with_mishap() {
    let source = r#"
        bind result to Mishap("error")
        triumph_or(result, 0)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_expect_mishap_success() {
    let source = r#"
        bind result to Mishap("error message")
        expect_mishap(result, "Should have error")
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Text("error message".to_string()));
}

#[test]
fn test_expect_mishap_failure() {
    let source = r#"
        bind result to Triumph(42)
        expect_mishap(result, "Expected failure")
    "#;

    let result = eval_program(source);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Expected failure"));
}

// ============================================================================
// MAYBE EXTRACTION TESTS
// ============================================================================

#[test]
fn test_expect_present_success() {
    let source = r#"
        bind maybe to Present(42)
        expect_present(maybe, "Should have value")
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_expect_present_failure() {
    let source = r#"
        bind maybe to Absent
        expect_present(maybe, "Expected value")
    "#;

    let result = eval_program(source);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Expected value"));
}

#[test]
fn test_present_or_with_present() {
    let source = r#"
        bind maybe to Present(42)
        present_or(maybe, 0)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_present_or_with_absent() {
    let source = r#"
        bind maybe to Absent
        present_or(maybe, 0)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(0.0));
}

// ============================================================================
// CONVERSION TESTS
// ============================================================================

#[test]
fn test_present_or_mishap_with_present() {
    let source = r#"
        bind maybe to Present(42)
        bind result to present_or_mishap(maybe, "No value")
        is_triumph(result)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_present_or_mishap_with_absent() {
    let source = r#"
        bind maybe to Absent
        bind result to present_or_mishap(maybe, "No value")
        is_mishap(result)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_present_or_mishap_extracts_value() {
    let source = r#"
        bind maybe to Present(42)
        bind result to present_or_mishap(maybe, "error")
        triumph_or(result, 0)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_present_or_mishap_extracts_error() {
    let source = r#"
        bind maybe to Absent
        bind result to present_or_mishap(maybe, "error message")
        expect_mishap(result, "Should be mishap")
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Text("error message".to_string()));
}

#[test]
fn test_triumph_or_absent_with_triumph() {
    let source = r#"
        bind result to Triumph(42)
        bind maybe to triumph_or_absent(result)
        is_present(maybe)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_triumph_or_absent_with_mishap() {
    let source = r#"
        bind result to Mishap("error")
        bind maybe to triumph_or_absent(result)
        is_absent(maybe)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_triumph_or_absent_extracts_value() {
    let source = r#"
        bind result to Triumph(42)
        bind maybe to triumph_or_absent(result)
        present_or(maybe, 0)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// COMBINATION TESTS
// ============================================================================

#[test]
fn test_both_triumph_with_two_triumphs() {
    let source = r#"
        bind r1 to Triumph(42)
        bind r2 to Triumph("hello")
        bind combined to both_triumph(r1, r2)
        is_triumph(combined)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_both_triumph_extracts_both_values() {
    let source = r#"
        bind r1 to Triumph(42)
        bind r2 to Triumph(24)
        bind combined to both_triumph(r1, r2)
        bind pair to triumph_or(combined, [])
        list_length(pair)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_both_triumph_first_mishap() {
    let source = r#"
        bind r1 to Mishap("error1")
        bind r2 to Triumph(42)
        bind combined to both_triumph(r1, r2)
        is_mishap(combined)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_both_triumph_second_mishap() {
    let source = r#"
        bind r1 to Triumph(42)
        bind r2 to Mishap("error2")
        bind combined to both_triumph(r1, r2)
        is_mishap(combined)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_both_triumph_returns_first_mishap() {
    let source = r#"
        bind r1 to Mishap("error1")
        bind r2 to Mishap("error2")
        bind combined to both_triumph(r1, r2)
        expect_mishap(combined, "Should be mishap")
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Text("error1".to_string()));
}

#[test]
fn test_either_triumph_first_succeeds() {
    let source = r#"
        bind r1 to Triumph(42)
        bind r2 to Triumph(24)
        bind result to either_triumph(r1, r2)
        triumph_or(result, 0)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_either_triumph_first_fails_second_succeeds() {
    let source = r#"
        bind r1 to Mishap("error")
        bind r2 to Triumph(24)
        bind result to either_triumph(r1, r2)
        triumph_or(result, 0)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(24.0));
}

#[test]
fn test_either_triumph_both_fail() {
    let source = r#"
        bind r1 to Mishap("error1")
        bind r2 to Mishap("error2")
        bind result to either_triumph(r1, r2)
        expect_mishap(result, "Both failed")
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Text("error2".to_string()));
}

// ============================================================================
// USAGE PATTERN TESTS
// ============================================================================

#[test]
fn test_chaining_conversions() {
    let source = r#"
        # Maybe -> Outcome -> Maybe
        bind maybe1 to Present(42)
        bind outcome to present_or_mishap(maybe1, "error")
        bind maybe2 to triumph_or_absent(outcome)
        present_or(maybe2, 0)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_outcome_with_default_fallback() {
    let source = r#"
        # Simulating a function that might fail
        bind result to Mishap("Not found")
        bind value to triumph_or(result, 99)
        value
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(99.0));
}

#[test]
fn test_combining_multiple_outcomes() {
    let source = r#"
        # Combine three outcomes (using nested both_triumph)
        bind r1 to Triumph(1)
        bind r2 to Triumph(2)
        bind r3 to Triumph(3)
        bind pair12 to both_triumph(r1, r2)
        bind combined to both_triumph(pair12, r3)
        is_triumph(combined)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Truth(true));
}

#[test]
fn test_fallback_chain_with_either() {
    let source = r#"
        # Try multiple sources with fallback
        bind primary to Mishap("primary failed")
        bind secondary to Mishap("secondary failed")
        bind tertiary to Triumph(42)
        bind temp to either_triumph(primary, secondary)
        bind result to either_triumph(temp, tertiary)
        triumph_or(result, 0)
    "#;

    let result = eval_program(source).expect("Eval failed");
    assert_eq!(result, Value::Number(42.0));
}
