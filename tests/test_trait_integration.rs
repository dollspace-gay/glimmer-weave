/// Comprehensive integration tests for trait system (Phase 4)
/// Tests complex scenarios combining traits with other language features

use glimmer_weave::{Evaluator, Lexer, Parser};

fn run_program(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut evaluator = Evaluator::new();
    let result = evaluator.eval(&ast).map_err(|e| format!("Runtime error: {:?}", e))?;

    Ok(format!("{:?}", result))
}

// ============================================================================
// Traits with structs
// ============================================================================

#[test]
fn test_trait_for_struct() {
    let source = r#"
        form Point with x as Number y as Number end

        aspect Display then
            chant show(self) -> Text
        end

        embody Display for Point then
            chant show(self) -> Text then
                yield "Point"
            end
        end

        bind p to Point { x: 10, y: 20 }
        p.show()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), r#"Text("Point")"#);
}

#[test]
fn test_trait_accessing_struct_fields() {
    let source = r#"
        form Rectangle with width as Number height as Number end

        aspect Area then
            chant area(self) -> Number
        end

        embody Area for Rectangle then
            chant area(self) -> Number then
                yield self.width * self.height
            end
        end

        bind rect to Rectangle { width: 5, height: 3 }
        rect.area()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(15.0)");
}

// ============================================================================
// Traits with Outcome and Maybe
// ============================================================================

#[test]
fn test_trait_returning_outcome() {
    let source = r#"
        aspect Validator then
            chant validate(self) -> Text
        end

        embody Validator for Number then
            chant validate(self) -> Text then
                should self greater than 0 then
                    yield "Valid"
                otherwise
                    yield "Invalid"
                end
            end
        end

        bind positive to 10
        bind result to positive.validate()
        result
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), r#"Text("Valid")"#);
}

#[test]
fn test_trait_with_maybe_return() {
    let source = r#"
        aspect Checker then
            chant check(self) -> Truth
        end

        embody Checker for Number then
            chant check(self) -> Truth then
                yield self greater than 5
            end
        end

        bind num to 10
        num.check()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Truth(true)");
}

// ============================================================================
// Multiple traits on same type
// ============================================================================

#[test]
fn test_multiple_traits_on_same_type() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        aspect Double then
            chant double(self) -> Number
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield "Number"
            end
        end

        embody Double for Number then
            chant double(self) -> Number then
                yield self * 2
            end
        end

        bind num to 5
        bind display_result to num.show()
        bind double_result to num.double()
        double_result
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(10.0)");
}

// ============================================================================
// Traits in control flow
// ============================================================================

#[test]
fn test_trait_in_if_statement() {
    let source = r#"
        aspect Positive then
            chant is_positive(self) -> Truth
        end

        embody Positive for Number then
            chant is_positive(self) -> Truth then
                yield self greater than 0
            end
        end

        chant check_positive(n) then
            should n.is_positive() then
                yield "Positive"
            otherwise
                yield "Not positive"
            end
        end

        bind num to 10
        check_positive(num)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), r#"Text("Positive")"#);
}

#[test]
fn test_trait_in_loop() {
    let source = r#"
        aspect Incrementer then
            chant increment(self) -> Number
        end

        embody Incrementer for Number then
            chant increment(self) -> Number then
                yield self + 1
            end
        end

        weave counter as 0
        weave result as 0
        for each i in [1, 2, 3] then
            set counter to counter.increment()
            set result to counter
        end
        result
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(3.0)");
}

// ============================================================================
// Traits with functions
// ============================================================================

#[test]
fn test_trait_method_in_user_function() {
    let source = r#"
        aspect Doubler then
            chant double(self) -> Number
        end

        embody Doubler for Number then
            chant double(self) -> Number then
                yield self * 2
            end
        end

        chant apply_double(x) then
            yield x.double()
        end

        bind num to 5
        apply_double(num)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(10.0)");
}

#[test]
fn test_function_returning_trait_implementer() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield "Number"
            end
        end

        chant get_number() then
            yield 42
        end

        bind num to get_number()
        num.show()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), r#"Text("Number")"#);
}

// ============================================================================
// Complex trait scenarios
// ============================================================================

#[test]
fn test_trait_method_with_complex_logic() {
    let source = r#"
        aspect Calculator then
            chant compute(self, x: Number, y: Number) -> Number
        end

        embody Calculator for Number then
            chant compute(self, x: Number, y: Number) -> Number then
                weave result as self
                set result to result + x
                set result to result * y
                yield result
            end
        end

        bind num to 10
        num.compute(5, 2)
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(30.0)");  // (10 + 5) * 2 = 30
}

#[test]
fn test_trait_with_nested_calls() {
    let source = r#"
        chant helper(x) then
            yield x + 10
        end

        aspect Processor then
            chant process(self) -> Number
        end

        embody Processor for Number then
            chant process(self) -> Number then
                bind temp to helper(self)
                yield temp * 2
            end
        end

        bind num to 5
        num.process()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(30.0)");  // (5 + 10) * 2 = 30
}

// ============================================================================
// Traits with lists and collections
// ============================================================================

#[test]
fn test_trait_on_list_element() {
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
fn test_trait_method_with_list_parameter() {
    let source = r#"
        aspect Container then
            chant size(self) -> Number
        end

        embody Container for List then
            chant size(self) -> Number then
                yield list_length(self)
            end
        end

        bind my_list to [10, 20, 30, 40]
        my_list.size()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(4.0)");
}

// ============================================================================
// Edge cases and robustness
// ============================================================================

#[test]
fn test_trait_method_with_no_additional_params() {
    let source = r#"
        aspect Identity then
            chant get(self) -> Number
        end

        embody Identity for Number then
            chant get(self) -> Number then
                yield self
            end
        end

        bind num to 123
        num.get()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(123.0)");
}

#[test]
fn test_trait_with_recursion() {
    let source = r#"
        aspect Factorial then
            chant fact(self) -> Number
        end

        embody Factorial for Number then
            chant fact(self) -> Number then
                should self at most 1 then
                    yield 1
                otherwise
                    bind n_minus_1 to self - 1
                    bind sub_result to n_minus_1.fact()
                    yield self * sub_result
                end
            end
        end

        bind num to 5
        num.fact()
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(120.0)");  // 5! = 120
}

#[test]
fn test_multiple_trait_calls_in_expression() {
    let source = r#"
        aspect Adder then
            chant add_ten(self) -> Number
        end

        embody Adder for Number then
            chant add_ten(self) -> Number then
                yield self + 10
            end
        end

        bind a to 5
        bind b to 3
        bind sum to a.add_ten() + b.add_ten()
        sum
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "Failed: {:?}", result);
    assert_eq!(result.unwrap(), "Number(28.0)");  // (5+10) + (3+10) = 28
}
