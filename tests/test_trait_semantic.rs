/// Tests for trait system semantic analysis (Phase 2)
/// Verifies that trait definitions and implementations are properly validated

use glimmer_weave::{Lexer, Parser, SemanticAnalyzer};

fn analyze_source(source: &str) -> Result<(), Vec<String>> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| vec![format!("Parse error: {:?}", e)])?;

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast).map_err(|errors| {
        errors.iter().map(|e| format!("{:?}", e)).collect()
    })
}

// ============================================================================
// Valid trait definitions
// ============================================================================

#[test]
fn test_valid_simple_trait() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_valid_generic_trait() {
    let source = r#"
        aspect Container<T> then
            chant add(self, item: T)
            chant size(self) -> Number
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_valid_trait_with_multiple_methods() {
    let source = r#"
        aspect Comparable then
            chant compare(self, other) -> Number
            chant is_less(self, other) -> Truth
            chant is_equal(self, other) -> Truth
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

// ============================================================================
// Invalid trait definitions
// ============================================================================

#[test]
fn test_trait_method_without_self() {
    let source = r#"
        aspect Display then
            chant show(value) -> Text
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_err(), "Should fail - method missing 'self' parameter");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("must have 'self'")),
            "Expected error about missing 'self' parameter: {:?}", errors);
}

#[test]
fn test_trait_method_with_wrong_first_param() {
    let source = r#"
        aspect Display then
            chant show(this) -> Text
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_err(), "Should fail - wrong first parameter name");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("must have 'self'")),
            "Expected error about 'self' parameter: {:?}", errors);
}

#[test]
fn test_duplicate_trait_definition() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        aspect Display then
            chant render(self) -> Text
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_err(), "Should fail - duplicate trait definition");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("DuplicateDefinition")),
            "Expected duplicate definition error: {:?}", errors);
}

// ============================================================================
// Valid trait implementations
// ============================================================================

#[test]
fn test_valid_simple_implementation() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield to_text(self)
            end
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_valid_generic_trait_implementation() {
    let source = r#"
        aspect Container<T> then
            chant add(self, item: T)
        end

        embody Container<Number> for NumberList then
            chant add(self, item: Number) then
                yield list_append(self, item)
            end
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_valid_multiple_methods_implementation() {
    let source = r#"
        aspect Comparable then
            chant compare(self, other) -> Number
            chant is_equal(self, other) -> Truth
        end

        embody Comparable for Number then
            chant compare(self, other) -> Number then
                should self less than other then
                    yield -1
                otherwise
                    yield 0
                end
            end

            chant is_equal(self, other) -> Truth then
                yield self is other
            end
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

// ============================================================================
// Invalid trait implementations
// ============================================================================

#[test]
fn test_implement_undefined_trait() {
    let source = r#"
        embody Display for Number then
            chant show(self) -> Text then
                yield to_text(self)
            end
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_err(), "Should fail - trait not defined");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("UndefinedVariable")),
            "Expected undefined trait error: {:?}", errors);
}

#[test]
fn test_missing_trait_method_in_implementation() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
            chant debug(self) -> Text
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield to_text(self)
            end
            # Missing 'debug' method
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_err(), "Should fail - missing method implementation");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("Missing method 'debug'")),
            "Expected missing method error: {:?}", errors);
}

#[test]
fn test_wrong_number_of_type_arguments() {
    let source = r#"
        aspect Container<T> then
            chant add(self, item: T)
        end

        embody Container for NumberList then
            chant add(self, item: Number) then
                yield list_append(self, item)
            end
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_err(), "Should fail - wrong type argument count");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("expects 1 type argument")),
            "Expected type argument count mismatch: {:?}", errors);
}

#[test]
fn test_duplicate_trait_implementation() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield to_text(self)
            end
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield "duplicate"
            end
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_err(), "Should fail - duplicate implementation");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("already implemented")),
            "Expected duplicate implementation error: {:?}", errors);
}

// ============================================================================
// Complex scenarios
// ============================================================================

#[test]
fn test_multiple_traits_and_implementations() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        aspect Comparable then
            chant compare(self, other) -> Number
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield to_text(self)
            end
        end

        embody Comparable for Number then
            chant compare(self, other) -> Number then
                yield 0
            end
        end

        embody Display for Text then
            chant show(self) -> Text then
                yield self
            end
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_trait_with_existing_function() {
    let source = r#"
        chant helper() -> Number then
            yield 42
        end

        aspect Display then
            chant show(self) -> Text
        end

        embody Display for Number then
            chant show(self) -> Text then
                bind n to helper()
                yield to_text(n)
            end
        end
    "#;

    let result = analyze_source(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
}
