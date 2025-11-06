/// Tests for user-defined enum (variant) types - Phase 1b: Evaluation
///
/// These tests verify that simple enums (unit variants only) work correctly
/// at runtime.

use glimmer_weave::{Lexer, Parser, Evaluator, Value};

/// Helper function to evaluate source code and return the environment
fn eval_program(source: &str) -> Result<Evaluator, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut evaluator = Evaluator::new();
    for node in &ast {
        evaluator.eval_node(node).map_err(|e| format!("Runtime error: {:?}", e))?;
    }

    Ok(evaluator)
}

/// Helper to evaluate and get a variable's value
fn eval_and_get(source: &str, var_name: &str) -> Result<Value, String> {
    let evaluator = eval_program(source)?;
    evaluator.environment().get(var_name).map_err(|e| format!("{:?}", e))
}

#[test]
fn test_simple_enum_definition() {
    let source = r#"
        variant Color then
            Red,
            Green,
            Blue
        end
    "#;

    let evaluator = eval_program(source).expect("Failed to evaluate simple enum");

    // Check that the enum definition exists
    let color_def = evaluator.environment().get("Color");
    assert!(color_def.is_ok(), "Enum definition 'Color' not found");

    match color_def.unwrap() {
        Value::VariantDef { name, variants, .. } => {
            assert_eq!(name, "Color");
            assert_eq!(variants.len(), 3);
        }
        _ => panic!("Expected VariantDef value"),
    }
}

#[test]
fn test_unit_variant_constructors() {
    let source = r#"
        variant Color then
            Red,
            Green,
            Blue
        end
    "#;

    let evaluator = eval_program(source).expect("Failed to evaluate");

    // Check that each variant constructor exists and creates correct values
    for variant_name in &["Red", "Green", "Blue"] {
        let variant_value = evaluator.environment().get(variant_name);
        assert!(variant_value.is_ok(), "Variant '{}' not found", variant_name);

        match variant_value.unwrap() {
            Value::VariantValue { enum_name, variant_name: vname, fields, .. } => {
                assert_eq!(enum_name, "Color");
                assert_eq!(vname, *variant_name);
                assert_eq!(fields.len(), 0, "Unit variant should have no fields");
            }
            _ => panic!("Expected VariantValue for '{}'", variant_name),
        }
    }
}

#[test]
fn test_bind_to_variant() {
    let source = r#"
        variant Direction then
            North,
            South,
            East,
            West
        end

        bind heading to North
    "#;

    let result = eval_and_get(source, "heading");
    assert!(result.is_ok(), "Failed to bind variant: {:?}", result);

    match result.unwrap() {
        Value::VariantValue { enum_name, variant_name, fields, .. } => {
            assert_eq!(enum_name, "Direction");
            assert_eq!(variant_name, "North");
            assert_eq!(fields.len(), 0);
        }
        _ => panic!("Expected VariantValue"),
    }
}

#[test]
fn test_multiple_enum_definitions() {
    let source = r#"
        variant Color then Red, Green, Blue end
        variant Status then Pending, Active, Complete end

        bind my_color to Green
        bind my_status to Active
    "#;

    let evaluator = eval_program(source).expect("Failed to evaluate");

    // Check Color enum
    let color = evaluator.environment().get("my_color").unwrap();
    match color {
        Value::VariantValue { enum_name, variant_name, .. } => {
            assert_eq!(enum_name, "Color");
            assert_eq!(variant_name, "Green");
        }
        _ => panic!("Expected VariantValue for my_color"),
    }

    // Check Status enum
    let status = evaluator.environment().get("my_status").unwrap();
    match status {
        Value::VariantValue { enum_name, variant_name, .. } => {
            assert_eq!(enum_name, "Status");
            assert_eq!(variant_name, "Active");
        }
        _ => panic!("Expected VariantValue for my_status"),
    }
}

#[test]
fn test_variant_in_list() {
    let source = r#"
        variant Color then Red, Green, Blue end

        bind colors to [Red, Green, Blue]
    "#;

    let result = eval_and_get(source, "colors");
    assert!(result.is_ok(), "Failed to create list of variants: {:?}", result);

    match result.unwrap() {
        Value::List(items) => {
            assert_eq!(items.len(), 3);

            let names = ["Red", "Green", "Blue"];
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::VariantValue { variant_name, .. } => {
                        assert_eq!(variant_name, names[i]);
                    }
                    _ => panic!("Expected VariantValue in list"),
                }
            }
        }
        _ => panic!("Expected List value"),
    }
}

#[test]
fn test_variant_to_text() {
    let source = r#"
        variant Status then Pending, Active, Complete end

        bind status to Active
        bind status_text to to_text(status)
    "#;

    let result = eval_and_get(source, "status_text");
    assert!(result.is_ok(), "Failed to convert variant to text: {:?}", result);

    match result.unwrap() {
        Value::Text(text) => {
            assert_eq!(text, "Active");
        }
        _ => panic!("Expected Text value"),
    }
}

#[test]
fn test_single_variant_enum() {
    let source = r#"
        variant Unit then Only end

        bind value to Only
    "#;

    let result = eval_and_get(source, "value");
    assert!(result.is_ok(), "Failed with single variant enum: {:?}", result);

    match result.unwrap() {
        Value::VariantValue { enum_name, variant_name, .. } => {
            assert_eq!(enum_name, "Unit");
            assert_eq!(variant_name, "Only");
        }
        _ => panic!("Expected VariantValue"),
    }
}

#[test]
fn test_variant_comparison_identity() {
    // For now, we can't directly compare variants with `is`
    // This test just verifies that binding the same variant works
    let source = r#"
        variant Bool then Yes, No end

        bind a to Yes
        bind b to Yes
    "#;

    let evaluator = eval_program(source).expect("Failed to evaluate");

    let a = evaluator.environment().get("a").unwrap();
    let b = evaluator.environment().get("b").unwrap();

    // Both should be Yes variants
    match (a, b) {
        (
            Value::VariantValue { variant_name: vn1, .. },
            Value::VariantValue { variant_name: vn2, .. }
        ) => {
            assert_eq!(vn1, "Yes");
            assert_eq!(vn2, "Yes");
        }
        _ => panic!("Expected VariantValue for both a and b"),
    }
}

#[test]
fn test_variants_in_different_scopes() {
    // Variants should be available after enum definition
    let source = r#"
        variant Color then Red, Green end

        should true then
            bind inner_color to Red
        end

        bind outer_color to Green
    "#;

    let evaluator = eval_program(source).expect("Failed to use variants in different scopes");
    let outer = evaluator.environment().get("outer_color").unwrap();

    match outer {
        Value::VariantValue { variant_name, .. } => {
            assert_eq!(variant_name, "Green");
        }
        _ => panic!("Expected VariantValue"),
    }
}

#[test]
fn test_type_name_for_variant() {
    let source = r#"
        variant Status then Ready end
        bind s to Ready
    "#;

    let evaluator = eval_program(source).expect("Failed to evaluate");
    let status_value = evaluator.environment().get("s").unwrap();

    // type_name() should return the variant name
    assert_eq!(status_value.type_name(), "Ready");
}
