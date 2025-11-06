/// Comprehensive tests for the Pipeline operator
/// Tests the `|` operator for threading values through function calls

use glimmer_weave::{Evaluator, Lexer, Parser};
use glimmer_weave::eval::{RuntimeError, Value};

fn run_program(source: &str) -> Result<Value, RuntimeError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| RuntimeError::Custom(format!("Parse error: {:?}", e)))?;

    let mut evaluator = Evaluator::new();
    evaluator.eval(&ast)
}

// ============================================================================
// Basic Pipeline Tests
// ============================================================================

#[test]
fn test_pipeline_simple_value() {
    let source = r#"
        chant double(x) then
            yield x * 2
        end

        5 | double
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_pipeline_two_functions() {
    let source = r#"
        chant double(x) then
            yield x * 2
        end

        chant add_one(x) then
            yield x + 1
        end

        5 | double | add_one
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(11.0));  // (5 * 2) + 1 = 11
}

#[test]
fn test_pipeline_three_functions() {
    let source = r#"
        chant double(x) then
            yield x * 2
        end

        chant add_one(x) then
            yield x + 1
        end

        chant square(x) then
            yield x * x
        end

        3 | double | add_one | square
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(49.0));  // ((3 * 2) + 1)^2 = 7^2 = 49
}

// ============================================================================
// Pipeline with Function Calls (Additional Arguments)
// ============================================================================

#[test]
fn test_pipeline_with_extra_arguments() {
    let source = r#"
        chant add(a, b) then
            yield a + b
        end

        chant multiply(a, b) then
            yield a * b
        end

        5 | add(10) | multiply(2)
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(30.0));  // (5 + 10) * 2 = 30
}

#[test]
fn test_pipeline_multiple_arguments() {
    let source = r#"
        chant add_three(a, b, c) then
            yield a + b + c
        end

        10 | add_three(20, 30)
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(60.0));  // 10 + 20 + 30 = 60
}

// ============================================================================
// Pipeline with Lists
// ============================================================================

#[test]
fn test_pipeline_list_operations() {
    let source = r#"
        chant double_list(lst) then
            weave result as []
            for each item in lst then
                set result to list_push(result, item * 2)
            end
            yield result
        end

        chant sum_list(lst) then
            weave total as 0
            for each item in lst then
                set total to total + item
            end
            yield total
        end

        [1, 2, 3, 4, 5] | double_list | sum_list
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(30.0));  // (1+2+3+4+5)*2 = 30
}

// ============================================================================
// Pipeline with Native Functions
// ============================================================================

#[test]
fn test_pipeline_with_native_functions() {
    let source = r#"
        [1, 2, 3] | list_length
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_pipeline_native_and_user_functions() {
    let source = r#"
        chant double(x) then
            yield x * 2
        end

        [1, 2, 3] | list_length | double
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(6.0));  // len([1,2,3]) * 2 = 6
}

// ============================================================================
// Pipeline with Expressions
// ============================================================================

#[test]
fn test_pipeline_expression_as_first_stage() {
    let source = r#"
        chant double(x) then
            yield x * 2
        end

        (3 + 2) | double
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(10.0));  // (3 + 2) * 2 = 10
}

#[test]
fn test_pipeline_with_variables() {
    let source = r#"
        chant double(x) then
            yield x * 2
        end

        bind value to 7
        value | double
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(14.0));
}

// ============================================================================
// Pipeline with Complex Data Types
// ============================================================================

#[test]
fn test_pipeline_with_structs() {
    let source = r#"
        form Person with
            name as Text
            age as Number
        end

        chant get_age(person) then
            yield person.age
        end

        chant double(x) then
            yield x * 2
        end

        bind alice to Person { name: "Alice", age: 30 }
        alice | get_age | double
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(60.0));  // 30 * 2 = 60
}

#[test]
fn test_pipeline_with_maps() {
    let source = r#"
        chant get_value(map) then
            yield map["value"]
        end

        chant triple(x) then
            yield x * 3
        end

        bind data to { value: 10 }
        data | get_value | triple
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(30.0));  // 10 * 3 = 30
}

// ============================================================================
// Pipeline Error Cases
// ============================================================================

