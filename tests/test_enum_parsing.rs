/// Tests for user-defined enum (variant) types - Phase 1: Parsing
///
/// These tests verify that enum definitions parse correctly according
/// to the natural language syntax design.

use glimmer_weave::{Lexer, Parser, AstNode};

/// Helper function to parse source code and return the first AST node
fn parse_first(source: &str) -> Result<AstNode, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("{:?}", e))?;

    ast.into_iter().next().ok_or_else(|| "No AST nodes parsed".to_string())
}

#[test]
fn test_parse_simple_enum() {
    let source = r#"
        variant Color then
            Red,
            Green,
            Blue
        end
    "#;

    let result = parse_first(source);
    assert!(result.is_ok(), "Failed to parse simple enum: {:?}", result);

    match result.unwrap() {
        AstNode::VariantDef { name, type_params, variants, .. } => {
            assert_eq!(name, "Color");
            assert_eq!(type_params.len(), 0);
            assert_eq!(variants.len(), 3);

            assert_eq!(variants[0].name, "Red");
            assert_eq!(variants[0].fields.len(), 0);

            assert_eq!(variants[1].name, "Green");
            assert_eq!(variants[1].fields.len(), 0);

            assert_eq!(variants[2].name, "Blue");
            assert_eq!(variants[2].fields.len(), 0);
        }
        _ => panic!("Expected VariantDef node"),
    }
}

#[test]
fn test_parse_enum_without_commas() {
    let source = r#"
        variant Direction then
            North
            South
            East
            West
        end
    "#;

    let result = parse_first(source);
    assert!(result.is_ok(), "Failed to parse enum without commas: {:?}", result);

    match result.unwrap() {
        AstNode::VariantDef { name, variants, .. } => {
            assert_eq!(name, "Direction");
            assert_eq!(variants.len(), 4);
            assert_eq!(variants[0].name, "North");
            assert_eq!(variants[3].name, "West");
        }
        _ => panic!("Expected VariantDef node"),
    }
}

#[test]
fn test_parse_enum_with_data() {
    let source = r#"
        variant Message then
            Quit,
            Move(x: Number, y: Number),
            Write(text: Text)
        end
    "#;

    let result = parse_first(source);
    assert!(result.is_ok(), "Failed to parse enum with data: {:?}", result);

    match result.unwrap() {
        AstNode::VariantDef { name, variants, .. } => {
            assert_eq!(name, "Message");
            assert_eq!(variants.len(), 3);

            // Quit - unit variant
            assert_eq!(variants[0].name, "Quit");
            assert_eq!(variants[0].fields.len(), 0);

            // Move - variant with two fields
            assert_eq!(variants[1].name, "Move");
            assert_eq!(variants[1].fields.len(), 2);
            assert_eq!(variants[1].fields[0].name, "x");
            assert_eq!(variants[1].fields[1].name, "y");

            // Write - variant with one field
            assert_eq!(variants[2].name, "Write");
            assert_eq!(variants[2].fields.len(), 1);
            assert_eq!(variants[2].fields[0].name, "text");
        }
        _ => panic!("Expected VariantDef node"),
    }
}

#[test]
fn test_parse_generic_enum() {
    let source = r#"
        variant Option<T> then
            Some(value: T),
            None
        end
    "#;

    let result = parse_first(source);
    assert!(result.is_ok(), "Failed to parse generic enum: {:?}", result);

    match result.unwrap() {
        AstNode::VariantDef { name, type_params, variants, .. } => {
            assert_eq!(name, "Option");
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0], "T");

            assert_eq!(variants.len(), 2);

            // Some(value: T)
            assert_eq!(variants[0].name, "Some");
            assert_eq!(variants[0].fields.len(), 1);
            assert_eq!(variants[0].fields[0].name, "value");

            // None
            assert_eq!(variants[1].name, "None");
            assert_eq!(variants[1].fields.len(), 0);
        }
        _ => panic!("Expected VariantDef node"),
    }
}

