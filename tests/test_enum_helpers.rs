/// Tests for user-defined enum (variant) types - Phase 4: Helper Methods
///
/// These tests verify the natural language helper functions for working with enums:
/// - is_variant: Check if value matches a variant
/// - expect_variant: Extract fields or panic
/// - variant_or: Extract fields or return default
/// - refine_variant: Transform variant if matches

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
// is_variant() Tests
// ============================================================================

#[test]
fn test_is_variant_matches() {
    let source = r#"
        variant Color then Red, Green, Blue end

        bind color to Red
        bind result to is_variant(color, "Red")
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Truth(b) => assert!(b, "Expected is_variant to return true"),
        _ => panic!("Expected Truth value"),
    }
}

#[test]
fn test_is_variant_does_not_match() {
    let source = r#"
        variant Color then Red, Green, Blue end

        bind color to Red
        bind result to is_variant(color, "Green")
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Truth(b) => assert!(!b, "Expected is_variant to return false"),
        _ => panic!("Expected Truth value"),
    }
}

#[test]
fn test_is_variant_with_data() {
    let source = r#"
        variant Message then
            Quit,
            Move(x: Number, y: Number)
        end

        bind msg to Move(10, 20)
        bind result to is_variant(msg, "Move")
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Truth(b) => assert!(b, "Expected is_variant to return true for Move"),
        _ => panic!("Expected Truth value"),
    }
}

#[test]
fn test_is_variant_type_error() {
    let source = r#"
        bind result to is_variant(42, "Red")
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_err(), "Should fail with type error");
    assert!(result.unwrap_err().contains("TypeError"));
}

// ============================================================================
// expect_variant() Tests
// ============================================================================

#[test]
fn test_expect_variant_successful() {
    let source = r#"
        variant Message then
            Write(text: Text)
        end

        bind msg to Write("Hello")
        bind fields to expect_variant(msg, "Write", "Expected Write variant")
    "#;

    let result = eval_and_get(source, "fields");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            match &items[0] {
                Value::Text(s) => assert_eq!(s, "Hello"),
                _ => panic!("Expected Text in fields"),
            }
        }
        _ => panic!("Expected List value"),
    }
}

#[test]
fn test_expect_variant_multiple_fields() {
    let source = r#"
        variant Point then
            Point2D(x: Number, y: Number)
        end

        bind p to Point2D(5, 10)
        bind coords to expect_variant(p, "Point2D", "Expected Point2D")
    "#;

    let result = eval_and_get(source, "coords");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
            match (&items[0], &items[1]) {
                (Value::Number(x), Value::Number(y)) => {
                    assert_eq!(*x, 5.0);
                    assert_eq!(*y, 10.0);
                }
                _ => panic!("Expected Numbers in fields"),
            }
        }
        _ => panic!("Expected List value"),
    }
}

#[test]
fn test_expect_variant_unit_variant() {
    let source = r#"
        variant Status then Active, Inactive end

        bind status to Active
        bind fields to expect_variant(status, "Active", "Expected Active")
    "#;

    let result = eval_and_get(source, "fields");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::List(items) => {
            assert_eq!(items.len(), 0, "Unit variant should have no fields");
        }
        _ => panic!("Expected List value"),
    }
}

#[test]
fn test_expect_variant_mismatch_error() {
    let source = r#"
        variant Color then Red, Green end

        bind color to Red
        bind result to expect_variant(color, "Green", "Wrong color!")
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_err(), "Should fail with variant mismatch");
    let error = result.unwrap_err();
    assert!(error.contains("Wrong color!"));
    assert!(error.contains("expected variant 'Green'"));
    assert!(error.contains("got 'Red'"));
}

#[test]
fn test_expect_variant_type_error() {
    let source = r#"
        bind result to expect_variant("text", "Red", "Error message")
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_err(), "Should fail with type error");
    assert!(result.unwrap_err().contains("TypeError"));
}

// ============================================================================
// variant_or() Tests
// ============================================================================

#[test]
fn test_variant_or_matches() {
    let source = r#"
        variant Option then
            Some(value: Number),
            Empty
        end

        bind opt to Some(42)
        bind result to variant_or(opt, "Some", 0)
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            match &items[0] {
                Value::Number(n) => assert_eq!(*n, 42.0),
                _ => panic!("Expected Number in list"),
            }
        }
        _ => panic!("Expected List value"),
    }
}

#[test]
fn test_variant_or_returns_default() {
    let source = r#"
        variant Option then
            Some(value: Number),
            Empty
        end

        bind opt to Empty
        bind result to variant_or(opt, "Some", 999)
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 999.0),
        _ => panic!("Expected Number (default value)"),
    }
}

#[test]
fn test_variant_or_with_list_default() {
    let source = r#"
        variant Data then
            Values(items: Text)
        end

        bind data to Values("test")
        bind result to variant_or(data, "Other", [1, 2, 3])
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::List(items) => {
            assert_eq!(items.len(), 3, "Should return default list");
        }
        _ => panic!("Expected List (default value)"),
    }
}

