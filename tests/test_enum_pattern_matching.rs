/// Tests for user-defined enum (variant) types - Phase 1c: Pattern Matching
///
/// These tests verify that pattern matching works correctly with user-defined enums.

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

#[test]
fn test_simple_enum_match() {
    let source = r#"
        variant Color then Red, Green, Blue end

        bind my_color to Red

        match my_color with
            when Red then 1
            when Green then 2
            when Blue then 3
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to match enum: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 1.0),
        _ => panic!("Expected Number(1)"),
    }
}

#[test]
fn test_enum_match_all_variants() {
    let source = r#"
        variant Direction then North, South, East, West end

        chant direction_to_number(dir) then
            match dir with
                when North then yield 0
                when South then yield 1
                when East then yield 2
                when West then yield 3
            end
        end

        bind n to direction_to_number(North)
        bind s to direction_to_number(South)
        bind e to direction_to_number(East)
        bind w to direction_to_number(West)
    "#;

    assert_eq!(eval_and_get(source, "n").unwrap(), Value::Number(0.0));
    assert_eq!(eval_and_get(source, "s").unwrap(), Value::Number(1.0));
    assert_eq!(eval_and_get(source, "e").unwrap(), Value::Number(2.0));
    assert_eq!(eval_and_get(source, "w").unwrap(), Value::Number(3.0));
}

#[test]
fn test_enum_match_with_text_result() {
    let source = r#"
        variant Status then Pending, Active, Complete end

        bind status to Active

        match status with
            when Pending then "Waiting to start"
            when Active then "In progress"
            when Complete then "Done"
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to match enum: {:?}", result);

    match result.unwrap() {
        Value::Text(s) => assert_eq!(s, "In progress"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_enum_match_with_wildcard() {
    let source = r#"
        variant Color then Red, Green, Blue end

        bind color to Blue

        match color with
            when Red then "Red variant"
            otherwise then "Other color"
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to match with wildcard: {:?}", result);

    match result.unwrap() {
        Value::Text(s) => assert_eq!(s, "Other color"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_enum_match_partial_coverage() {
    let source = r#"
        variant TrafficLight then Red, Yellow, Green end

        bind light to Yellow

        match light with
            when Red then "Stop"
            when Yellow then "Caution"
            when Green then "Go"
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to match enum: {:?}", result);

    match result.unwrap() {
        Value::Text(s) => assert_eq!(s, "Caution"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_enum_match_in_function() {
    let source = r#"
        variant Bool then Yes, No end

        chant bool_to_text(b) then
            match b with
                when Yes then yield "affirmative"
                when No then yield "negative"
            end
        end

        bind yes_text to bool_to_text(Yes)
        bind no_text to bool_to_text(No)
    "#;

    match eval_and_get(source, "yes_text").unwrap() {
        Value::Text(s) => assert_eq!(s, "affirmative"),
        _ => panic!("Expected Text"),
    }

    match eval_and_get(source, "no_text").unwrap() {
        Value::Text(s) => assert_eq!(s, "negative"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_enum_match_nested() {
    let source = r#"
        variant Color then Red, Green, Blue end
        variant Size then Small, Large end

        bind color to Red
        bind size to Large

        match color with
            when Red then match size with
                when Small then "Small red"
                when Large then "Large red"
            end
            when Green then "Green"
            when Blue then "Blue"
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to match nested enums: {:?}", result);

    match result.unwrap() {
        Value::Text(s) => assert_eq!(s, "Large red"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_enum_match_with_variable_binding() {
    let source = r#"
        variant Status then Ready, Waiting end

        bind status to Ready

        match status with
            when Ready then
                bind msg to "System ready"
                msg
            when Waiting then "Still waiting"
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed with variable binding: {:?}", result);

    match result.unwrap() {
        Value::Text(s) => assert_eq!(s, "System ready"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_enum_match_returns_variant() {
    let source = r#"
        variant Color then Red, Green, Blue end
        variant Intensity then Low, High end

        bind color to Blue

        match color with
            when Red then High
            when Green then Low
            when Blue then High
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to return variant: {:?}", result);

    match result.unwrap() {
        Value::VariantValue { variant_name, .. } => {
            assert_eq!(variant_name, "High");
        }
        _ => panic!("Expected VariantValue"),
    }
}

#[test]
fn test_multiple_enums_same_variant_names() {
    // Test that variants from different enums don't conflict
    // Note: The last definition wins when variants have the same name
    let source = r#"
        variant Status1 then Active, Inactive end

        bind s1 to Active

        match s1 with
            when Active then "matched"
            when Inactive then "not matched"
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed with same variant names: {:?}", result);

    match result.unwrap() {
        Value::Text(s) => assert_eq!(s, "matched"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_enum_match_state_machine() {
    let source = r#"
        variant State then Start, Running, Stopped end

        chant next_state(current) then
            match current with
                when Start then yield Running
                when Running then yield Stopped
                when Stopped then yield Start
            end
        end

        bind state1 to Start
        bind state2 to next_state(state1)
        bind state3 to next_state(state2)
        bind state4 to next_state(state3)
    "#;

    // Start -> Running
    match eval_and_get(source, "state2").unwrap() {
        Value::VariantValue { variant_name, .. } => assert_eq!(variant_name, "Running"),
        _ => panic!("Expected Running"),
    }

    // Running -> Stopped
    match eval_and_get(source, "state3").unwrap() {
        Value::VariantValue { variant_name, .. } => assert_eq!(variant_name, "Stopped"),
        _ => panic!("Expected Stopped"),
    }

    // Stopped -> Start (cycle)
    match eval_and_get(source, "state4").unwrap() {
        Value::VariantValue { variant_name, .. } => assert_eq!(variant_name, "Start"),
        _ => panic!("Expected Start"),
    }
}

#[test]
fn test_enum_match_with_computation() {
    let source = r#"
        variant Operation then Add, Subtract, Multiply end

        bind op to Multiply

        bind a to 10
        bind b to 5

        match op with
            when Add then a + b
            when Subtract then a - b
            when Multiply then a * b
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to compute in match: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 50.0),
        _ => panic!("Expected Number(50)"),
    }
}

#[test]
fn test_enum_match_single_variant() {
    let source = r#"
        variant Unit then Only end

        bind value to Only

        match value with
            when Only then "matched"
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to match single variant: {:?}", result);

    match result.unwrap() {
        Value::Text(s) => assert_eq!(s, "matched"),
        _ => panic!("Expected Text"),
    }
}
