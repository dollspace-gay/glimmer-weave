/// Tests for trait system runtime dispatch (Phase 3)
/// Verifies that trait methods can be called and execute correctly at runtime

use glimmer_weave::{Evaluator, Lexer, Parser};

fn run_program(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut evaluator = Evaluator::new();
    let result = evaluator.eval(&ast).map_err(|e| format!("Runtime error: {:?}", e))?;

    // Convert result to string for comparison
    Ok(format!("{:?}", result))
}

// ============================================================================
// Basic trait method calls
// ============================================================================

#[test]
fn test_simple_trait_method_call() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield "Number"
            end
        end

        bind num to 42
        num.show()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), r#"Text("Number")"#);
}

#[test]
fn test_trait_method_using_self() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        embody Display for Text then
            chant show(self) -> Text then
                yield self
            end
        end

        bind greeting to "Hello"
        greeting.show()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), r#"Text("Hello")"#);
}

#[test]
fn test_trait_method_with_parameters() {
    let source = r#"
        aspect Formatter then
            chant format(self, prefix: Text) -> Text
        end

        embody Formatter for Number then
            chant format(self, prefix: Text) -> Text then
                yield prefix
            end
        end

        bind num to 123
        num.format("Value: ")
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), r#"Text("Value: ")"#);
}

// ============================================================================
// Multiple trait implementations
// ============================================================================

#[test]
fn test_multiple_types_implementing_same_trait() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield "A Number"
            end
        end

        embody Display for Text then
            chant show(self) -> Text then
                yield "A Text"
            end
        end

        bind n to 42
        bind t to "hello"
        bind n_result to n.show()
        bind t_result to t.show()
        t_result
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), r#"Text("A Text")"#);
}

#[test]
fn test_trait_with_multiple_methods() {
    let source = r#"
        aspect Comparable then
            chant is_positive(self) -> Truth
            chant is_zero(self) -> Truth
        end

        embody Comparable for Number then
            chant is_positive(self) -> Truth then
                yield self greater than 0
            end

            chant is_zero(self) -> Truth then
                yield self is 0
            end
        end

        bind num to 5
        num.is_positive()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Truth(true)");
}

// ============================================================================
// Trait methods calling other functions
// ============================================================================

#[test]
fn test_trait_method_calling_builtin() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield to_text(self)
            end
        end

        bind num to 42
        num.show()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), r#"Text("42")"#);
}

#[test]
fn test_trait_method_calling_user_function() {
    let source = r#"
        chant helper(x) then
            yield x + 10
        end

        aspect Calculator then
            chant compute(self) -> Number
        end

        embody Calculator for Number then
            chant compute(self) -> Number then
                yield helper(self)
            end
        end

        bind num to 5
        num.compute()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(15.0)");
}

// ============================================================================
// Complex scenarios
// ============================================================================

#[test]
fn test_trait_in_function_parameter() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield "Number"
            end
        end

        chant print_value(val) then
            yield val.show()
        end

        bind num to 42
        print_value(num)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), r#"Text("Number")"#);
}

#[test]
fn test_chained_trait_method_calls() {
    let source = r#"
        aspect Builder then
            chant add_one(self) -> Number
        end

        embody Builder for Number then
            chant add_one(self) -> Number then
                yield self + 1
            end
        end

        bind num to 5
        bind result1 to num.add_one()
        bind result2 to result1.add_one()
        result2
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(7.0)");
}

#[test]
fn test_trait_with_control_flow() {
    let source = r#"
        aspect Checker then
            chant check(self) -> Text
        end

        embody Checker for Number then
            chant check(self) -> Text then
                should self greater than 10 then
                    yield "Big"
                otherwise
                    yield "Small"
                end
            end
        end

        bind small to 5
        bind big to 20
        bind small_result to small.check()
        bind big_result to big.check()
        big_result
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), r#"Text("Big")"#);
}

// ============================================================================
// Generic traits (basic - full support in Phase 4)
// ============================================================================

#[test]
fn test_generic_trait_concrete_instantiation() {
    let source = r#"
        aspect Container<T> then
            chant size(self) -> Number
        end

        embody Container<Number> for List then
            chant size(self) -> Number then
                yield list_length(self)
            end
        end

        bind my_list to [1, 2, 3]
        my_list.size()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(3.0)");
}
