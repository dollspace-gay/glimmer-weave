/// Tests for user-defined enum (variant) types - Phase 2: Enums with Data
///
/// These tests verify that enums with associated data (tagged unions) work correctly.

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
fn test_variant_constructor_with_single_field() {
    let source = r#"
        variant Message then
            Write(text: Text)
        end

        bind msg to Write("Hello")
    "#;

    let result = eval_and_get(source, "msg");
    assert!(result.is_ok(), "Failed to create variant with data: {:?}", result);

    match result.unwrap() {
        Value::VariantValue { enum_name, variant_name, fields, .. } => {
            assert_eq!(enum_name, "Message");
            assert_eq!(variant_name, "Write");
            assert_eq!(fields.len(), 1);
            match &fields[0] {
                Value::Text(s) => assert_eq!(s, "Hello"),
                _ => panic!("Expected Text field"),
            }
        }
        _ => panic!("Expected VariantValue"),
    }
}

#[test]
fn test_variant_constructor_with_multiple_fields() {
    let source = r#"
        variant Message then
            Move(x: Number, y: Number)
        end

        bind msg to Move(10, 20)
    "#;

    let result = eval_and_get(source, "msg");
    assert!(result.is_ok(), "Failed to create variant with multiple fields: {:?}", result);

    match result.unwrap() {
        Value::VariantValue { enum_name, variant_name, fields, .. } => {
            assert_eq!(enum_name, "Message");
            assert_eq!(variant_name, "Move");
            assert_eq!(fields.len(), 2);
            match &fields[0] {
                Value::Number(n) => assert_eq!(*n, 10.0),
                _ => panic!("Expected Number field"),
            }
            match &fields[1] {
                Value::Number(n) => assert_eq!(*n, 20.0),
                _ => panic!("Expected Number field"),
            }
        }
        _ => panic!("Expected VariantValue"),
    }
}

#[test]
fn test_mixed_unit_and_data_variants() {
    let source = r#"
        variant Message then
            Quit,
            Move(x: Number, y: Number),
            Write(text: Text)
        end

        bind msg1 to Quit
        bind msg2 to Move(5, 10)
        bind msg3 to Write("Test")
    "#;

    // Test unit variant
    let msg1 = eval_and_get(source, "msg1").unwrap();
    match msg1 {
        Value::VariantValue { variant_name, fields, .. } => {
            assert_eq!(variant_name, "Quit");
            assert_eq!(fields.len(), 0);
        }
        _ => panic!("Expected VariantValue for msg1"),
    }

    // Test data variant with two fields
    let msg2 = eval_and_get(source, "msg2").unwrap();
    match msg2 {
        Value::VariantValue { variant_name, fields, .. } => {
            assert_eq!(variant_name, "Move");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected VariantValue for msg2"),
    }

    // Test data variant with one field
    let msg3 = eval_and_get(source, "msg3").unwrap();
    match msg3 {
        Value::VariantValue { variant_name, fields, .. } => {
            assert_eq!(variant_name, "Write");
            assert_eq!(fields.len(), 1);
        }
        _ => panic!("Expected VariantValue for msg3"),
    }
}

#[test]
fn test_variant_to_text_with_fields() {
    let source = r#"
        variant Point then
            Point2D(x: Number, y: Number)
        end

        bind p to Point2D(3, 4)
        bind p_text to to_text(p)
    "#;

    let result = eval_and_get(source, "p_text");
    assert!(result.is_ok(), "Failed to convert variant to text: {:?}", result);

    match result.unwrap() {
        Value::Text(text) => {
            assert_eq!(text, "Point2D(3, 4)");
        }
        _ => panic!("Expected Text value"),
    }
}

#[test]
fn test_variant_constructor_arity_error() {
    let source = r#"
        variant Message then
            Move(x: Number, y: Number)
        end

        bind msg to Move(10)
    "#;

    let result = eval_and_get(source, "msg");
    assert!(result.is_err(), "Should fail with arity mismatch");
    assert!(result.unwrap_err().contains("ArityMismatch"));
}

#[test]
fn test_nested_variant_values() {
    let source = r#"
        variant Container then
            Box(content: Text)
        end

        bind inner to Box("treasure")
        bind items to [Box("a"), Box("b"), Box("c")]
    "#;

    let inner = eval_and_get(source, "inner").unwrap();
    match inner {
        Value::VariantValue { variant_name, fields, .. } => {
            assert_eq!(variant_name, "Box");
            assert_eq!(fields.len(), 1);
        }
        _ => panic!("Expected VariantValue"),
    }

    let items = eval_and_get(source, "items").unwrap();
    match items {
        Value::List(list) => {
            assert_eq!(list.len(), 3);
            for item in list {
                match item {
                    Value::VariantValue { variant_name, .. } => {
                        assert_eq!(variant_name, "Box");
                    }
                    _ => panic!("Expected VariantValue in list"),
                }
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_variant_in_function() {
    let source = r#"
        variant Coord then
            Point(x: Number, y: Number)
        end

        chant make_point(a, b) then
            yield Point(a, b)
        end

        bind p to make_point(7, 8)
    "#;

    let result = eval_and_get(source, "p");
    assert!(result.is_ok(), "Failed to create variant in function: {:?}", result);

    match result.unwrap() {
        Value::VariantValue { variant_name, fields, .. } => {
            assert_eq!(variant_name, "Point");
            assert_eq!(fields.len(), 2);
            match (&fields[0], &fields[1]) {
                (Value::Number(x), Value::Number(y)) => {
                    assert_eq!(*x, 7.0);
                    assert_eq!(*y, 8.0);
                }
                _ => panic!("Expected Number fields"),
            }
        }
        _ => panic!("Expected VariantValue"),
    }
}

#[test]
fn test_three_field_variant() {
    let source = r#"
        variant Color then
            RGB(r: Number, g: Number, b: Number)
        end

        bind red to RGB(255, 0, 0)
    "#;

    let result = eval_and_get(source, "red");
    assert!(result.is_ok(), "Failed to create variant with three fields: {:?}", result);

    match result.unwrap() {
        Value::VariantValue { variant_name, fields, .. } => {
            assert_eq!(variant_name, "RGB");
            assert_eq!(fields.len(), 3);
            match (&fields[0], &fields[1], &fields[2]) {
                (Value::Number(r), Value::Number(g), Value::Number(b)) => {
                    assert_eq!(*r, 255.0);
                    assert_eq!(*g, 0.0);
                    assert_eq!(*b, 0.0);
                }
                _ => panic!("Expected Number fields"),
            }
        }
        _ => panic!("Expected VariantValue"),
    }
}
