/// Tests for loop control flow: break and continue statements
///
/// These tests verify that break and continue work correctly in:
/// - for each loops
/// - whilst loops
/// - nested loops
/// - error cases (outside loops)

use glimmer_weave::{Lexer, Parser, Evaluator, Value};

/// Helper function to evaluate source code and return the final value
fn eval_program(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
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
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut evaluator = Evaluator::new();
    for node in &ast {
        evaluator.eval_node(node).map_err(|e| format!("Runtime error: {:?}", e))?;
    }

    evaluator.environment().get(var_name).map_err(|e| format!("{:?}", e))
}

// ============================================================================
// Break in for each loops
// ============================================================================

#[test]
fn test_break_in_for_each_simple() {
    let source = r#"
        weave count as 0

        for each i in [1, 2, 3, 4, 5] then
            set count to count + 1
            should i is 3 then
                break
            end
        end
    "#;

    let result = eval_and_get(source, "count");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 3.0, "Loop should have broken after 3 iterations"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_break_exits_immediately() {
    let source = r#"
        weave sum as 0

        for each i in [1, 2, 3, 4, 5] then
            should i is 3 then
                break
            end
            set sum to sum + i
        end
    "#;

    let result = eval_and_get(source, "sum");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 3.0, "Sum should be 1+2 = 3 (3 not added due to break)"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_break_in_for_each_range() {
    let source = r#"
        weave final_value as 0

        for each i in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] then
            set final_value to i
            should i is 5 then
                break
            end
        end
    "#;

    let result = eval_and_get(source, "final_value");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 5.0, "Loop should break at 5"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_break_in_for_each_first_iteration() {
    let source = r#"
        weave executed as false

        for each i in [1, 2, 3] then
            break
            set executed to true
        end
    "#;

    let result = eval_and_get(source, "executed");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Truth(b) => assert!(!b, "Code after break should not execute"),
        _ => panic!("Expected Truth value"),
    }
}

// ============================================================================
// Break in whilst loops
// ============================================================================

#[test]
fn test_break_in_whilst_simple() {
    let source = r#"
        weave count as 0

        whilst true then
            set count to count + 1
            should count is 5 then
                break
            end
        end
    "#;

    let result = eval_and_get(source, "count");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 5.0, "Loop should break after count reaches 5"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_break_prevents_infinite_loop() {
    let source = r#"
        weave i as 0

        whilst true then
            set i to i + 1
            should i greater than 100 then
                break
            end
        end
    "#;

    let result = eval_and_get(source, "i");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 101.0, "Break should exit infinite loop"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_break_in_whilst_with_condition() {
    let source = r#"
        weave total as 0
        weave i as 1

        whilst i less than 20 then
            set total to total + i
            set i to i + 1
            should total greater than 10 then
                break
            end
        end
    "#;

    let result = eval_and_get(source, "total");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert!(n > 10.0 && n < 20.0, "Should break when total > 10"),
        _ => panic!("Expected Number value"),
    }
}

// ============================================================================
// Continue in for each loops
// ============================================================================

#[test]
fn test_continue_in_for_each_simple() {
    let source = r#"
        weave sum as 0

        for each i in [1, 2, 3, 4, 5] then
            should i is 3 then
                continue
            end
            set sum to sum + i
        end
    "#;

    let result = eval_and_get(source, "sum");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 12.0, "Sum should be 1+2+4+5 = 12 (3 skipped)"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_continue_skips_to_next_iteration() {
    let source = r#"
        weave evens as 0
        weave odds as 0

        for each i in [1, 2, 3, 4, 5, 6] then
            bind remainder to i % 2
            should remainder is 1 then
                set odds to odds + 1
                continue
            end
            set evens to evens + 1
        end
    "#;

    let evens = eval_and_get(source, "evens");
    assert!(evens.is_ok(), "Failed: {:?}", evens);
    match evens.unwrap() {
        Value::Number(n) => assert_eq!(n, 3.0, "Should count 3 even numbers"),
        _ => panic!("Expected Number value"),
    }

    let odds = eval_and_get(source, "odds");
    assert!(odds.is_ok(), "Failed: {:?}", odds);
    match odds.unwrap() {
        Value::Number(n) => assert_eq!(n, 3.0, "Should count 3 odd numbers"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_continue_all_iterations() {
    let source = r#"
        weave count as 0

        for each i in [1, 2, 3] then
            set count to count + 1
            continue
        end
    "#;

    let result = eval_and_get(source, "count");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 3.0, "All iterations should execute"),
        _ => panic!("Expected Number value"),
    }
}

// ============================================================================
// Continue in whilst loops
// ============================================================================

#[test]
fn test_continue_in_whilst_simple() {
    let source = r#"
        weave sum as 0
        weave i as 0

        whilst i less than 6 then
            set i to i + 1
            should i is 3 then
                continue
            end
            set sum to sum + i
        end
    "#;

    let result = eval_and_get(source, "sum");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 18.0, "Sum should be 1+2+4+5+6 = 18 (3 skipped)"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_continue_reevaluates_condition() {
    let source = r#"
        weave count as 0
        weave iterations as 0

        whilst count less than 5 then
            set iterations to iterations + 1
            set count to count + 1
            should count is 2 then
                continue
            end
        end
    "#;

    let result = eval_and_get(source, "iterations");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 5.0, "Should iterate 5 times despite continue"),
        _ => panic!("Expected Number value"),
    }
}

// ============================================================================
// Nested loops
// ============================================================================

#[test]
fn test_break_only_inner_loop() {
    let source = r#"
        weave outer_count as 0
        weave inner_count as 0

        for each i in [1, 2, 3] then
            set outer_count to outer_count + 1
            for each j in [1, 2, 3, 4, 5] then
                set inner_count to inner_count + 1
                should j is 3 then
                    break
                end
            end
        end
    "#;

    let outer = eval_and_get(source, "outer_count");
    assert!(outer.is_ok(), "Failed: {:?}", outer);
    match outer.unwrap() {
        Value::Number(n) => assert_eq!(n, 3.0, "Outer loop should complete all iterations"),
        _ => panic!("Expected Number value"),
    }

    let inner = eval_and_get(source, "inner_count");
    assert!(inner.is_ok(), "Failed: {:?}", inner);
    match inner.unwrap() {
        Value::Number(n) => assert_eq!(n, 9.0, "Inner loop breaks at 3, executes 3 times per outer = 9"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_continue_only_inner_loop() {
    let source = r#"
        weave sum as 0

        for each i in [1, 2] then
            for each j in [1, 2, 3] then
                should j is 2 then
                    continue
                end
                set sum to sum + j
            end
        end
    "#;

    let result = eval_and_get(source, "sum");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 8.0, "Sum should be 2*(1+3) = 8 (2 skipped each time)"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_nested_whilst_with_break() {
    let source = r#"
        weave i as 0
        weave total as 0

        whilst i less than 3 then
            set i to i + 1
            weave j as 0
            whilst j less than 10 then
                set j to j + 1
                set total to total + 1
                should j is 2 then
                    break
                end
            end
        end
    "#;

    let result = eval_and_get(source, "total");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 6.0, "Inner loop breaks at 2, runs 3 times = 6"),
        _ => panic!("Expected Number value"),
    }
}

// ============================================================================
// Error cases - break/continue outside loops
// ============================================================================

#[test]
fn test_break_outside_loop_error() {
    let source = r#"
        weave x as 5
        break
    "#;

    let result = eval_program(source);
    assert!(result.is_err(), "Should fail with break outside loop");
    let error = result.unwrap_err();
    assert!(error.contains("BreakOutsideLoop"), "Error message: {}", error);
}

#[test]
fn test_continue_outside_loop_error() {
    let source = r#"
        weave x as 5
        continue
    "#;

    let result = eval_program(source);
    assert!(result.is_err(), "Should fail with continue outside loop");
    let error = result.unwrap_err();
    assert!(error.contains("ContinueOutsideLoop"), "Error message: {}", error);
}

#[test]
fn test_break_in_function_outside_loop() {
    let source = r#"
        chant test() then
            break
        end

        bind result to test()
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_err(), "Break in function outside loop should error");
    let error = result.unwrap_err();
    assert!(error.contains("BreakOutsideLoop"), "Error message: {}", error);
}

#[test]
fn test_continue_in_conditional_outside_loop() {
    let source = r#"
        should true then
            continue
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_err(), "Continue in conditional outside loop should error");
    let error = result.unwrap_err();
    assert!(error.contains("ContinueOutsideLoop"), "Error message: {}", error);
}

// ============================================================================
// Complex scenarios
// ============================================================================

#[test]
fn test_break_with_pattern_matching() {
    let source = r#"
        variant Action then Stop, Continue, Process end

        weave count as 0

        for each action in [Process, Process, Stop, Process] then
            match action with
                when Stop then break
                when Process then set count to count + 1
                when Continue then continue
            end
        end
    "#;

    let result = eval_and_get(source, "count");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 2.0, "Should process 2 items before Stop"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_continue_with_multiple_conditions() {
    let source = r#"
        weave valid_count as 0

        for each i in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] then
            # Skip multiples of 2
            bind mod2 to i % 2
            should mod2 is 0 then
                continue
            end

            # Skip multiples of 3
            bind mod3 to i % 3
            should mod3 is 0 then
                continue
            end

            set valid_count to valid_count + 1
        end
    "#;

    let result = eval_and_get(source, "valid_count");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 3.0, "Should count 1,5,7 (3 numbers not divisible by 2 or 3)"),
        _ => panic!("Expected Number value"),
    }
}

#[test]
fn test_break_returns_last_value() {
    let source = r#"
        weave result as 0

        for each i in [10, 20, 30, 40] then
            set result to i
            should i is 30 then
                break
            end
        end
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 30.0, "Result should be last value before break"),
        _ => panic!("Expected Number value"),
    }
}