#[test]
fn test_variant_or_type_error() {
    let source = r#"
        bind result to variant_or(123, "Red", 0)
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_err(), "Should fail with type error");
    assert!(result.unwrap_err().contains("TypeError"));
}

// ============================================================================
// refine_variant() Tests
// ============================================================================

#[test]
fn test_refine_variant_matches() {
    let source = r#"
        variant Result then
            Success(value: Number),
            Failure
        end

        bind res to Success(42)

        chant double(fields) then
            yield fields
        end

        bind refined to refine_variant(res, "Success", double)
    "#;

    let result = eval_and_get(source, "refined");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Maybe { present, value } => {
            assert!(present, "Expected Present");
            match value {
                Some(boxed) => {
                    match *boxed {
                        Value::List(items) => {
                            assert_eq!(items.len(), 1);
                            match &items[0] {
                                Value::Number(n) => assert_eq!(*n, 42.0),
                                _ => panic!("Expected Number"),
                            }
                        }
                        _ => panic!("Expected List inside Maybe"),
                    }
                }
                None => panic!("Expected Some value"),
            }
        }
        _ => panic!("Expected Maybe value"),
    }
}

#[test]
fn test_refine_variant_does_not_match() {
    let source = r#"
        variant Result then
            Success(value: Number),
            Failure
        end

        bind res to Failure

        chant double(fields) then
            yield fields
        end

        bind refined to refine_variant(res, "Success", double)
    "#;

    let result = eval_and_get(source, "refined");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Maybe { present, .. } => {
            assert!(!present, "Expected Absent");
        }
        _ => panic!("Expected Maybe value"),
    }
}

#[test]
fn test_refine_variant_with_transformation() {
    let source = r#"
        variant Point then
            Point2D(x: Number, y: Number)
        end

        bind p to Point2D(3, 4)

        chant extract_coords(fields) then
            yield fields
        end

        bind result to refine_variant(p, "Point2D", extract_coords)
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Maybe { present, value } => {
            assert!(present);
            match value {
                Some(boxed) => {
                    match *boxed {
                        Value::List(items) => {
                            assert_eq!(items.len(), 2);
                            match (&items[0], &items[1]) {
                                (Value::Number(x), Value::Number(y)) => {
                                    assert_eq!(*x, 3.0);
                                    assert_eq!(*y, 4.0);
                                }
                                _ => panic!("Expected Numbers"),
                            }
                        }
                        _ => panic!("Expected List"),
                    }
                }
                None => panic!("Expected Some value"),
            }
        }
        _ => panic!("Expected Maybe value"),
    }
}

#[test]
fn test_refine_variant_type_error_not_variant() {
    let source = r#"
        chant dummy(x) then yield x end

        bind result to refine_variant("text", "Red", dummy)
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_err(), "Should fail with type error");
    assert!(result.unwrap_err().contains("TypeError"));
}

#[test]
fn test_refine_variant_type_error_not_function() {
    let source = r#"
        variant Color then Red end

        bind color to Red
        bind result to refine_variant(color, "Red", 42)
    "#;

    let result = eval_and_get(source, "result");
    assert!(result.is_err(), "Should fail with type error");
    assert!(result.unwrap_err().contains("TypeError"));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_helpers_with_generic_enums() {
    let source = r#"
        variant Option<T> then
            Some(value: T),
            Empty
        end

        bind maybe_num to Some<Number>(100)

        bind is_some to is_variant(maybe_num, "Some")
        bind fields to variant_or(maybe_num, "Some", [0])
    "#;

    let is_some = eval_and_get(source, "is_some").unwrap();
    match is_some {
        Value::Truth(b) => assert!(b),
        _ => panic!("Expected Truth"),
    }

    let fields = eval_and_get(source, "fields").unwrap();
    match fields {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            match &items[0] {
                Value::Number(n) => assert_eq!(*n, 100.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_helpers_in_function() {
    let source = r#"
        variant Status then Ready, Busy end

        chant check_status(s) then
            match is_variant(s, "Ready") with
                when true then yield "All good"
                when false then yield "Not ready"
            end
        end

        bind status to Ready
        bind message to check_status(status)
    "#;

    let result = eval_and_get(source, "message");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Text(s) => assert_eq!(s, "All good"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_expect_variant_used_in_function() {
    let source = r#"
        variant Pair then
            Tuple(a: Number, b: Number)
        end

        bind pair to Tuple(7, 13)
        bind fields to expect_variant(pair, "Tuple", "Expected Tuple")

        # Verify we got a list with the correct values
        bind count to list_length(fields)
        bind head to list_first(fields)
    "#;

    let result = eval_and_get(source, "count");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 2.0),
        _ => panic!("Expected Number"),
    }

    let result = eval_and_get(source, "head");
    assert!(result.is_ok(), "Failed: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 7.0),
        _ => panic!("Expected Number"),
    }
}
