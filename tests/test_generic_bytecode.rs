/// Tests for generic type parameters in bytecode compilation
/// Tests that monomorphization correctly generates specialized functions

use glimmer_weave::{Lexer, Parser, bytecode_compiler};

fn compile_source(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    // Use monomorphization compilation
    bytecode_compiler::compile_with_monomorphization(&ast)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    Ok(())
}

#[test]
fn test_monomorphize_identity_number() {
    let source = r#"
        chant identity<T>(x: T) -> T then
            yield x
        end

        identity<Number>(42)
    "#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result);
}

#[test]
fn test_monomorphize_identity_text() {
    let source = r#"
        chant identity<T>(x: T) -> T then
            yield x
        end

        identity<Text>("hello")
    "#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result);
}

#[test]
fn test_monomorphize_multiple_instantiations() {
    let source = r#"
        chant identity<T>(x: T) -> T then
            yield x
        end

        identity<Number>(42)
        identity<Text>("hello")
        identity<Truth>(true)
    "#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result);
}

#[test]
fn test_monomorphize_pair_function() {
    let source = r#"
        chant make_pair<T, U>(a: T, b: U) -> Number then
            yield 100
        end

        make_pair<Number, Text>(42, "hello")
    "#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result);
}

#[test]
fn test_non_generic_function_unaffected() {
    let source = r#"
        chant double(x: Number) -> Number then
            yield x + x
        end

        double(21)
    "#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result);
}

#[test]
fn test_mixed_generic_and_non_generic() {
    let source = r#"
        chant identity<T>(x: T) -> T then
            yield x
        end

        chant double(x: Number) -> Number then
            yield x + x
        end

        identity<Number>(42)
        double(21)
    "#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result);
}

#[test]
fn test_generic_function_no_calls() {
    // Generic function defined but never called
    // Should compile (monomorphization only generates specialized versions for calls)
    let source = r#"
        chant identity<T>(x: T) -> T then
            yield x
        end

        42
    "#;

    let result = compile_source(source);
    // This should succeed - no instantiations needed
    assert!(result.is_ok(), "Compilation failed: {:?}", result);
}

#[test]
fn test_generic_call_in_bind() {
    let source = r#"
        chant identity<T>(x: T) -> T then
            yield x
        end

        bind x to identity<Number>(42)
    "#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result);
}

#[test]
fn test_nested_generic_calls() {
    let source = r#"
        chant identity<T>(x: T) -> T then
            yield x
        end

        identity<Number>(identity<Number>(42))
    "#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result);
}