#[test]
fn test_pipeline_empty() {
    let source = r#"
        # This should fail at parse time, but if it doesn't, runtime should handle it
        5 |
    "#;

    // This will likely fail at parse time, which is fine
    let result = run_program(source);
    assert!(result.is_err());
}

#[test]
fn test_pipeline_arity_mismatch() {
    let source = r#"
        chant add_two(a, b) then
            yield a + b
        end

        5 | add_two
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Should fail with arity mismatch");
    match result.unwrap_err() {
        RuntimeError::ArityMismatch { expected, got } => {
            assert_eq!(expected, 2);
            assert_eq!(got, 1);
        }
        err => panic!("Expected ArityMismatch, got {:?}", err),
    }
}

#[test]
fn test_pipeline_not_callable() {
    let source = r#"
        bind not_a_function to 42
        5 | not_a_function
    "#;

    let result = run_program(source);
    assert!(result.is_err(), "Should fail when piping to non-function");
    match result.unwrap_err() {
        RuntimeError::NotCallable(_) => { /* Expected */ }
        err => panic!("Expected NotCallable, got {:?}", err),
    }
}

// ============================================================================
// Pipeline with Pattern Matching
// ============================================================================

#[test]
fn test_pipeline_with_maybe_types() {
    let source = r#"
        chant find_positive(x) then
            should x greater than 0 then
                yield Present(x)
            otherwise
                yield Absent
            end
        end

        chant double_maybe(maybe_val) then
            match maybe_val with
                when Present(x) then yield Present(x * 2)
                when Absent then yield Absent
            end
        end

        5 | find_positive | double_maybe
    "#;

    let result = run_program(source).expect("Should succeed");
    // Result should be Present(10) - represented as Maybe type in runtime
    match result {
        Value::Maybe { present, value } => {
            assert!(present, "Expected Present, got Absent");
            assert_eq!(value, Some(Box::new(Value::Number(10.0))));
        }
        _ => panic!("Expected Maybe Present(10), got {:?}", result),
    }
}

// ============================================================================
// Pipeline with Recursion
// ============================================================================

#[test]
fn test_pipeline_with_recursive_functions() {
    let source = r#"
        chant factorial(n) then
            should n <= 1 then
                yield 1
            otherwise
                yield n * factorial(n - 1)
            end
        end

        chant add_one(x) then
            yield x + 1
        end

        4 | add_one | factorial
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(120.0));  // 5! = 120
}

// ============================================================================
// Pipeline Performance/Stress Tests
// ============================================================================

#[test]
fn test_pipeline_long_chain() {
    let source = r#"
        chant add_one(x) then
            yield x + 1
        end

        0 | add_one | add_one | add_one | add_one | add_one | add_one | add_one | add_one | add_one | add_one
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(10.0));
}

// ============================================================================
// Mixed Pipeline Styles
// ============================================================================

#[test]
fn test_pipeline_mixed_identifiers_and_calls() {
    let source = r#"
        chant double(x) then
            yield x * 2
        end

        chant add(a, b) then
            yield a + b
        end

        5 | double | add(3) | double
    "#;

    let result = run_program(source).expect("Should succeed");
    assert_eq!(result, Value::Number(26.0));  // ((5 * 2) + 3) * 2 = 26
}

#[test]
fn test_pipeline_complex_real_world_example() {
    let source = r#"
        # Data transformation pipeline
        chant filter_positive(lst) then
            weave result as []
            for each item in lst then
                should item greater than 0 then
                    set result to list_push(result, item)
                end
            end
            yield result
        end

        chant double_all(lst) then
            weave result as []
            for each item in lst then
                set result to list_push(result, item * 2)
            end
            yield result
        end

        chant sum(lst) then
            weave total as 0
            for each item in lst then
                set total to total + item
            end
            yield total
        end

        [-2, -1, 0, 1, 2, 3, 4, 5] | filter_positive | double_all | sum
    "#;

    let result = run_program(source).expect("Should succeed");
    // filter_positive: [1, 2, 3, 4, 5]
    // double_all: [2, 4, 6, 8, 10]
    // sum: 30
    assert_eq!(result, Value::Number(30.0));
}
