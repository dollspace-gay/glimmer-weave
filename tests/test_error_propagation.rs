/// Tests for error propagation operator (?)
///
/// These tests verify that the `?` operator correctly propagates Mishap values
/// and unwraps Triumph values in the Outcome type.

use glimmer_weave::{Lexer, Parser, Evaluator, Value};

/// Helper function to evaluate source code and return the final value
fn eval_program(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut evaluator = Evaluator::new();
    let mut result = Value::Nothing;
    for node in &ast {
        result = evaluator.eval_node(node).map_err(|e| format!("Runtime error: {:?}", e))?;
    }

    Ok(result)
}

/// Helper function to evaluate source code and get a variable's value
fn eval_and_get(source: &str, var_name: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut evaluator = Evaluator::new();
    for node in &ast {
        evaluator.eval_node(node).map_err(|e| format!("Runtime error: {:?}", e))?;
    }

    evaluator.environment().get(var_name).map_err(|e| format!("{:?}", e))
}

// ============================================================================
// Basic ? on Triumph (success case)
// ============================================================================

#[test]
fn test_try_on_triumph_unwraps_value() {
    let source = r#"
        chant get_value() then
            yield Triumph(42)
        end

        chant use_value() then
            bind x to get_value()?
            yield Triumph(x * 2)
        end

        bind result to use_value()
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Outcome { success, value } => {
            assert!(success, "Expected Triumph");
            match *value {
                Value::Number(n) => assert_eq!(n, 84.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected Outcome"),
    }
}

#[test]
fn test_try_unwraps_to_usable_value() {
    let source = r#"
        chant test() then
            bind val to Triumph(10)?
            yield Triumph(val + 5)
        end

        bind result to test()
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Outcome { success, value } => {
            assert!(success);
            match *value {
                Value::Number(n) => assert_eq!(n, 15.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected Outcome"),
    }
}

// ============================================================================
// Basic ? on Mishap (error propagation)
// ============================================================================

#[test]
fn test_try_on_mishap_propagates_error() {
    let source = r#"
        chant get_error() then
            yield Mishap("Something went wrong")
        end

        chant use_value() then
            bind x to get_error()?
            yield Triumph(x * 2)  # This should not execute
        end

        bind result to use_value()
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Outcome { success, value } => {
            assert!(!success, "Expected Mishap");
            match *value {
                Value::Text(ref s) => assert_eq!(s, "Something went wrong"),
                _ => panic!("Expected Text error"),
            }
        }
        _ => panic!("Expected Outcome"),
    }
}

#[test]
fn test_try_stops_execution_on_mishap() {
    let source = r#"
        weave side_effect as 0

        chant test() then
            bind x to Mishap("error")?
            set side_effect to 99  # Should not execute
            yield Triumph(x)
        end

        bind result to test()
    "#;

    let result = eval_and_get(source, "side_effect");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 0.0, "Side effect should not have executed"),
        _ => panic!("Expected Number"),
    }
}

// ============================================================================
// Chained ? operations
// ============================================================================

#[test]
fn test_chained_try_all_success() {
    let source = r#"
        chant step1() then yield Triumph(10) end
        chant step2(x) then yield Triumph(x + 5) end
        chant step3(x) then yield Triumph(x * 2) end

        chant process() then
            bind a to step1()?
            bind b to step2(a)?
            bind c to step3(b)?
            yield Triumph(c)
        end

        bind result to process()
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Outcome { success, value } => {
            assert!(success);
            match *value {
                Value::Number(n) => assert_eq!(n, 30.0), // (10 + 5) * 2 = 30
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected Outcome"),
    }
}

#[test]
fn test_chained_try_stops_at_first_mishap() {
    let source = r#"
        chant step1() then yield Triumph(10) end
        chant step2(x) then yield Mishap("step2 failed") end
        chant step3(x) then yield Triumph(x * 2) end

        chant process() then
            bind a to step1()?
            bind b to step2(a)?
            bind c to step3(b)?  # Should not execute
            yield Triumph(c)
        end

        bind result to process()
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Outcome { success, value } => {
            assert!(!success, "Expected Mishap");
            match *value {
                Value::Text(ref s) => assert_eq!(s, "step2 failed"),
                _ => panic!("Expected Text error"),
            }
        }
        _ => panic!("Expected Outcome"),
    }
}

// ============================================================================
// ? in expressions
// ============================================================================

#[test]
fn test_try_in_arithmetic_expression() {
    let source = r#"
        chant get_num() then yield Triumph(5) end

        chant calc() then
            bind result to get_num()? + 10
            yield Triumph(result)
        end

        bind result to calc()
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Outcome { success, value } => {
            assert!(success);
            match *value {
                Value::Number(n) => assert_eq!(n, 15.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected Outcome"),
    }
}

#[test]
fn test_try_in_function_call_argument() {
    let source = r#"
        chant get_value() then yield Triumph(7) end
        chant double(x) then yield Triumph(x * 2) end

        chant process() then
            bind result to double(get_value()?)
            yield result
        end

        bind result to process()
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Outcome { success, value } => {
            assert!(success);
            match *value {
                Value::Number(n) => assert_eq!(n, 14.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected Outcome"),
    }
}

// ============================================================================
// Error: ? on non-Outcome value
// ============================================================================

#[test]
fn test_try_on_non_outcome_is_error() {
    let source = r#"
        chant test() then
            bind x to 42?  # Error: can't use ? on Number
            yield Triumph(x)
        end

        bind result to test()
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_err(), "Should fail with type error");
    let error = result.unwrap_err();
    assert!(error.contains("TypeError"), "Error message: {}", error);
}

#[test]
fn test_try_on_text_is_error() {
    let source = r#"
        chant test() then
            bind x to "hello"?
            yield Triumph(x)
        end

        bind result to test()
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_err(), "Should fail with type error");
    let error = result.unwrap_err();
    assert!(error.contains("TypeError"), "Error message: {}", error);
}

// ============================================================================
// Integration with pattern matching
// ============================================================================

#[test]
fn test_try_with_match_expression() {
    let source = r#"
        variant Status then Ready, NotReady end

        chant get_status() then
            yield Triumph(Ready)
        end

        chant process() then
            bind status to get_status()?
            match status with
                when Ready then yield Triumph("All good")
                when NotReady then yield Mishap("Not ready")
            end
        end

        bind result to process()
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Outcome { success, value } => {
            assert!(success);
            match *value {
                Value::Text(ref s) => assert_eq!(s, "All good"),
                _ => panic!("Expected Text"),
            }
        }
        _ => panic!("Expected Outcome"),
    }
}

// ============================================================================
// Integration with conditionals
// ============================================================================

#[test]
fn test_try_inside_conditional() {
    let source = r#"
        chant get_value(should_succeed) then
            should should_succeed then
                yield Triumph(42)
            otherwise
                yield Mishap("Failed")
            end
        end

        chant process(flag) then
            should flag then
                bind x to get_value(flag)?
                yield Triumph(x)
            otherwise
                yield Triumph(0)
            end
        end

        bind result1 to process(true)
        bind result2 to process(false)
    "#;

    let result1 = eval_and_get(source, "result1");
    assert!(result1.is_ok(), "Failed: {:?}", result1);
    match result1.unwrap() {
        Value::Outcome { success, value } => {
            assert!(success);
            match *value {
                Value::Number(n) => assert_eq!(n, 42.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected Outcome"),
    }

    let result2 = eval_and_get(source, "result2");
    assert!(result2.is_ok(), "Failed: {:?}", result2);
    match result2.unwrap() {
        Value::Outcome { success, value } => {
            assert!(success, "Expected Triumph(0) from otherwise branch");
            match *value {
                Value::Number(n) => assert_eq!(n, 0.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected Outcome"),
    }
}

// ============================================================================
// Integration with loops
// ============================================================================

#[test]
fn test_try_inside_loop() {
    let source = r#"
        chant safe_divide(a, b) then
            should b is 0 then
                yield Mishap("Division by zero")
            otherwise
                yield Triumph(a / b)
            end
        end

        chant process_first(nums) then
            for each n in nums then
                bind result to safe_divide(10, n)?
                yield Triumph(result)  # Return first successful result
            end

            yield Mishap("Empty list")
        end

        bind success_case to process_first([2, 5, 1])
        bind fail_case to process_first([0, 2, 1])  # Zero first, so it fails immediately
    "#;

    let success = eval_and_get(source, "success_case");
    assert!(success.is_ok(), "Failed: {:?}", success);
    match success.unwrap() {
        Value::Outcome { success, value } => {
            assert!(success);
            match *value {
                Value::Number(n) => assert_eq!(n, 5.0), // 10 / 2 = 5
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected Outcome"),
    }

    let fail = eval_and_get(source, "fail_case");
    assert!(fail.is_ok(), "Failed: {:?}", fail);
    match fail.unwrap() {
        Value::Outcome { success, .. } => {
            assert!(!success, "Expected Mishap when dividing by zero");
        }
        _ => panic!("Expected Outcome"),
    }
}

// ============================================================================
// Complex scenarios
// ============================================================================

#[test]
fn test_nested_functions_with_try() {
    let source = r#"
        chant inner(x) then
            should x less than 0 then
                yield Mishap("Negative input")
            otherwise
                yield Triumph(x * x)
            end
        end

        chant middle(x) then
            bind squared to inner(x)?
            yield Triumph(squared + 10)
        end

        chant outer(x) then
            bind result to middle(x)?
            yield Triumph(result * 2)
        end

        bind success to outer(5)
        bind failure to outer(-3)
    "#;

    let success = eval_and_get(source, "success");
    assert!(success.is_ok(), "Failed: {:?}", success);
    match success.unwrap() {
        Value::Outcome { success, value } => {
            assert!(success);
            match *value {
                Value::Number(n) => assert_eq!(n, 70.0), // ((5*5) + 10) * 2 = 70
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected Outcome"),
    }

    let failure = eval_and_get(source, "failure");
    assert!(failure.is_ok(), "Failed: {:?}", failure);
    match failure.unwrap() {
        Value::Outcome { success, value } => {
            assert!(!success);
            match *value {
                Value::Text(ref s) => assert_eq!(s, "Negative input"),
                _ => panic!("Expected Text"),
            }
        }
        _ => panic!("Expected Outcome"),
    }
}

#[test]
fn test_try_with_different_error_types() {
    let source = r#"
        chant fail_with_text() then
            yield Mishap("Text error")
        end

        chant fail_with_number() then
            yield Mishap(404)
        end

        chant process1() then
            bind x to fail_with_text()?
            yield Triumph(x)
        end

        chant process2() then
            bind x to fail_with_number()?
            yield Triumph(x)
        end

        bind err1 to process1()
        bind err2 to process2()
    "#;

    let err1 = eval_and_get(source, "err1");
    assert!(err1.is_ok(), "Failed: {:?}", err1);
    match err1.unwrap() {
        Value::Outcome { success, value } => {
            assert!(!success);
            match *value {
                Value::Text(ref s) => assert_eq!(s, "Text error"),
                _ => panic!("Expected Text error"),
            }
        }
        _ => panic!("Expected Outcome"),
    }

    let err2 = eval_and_get(source, "err2");
    assert!(err2.is_ok(), "Failed: {:?}", err2);
    match err2.unwrap() {
        Value::Outcome { success, value } => {
            assert!(!success);
            match *value {
                Value::Number(n) => assert_eq!(n, 404.0),
                _ => panic!("Expected Number error"),
            }
        }
        _ => panic!("Expected Outcome"),
    }
}
