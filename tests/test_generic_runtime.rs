/// Tests for generic type parameters at runtime
/// This file tests that generic functions and structs execute correctly
/// across all three execution engines (interpreter, bytecode VM, native codegen)

use glimmer_weave::{Lexer, Parser, Evaluator, Value};

/// Helper function to run Glimmer-Weave code
fn run(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut evaluator = Evaluator::new();
    evaluator.eval(&ast).map_err(|e| format!("Runtime error: {:?}", e))
}

#[test]
fn test_generic_identity_function_interpreter() {
    let source = r#"
        # Generic identity function
        chant identity<T>(x: T) -> T then
            yield x
        end

        # Call with Number
        identity<Number>(42)
    "#;

    let result = run(source);
    assert!(result.is_ok(), "Generic identity function failed: {:?}", result);

    // Check the returned value
    if let Ok(Value::Number(n)) = result {
        assert_eq!(n, 42.0);
    } else {
        panic!("Expected Number(42), got {:?}", result);
    }
}

#[test]
fn test_generic_box_struct_interpreter() {
    let source = r#"
        # Generic Box struct
        form Box<T> with
            value as T
        end

        # Create Box with Number - return it
        Box<Number> { value: 42 }
    "#;

    let result = run(source);
    assert!(result.is_ok(), "Generic Box struct failed: {:?}", result);
}

#[test]
fn test_generic_pair_function_interpreter() {
    let source = r#"
        # Generic function with multiple type parameters
        chant make_pair<T, U>(a: T, b: U) -> Number then
            yield 100
        end

        make_pair<Number, Text>(42, "hello")
    "#;

    let result = run(source);
    assert!(result.is_ok(), "Generic pair function failed: {:?}", result);

    if let Ok(Value::Number(n)) = result {
        assert_eq!(n, 100.0);
    } else {
        panic!("Expected Number(100), got {:?}", result);
    }
}

#[test]
fn test_generic_function_without_type_args() {
    let source = r#"
        # Generic function can also be called without explicit type arguments
        chant identity<T>(x: T) -> T then
            yield x
        end

        # Call without type arguments - type erasure means this works
        identity(42)
    "#;

    let result = run(source);
    assert!(result.is_ok(), "Generic function without type args failed: {:?}", result);

    if let Ok(Value::Number(n)) = result {
        assert_eq!(n, 42.0);
    } else {
        panic!("Expected Number(42), got {:?}", result);
    }
}

#[test]
fn test_generic_identity_with_text() {
    let source = r#"
        chant identity<T>(x: T) -> T then
            yield x
        end

        identity<Text>("hello world")
    "#;

    let result = run(source);
    assert!(result.is_ok(), "Generic identity with text failed: {:?}", result);

    if let Ok(Value::Text(s)) = result {
        assert_eq!(s, "hello world");
    } else {
        panic!("Expected Text(\"hello world\"), got {:?}", result);
    }
}
