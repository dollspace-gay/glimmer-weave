/// Tests for user-defined enum (variant) types - Phase 2: Pattern Matching with Data
///
/// These tests verify that pattern matching works correctly with enums that have associated data,
/// including field extraction and binding.

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
fn test_match_single_field_extraction() {
    let source = r#"
        variant Message then
            Write(text: Text)
        end

        bind msg to Write("Hello")

        match msg with
            when Write(t) then t
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to match and extract field: {:?}", result);

    match result.unwrap() {
        Value::Text(s) => assert_eq!(s, "Hello"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_match_two_fields_extraction() {
    let source = r#"
        variant Message then
            Move(x: Number, y: Number)
        end

        bind msg to Move(10, 20)

        match msg with
            when Move(a, b) then a + b
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to match and extract two fields: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 30.0),
        _ => panic!("Expected Number(30)"),
    }
}

#[test]
fn test_match_three_fields_extraction() {
    let source = r#"
        variant Color then
            RGB(r: Number, g: Number, b: Number)
        end

        bind color to RGB(255, 128, 64)

        match color with
            when RGB(r, g, b) then r + g + b
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to match and extract three fields: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 447.0),
        _ => panic!("Expected Number(447)"),
    }
}

#[test]
fn test_match_mixed_variants() {
    let source = r#"
        variant Message then
            Quit,
            Move(x: Number, y: Number),
            Write(text: Text)
        end

        chant process(msg) then
            match msg with
                when Quit then yield "Quitting"
                when Move(x, y) then yield "Moving to: " + to_text(x) + "," + to_text(y)
                when Write(text) then yield "Writing: " + text
            end
        end

        bind r1 to process(Quit)
        bind r2 to process(Move(5, 10))
        bind r3 to process(Write("test"))
    "#;

    match eval_and_get(source, "r1").unwrap() {
        Value::Text(s) => assert_eq!(s, "Quitting"),
        _ => panic!("Expected Text for r1"),
    }

    match eval_and_get(source, "r2").unwrap() {
        Value::Text(s) => assert_eq!(s, "Moving to: 5,10"),
        _ => panic!("Expected Text for r2"),
    }

    match eval_and_get(source, "r3").unwrap() {
        Value::Text(s) => assert_eq!(s, "Writing: test"),
        _ => panic!("Expected Text for r3"),
    }
}

#[test]
fn test_match_field_in_computation() {
    let source = r#"
        variant Point then
            Point2D(x: Number, y: Number)
        end

        chant distance_from_origin(point) then
            match point with
                when Point2D(x, y) then yield x * x + y * y
            end
        end

        bind dist to distance_from_origin(Point2D(3, 4))
    "#;

    let result = eval_and_get(source, "dist");
    assert!(result.is_ok(), "Failed to compute with extracted fields: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 25.0), // 3^2 + 4^2
        _ => panic!("Expected Number(25)"),
    }
}

#[test]
fn test_match_nested_in_function() {
    let source = r#"
        variant Shape then
            Circle(radius: Number),
            Rectangle(width: Number, height: Number)
        end

        chant area(shape) then
            match shape with
                when Circle(r) then yield 3.14 * r * r
                when Rectangle(w, h) then yield w * h
            end
        end

        bind circle_area to area(Circle(5))
        bind rect_area to area(Rectangle(4, 6))
    "#;

    match eval_and_get(source, "circle_area").unwrap() {
        Value::Number(n) => assert!((n - 78.5).abs() < 0.1),
        _ => panic!("Expected Number for circle_area"),
    }

    match eval_and_get(source, "rect_area").unwrap() {
        Value::Number(n) => assert_eq!(n, 24.0),
        _ => panic!("Expected Number for rect_area"),
    }
}

#[test]
fn test_match_with_wildcard_for_data_variant() {
    let source = r#"
        variant Message then
            Move(x: Number, y: Number),
            Write(text: Text)
        end

        bind msg to Move(10, 20)

        match msg with
            when Write(t) then "Write: " + t
            otherwise then "Other message"
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to match with wildcard: {:?}", result);

    match result.unwrap() {
        Value::Text(s) => assert_eq!(s, "Other message"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_match_field_arity_mismatch() {
    let source = r#"
        variant Message then
            Move(x: Number, y: Number)
        end

        bind msg to Move(10, 20)

        match msg with
            when Move(x) then x
            otherwise then 0
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Should fall through to otherwise: {:?}", result);

    // Should fall through to otherwise because field count doesn't match
    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 0.0),
        _ => panic!("Expected Number(0)"),
    }
}

#[test]
fn test_match_using_fields_multiple_times() {
    let source = r#"
        variant Vector then
            Vec2(x: Number, y: Number)
        end

        bind v to Vec2(3, 4)

        match v with
            when Vec2(a, b) then a * a + b * b
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to use fields multiple times: {:?}", result);

    match result.unwrap() {
        Value::Number(n) => assert_eq!(n, 25.0),
        _ => panic!("Expected Number(25)"),
    }
}

#[test]
fn test_match_text_fields() {
    let source = r#"
        variant Person then
            Named(firstname: Text, lastname: Text)
        end

        bind person to Named("John", "Doe")

        match person with
            when Named(f, l) then f + " " + l
        end
    "#;

    let result = eval_program(source);
    assert!(result.is_ok(), "Failed to match text fields: {:?}", result);

    match result.unwrap() {
        Value::Text(s) => assert_eq!(s, "John Doe"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_match_multiple_cases_with_data() {
    let source = r#"
        variant Event then
            Click(x: Number, y: Number),
            KeyPress(key: Text),
            Resize(width: Number, height: Number)
        end

        chant describe(event) then
            match event with
                when Click(x, y) then yield "Clicked at " + to_text(x) + "," + to_text(y)
                when KeyPress(k) then yield "Key pressed: " + k
                when Resize(w, h) then yield "Resized to " + to_text(w) + "x" + to_text(h)
            end
        end

        bind d1 to describe(Click(100, 200))
        bind d2 to describe(KeyPress("Enter"))
        bind d3 to describe(Resize(800, 600))
    "#;

    match eval_and_get(source, "d1").unwrap() {
        Value::Text(s) => assert_eq!(s, "Clicked at 100,200"),
        _ => panic!("Expected Text for d1"),
    }

    match eval_and_get(source, "d2").unwrap() {
        Value::Text(s) => assert_eq!(s, "Key pressed: Enter"),
        _ => panic!("Expected Text for d2"),
    }

    match eval_and_get(source, "d3").unwrap() {
        Value::Text(s) => assert_eq!(s, "Resized to 800x600"),
        _ => panic!("Expected Text for d3"),
    }
}