#[test]
fn test_parse_generic_enum_multiple_params() {
    let source = r#"
        variant Result<T, E> then
            Ok(value: T),
            Err(error: E)
        end
    "#;

    let result = parse_first(source);
    assert!(result.is_ok(), "Failed to parse enum with multiple type params: {:?}", result);

    match result.unwrap() {
        AstNode::VariantDef { name, type_params, variants, .. } => {
            assert_eq!(name, "Result");
            assert_eq!(type_params.len(), 2);
            assert_eq!(type_params[0], "T");
            assert_eq!(type_params[1], "E");

            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Ok");
            assert_eq!(variants[1].name, "Err");
        }
        _ => panic!("Expected VariantDef node"),
    }
}

#[test]
fn test_parse_enum_single_variant() {
    let source = r#"
        variant Single then
            Only
        end
    "#;

    let result = parse_first(source);
    assert!(result.is_ok(), "Failed to parse single-variant enum: {:?}", result);

    match result.unwrap() {
        AstNode::VariantDef { name, variants, .. } => {
            assert_eq!(name, "Single");
            assert_eq!(variants.len(), 1);
            assert_eq!(variants[0].name, "Only");
        }
        _ => panic!("Expected VariantDef node"),
    }
}

#[test]
fn test_parse_complex_enum() {
    let source = r#"
        variant Shape then
            Circle(radius: Number),
            Rectangle(width: Number, height: Number),
            Triangle(base: Number, height: Number, angle: Number)
        end
    "#;

    let result = parse_first(source);
    assert!(result.is_ok(), "Failed to parse complex enum: {:?}", result);

    match result.unwrap() {
        AstNode::VariantDef { name, variants, .. } => {
            assert_eq!(name, "Shape");
            assert_eq!(variants.len(), 3);

            assert_eq!(variants[0].name, "Circle");
            assert_eq!(variants[0].fields.len(), 1);

            assert_eq!(variants[1].name, "Rectangle");
            assert_eq!(variants[1].fields.len(), 2);

            assert_eq!(variants[2].name, "Triangle");
            assert_eq!(variants[2].fields.len(), 3);
        }
        _ => panic!("Expected VariantDef node"),
    }
}

#[test]
fn test_parse_recursive_enum() {
    let source = r#"
        variant List<T> then
            Cons(head: T, tail: List<T>),
            Nil
        end
    "#;

    let result = parse_first(source);
    assert!(result.is_ok(), "Failed to parse recursive enum: {:?}", result);

    match result.unwrap() {
        AstNode::VariantDef { name, type_params, variants, .. } => {
            assert_eq!(name, "List");
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0], "T");

            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Cons");
            assert_eq!(variants[0].fields.len(), 2);
            assert_eq!(variants[1].name, "Nil");
        }
        _ => panic!("Expected VariantDef node"),
    }
}

#[test]
fn test_parse_enum_error_missing_then() {
    let source = r#"
        variant Color
            Red,
            Green
        end
    "#;

    let result = parse_first(source);
    assert!(result.is_err(), "Should fail without 'then' keyword");
}

#[test]
fn test_parse_enum_error_missing_end() {
    let source = r#"
        variant Color then
            Red,
            Green
    "#;

    let result = parse_first(source);
    assert!(result.is_err(), "Should fail without 'end' keyword");
}

#[test]
fn test_parse_enum_error_missing_name() {
    let source = r#"
        variant then
            Red
        end
    "#;

    let result = parse_first(source);
    assert!(result.is_err(), "Should fail without enum name");
}

#[test]
fn test_parse_enum_inline_style() {
    // Test compact inline style
    let source = "variant Status then Pending, Active, Complete end";

    let result = parse_first(source);
    assert!(result.is_ok(), "Failed to parse inline enum: {:?}", result);

    match result.unwrap() {
        AstNode::VariantDef { name, variants, .. } => {
            assert_eq!(name, "Status");
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].name, "Pending");
            assert_eq!(variants[1].name, "Active");
            assert_eq!(variants[2].name, "Complete");
        }
        _ => panic!("Expected VariantDef node"),
    }
}
