/// Tests for expanded standard library functions
/// Tests new string, math, and list operations added in glimmer-weave-w19

use glimmer_weave::*;

fn run_program(source: &str) -> Result<eval::Value, eval::RuntimeError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| {
        eval::RuntimeError::Custom(format!("Parse error: {:?}", e))
    })?;
    let mut evaluator = Evaluator::new();
    evaluator.eval(&ast)
}

// ============================================================================
// STRING FUNCTION TESTS
// ============================================================================

#[test]
fn test_string_replace() {
    let source = r#"
        bind text to "hello world"
        replace(text, "world", "Glimmer")
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Text(s) => assert_eq!(s, "hello Glimmer"),
        _ => panic!("Expected Text, got {:?}", result),
    }
}

#[test]
fn test_string_char_at() {
    let source = r#"
        bind text to "Glimmer"
        char_at(text, 0)
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Text(s) => assert_eq!(s, "G"),
        _ => panic!("Expected Text, got {:?}", result),
    }
}

#[test]
fn test_string_repeat() {
    let source = r#"
        repeat("ha", 3)
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Text(s) => assert_eq!(s, "hahaha"),
        _ => panic!("Expected Text, got {:?}", result),
    }
}

#[test]
fn test_string_pad_left() {
    let source = r#"
        pad_left("42", 5, "0")
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Text(s) => assert_eq!(s, "00042"),
        _ => panic!("Expected Text, got {:?}", result),
    }
}

#[test]
fn test_string_pad_right() {
    let source = r#"
        pad_right("42", 5, "0")
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Text(s) => assert_eq!(s, "42000"),
        _ => panic!("Expected Text, got {:?}", result),
    }
}

#[test]
fn test_string_reverse() {
    let source = r#"
        reverse("hello")
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Text(s) => assert_eq!(s, "olleh"),
        _ => panic!("Expected Text, got {:?}", result),
    }
}

// ============================================================================
// MATH FUNCTION TESTS
// ============================================================================

#[test]
fn test_math_sign() {
    let source = r#"
        [sign(5), sign(-3), sign(0)]
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], eval::Value::Number(1.0));
            assert_eq!(items[1], eval::Value::Number(-1.0));
            assert_eq!(items[2], eval::Value::Number(0.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_math_clamp() {
    let source = r#"
        [clamp(5, 0, 10), clamp(-5, 0, 10), clamp(15, 0, 10)]
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], eval::Value::Number(5.0));
            assert_eq!(items[1], eval::Value::Number(0.0));
            assert_eq!(items[2], eval::Value::Number(10.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_math_sin_cos() {
    let source = r#"
        # Test sin(0) = 0, cos(0) = 1
        [sin(0), cos(0)]
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::List(items) => {
            assert_eq!(items.len(), 2);
            match &items[0] {
                eval::Value::Number(n) => assert!(n.abs() < 1e-10),
                _ => panic!("Expected Number"),
            }
            match &items[1] {
                eval::Value::Number(n) => assert!((n - 1.0).abs() < 1e-10),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_math_log_exp() {
    let source = r#"
        # Test log(exp(1)) ≈ 1
        bind x to exp(1)
        log(x)
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Number(n) => assert!((n - 1.0).abs() < 1e-10),
        _ => panic!("Expected Number, got {:?}", result),
    }
}

// ============================================================================
// LIST FUNCTION TESTS
// ============================================================================

#[test]
fn test_list_concat() {
    let source = r#"
        bind a to [1, 2, 3]
        bind b to [4, 5, 6]
        list_concat(a, b)
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::List(items) => {
            assert_eq!(items.len(), 6);
            assert_eq!(items[0], eval::Value::Number(1.0));
            assert_eq!(items[5], eval::Value::Number(6.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_list_slice() {
    let source = r#"
        bind lst to [1, 2, 3, 4, 5]
        list_slice(lst, 1, 4)
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], eval::Value::Number(2.0));
            assert_eq!(items[1], eval::Value::Number(3.0));
            assert_eq!(items[2], eval::Value::Number(4.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_list_flatten() {
    let source = r#"
        bind nested to [[1, 2], [3, 4], [5]]
        list_flatten(nested)
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::List(items) => {
            assert_eq!(items.len(), 5);
            assert_eq!(items[0], eval::Value::Number(1.0));
            assert_eq!(items[4], eval::Value::Number(5.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_list_sum() {
    let source = r#"
        bind numbers to [1, 2, 3, 4, 5]
        list_sum(numbers)
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Number(n) => assert_eq!(n, 15.0),
        _ => panic!("Expected Number, got {:?}", result),
    }
}

#[test]
fn test_list_product() {
    let source = r#"
        bind numbers to [2, 3, 4]
        list_product(numbers)
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Number(n) => assert_eq!(n, 24.0),
        _ => panic!("Expected Number, got {:?}", result),
    }
}

#[test]
fn test_list_min_max() {
    let source = r#"
        bind numbers to [5, 2, 8, 1, 9, 3]
        [list_min(numbers), list_max(numbers)]
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::List(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], eval::Value::Number(1.0));
            assert_eq!(items[1], eval::Value::Number(9.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_list_contains() {
    let source = r#"
        bind numbers to [1, 2, 3, 4, 5]
        [list_contains(numbers, 3), list_contains(numbers, 10)]
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::List(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], eval::Value::Truth(true));
            assert_eq!(items[1], eval::Value::Truth(false));
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_list_index_of() {
    let source = r#"
        bind items to ["a", "b", "c", "d"]
        [list_index_of(items, "b"), list_index_of(items, "z")]
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::List(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], eval::Value::Number(1.0));
            assert_eq!(items[1], eval::Value::Number(-1.0)); // Not found
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

// ============================================================================
// INTEGRATION TESTS - Complex operations using multiple new functions
// ============================================================================

#[test]
fn test_string_processing_pipeline() {
    let source = r#"
        # Process text: trim, uppercase, pad, repeat
        bind text to "  hello  "
        bind step1 to trim(text)
        bind step2 to upper(step1)
        bind step3 to pad_left(step2, 10, "*")
        step3
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Text(s) => assert_eq!(s, "*****HELLO"),
        _ => panic!("Expected Text, got {:?}", result),
    }
}

#[test]
fn test_list_statistics() {
    let source = r#"
        # Calculate statistics on a list
        bind numbers to [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

        bind total to list_sum(numbers)
        bind avg to total / 10
        bind minimum to list_min(numbers)
        bind maximum to list_max(numbers)

        [total, avg, minimum, maximum]
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::List(items) => {
            assert_eq!(items.len(), 4);
            assert_eq!(items[0], eval::Value::Number(55.0));  // sum
            assert_eq!(items[1], eval::Value::Number(5.5));   // avg
            assert_eq!(items[2], eval::Value::Number(1.0));   // min
            assert_eq!(items[3], eval::Value::Number(10.0));  // max
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_nested_list_operations() {
    let source = r#"
        # Work with nested lists
        bind nested to [[1, 2], [3, 4], [5, 6]]
        bind flat to list_flatten(nested)
        bind doubled to [flat[0] * 2, flat[1] * 2, flat[2] * 2]
        bind total to list_sum(doubled)
        total
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Number(n) => assert_eq!(n, 12.0), // (1+2+3)*2 = 12
        _ => panic!("Expected Number, got {:?}", result),
    }
}

#[test]
fn test_math_trigonometry() {
    let source = r#"
        # Pythagorean identity: sin²(x) + cos²(x) = 1
        bind x to 0.5
        bind sin_x to sin(x)
        bind cos_x to cos(x)
        bind sin_squared to pow(sin_x, 2)
        bind cos_squared to pow(cos_x, 2)
        bind identity to sin_squared + cos_squared
        identity
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Number(n) => assert!((n - 1.0).abs() < 1e-10),
        _ => panic!("Expected Number, got {:?}", result),
    }
}

#[test]
fn test_string_manipulation_chain() {
    let source = r#"
        # Chain multiple string operations
        bind orig to "Hello"
        bind rev to reverse(orig)
        bind upper_rev to upper(rev)
        bind repeated to repeat(upper_rev, 2)
        repeated
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::Text(s) => assert_eq!(s, "OLLEHOLLEH"),
        _ => panic!("Expected Text, got {:?}", result),
    }
}

#[test]
fn test_list_slicing_and_concat() {
    let source = r#"
        # Slice and recombine lists
        bind original to [1, 2, 3, 4, 5, 6, 7, 8]
        bind first_half to list_slice(original, 0, 4)
        bind second_half to list_slice(original, 4, 8)
        bind reversed_first to list_reverse(first_half)
        list_concat(reversed_first, second_half)
    "#;
    let result = run_program(source).expect("Should succeed");
    match result {
        eval::Value::List(items) => {
            assert_eq!(items.len(), 8);
            assert_eq!(items[0], eval::Value::Number(4.0));
            assert_eq!(items[1], eval::Value::Number(3.0));
            assert_eq!(items[2], eval::Value::Number(2.0));
            assert_eq!(items[3], eval::Value::Number(1.0));
            assert_eq!(items[4], eval::Value::Number(5.0));
            assert_eq!(items[7], eval::Value::Number(8.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

// ============================================================================
// ERROR CASE TESTS
// ============================================================================

#[test]
fn test_char_at_out_of_bounds() {
    let source = r#"
        char_at("hello", 10)
    "#;
    let result = run_program(source);
    assert!(result.is_err(), "Should fail for out of bounds");
}

#[test]
fn test_log_negative() {
    let source = r#"
        log(-1)
    "#;
    let result = run_program(source);
    assert!(result.is_err(), "Should fail for negative number");
}

#[test]
fn test_list_min_empty() {
    let source = r#"
        list_min([])
    "#;
    let result = run_program(source);
    assert!(result.is_err(), "Should fail for empty list");
}

#[test]
fn test_clamp_invalid_range() {
    let source = r#"
        clamp(5, 10, 0)
    "#;
    let result = run_program(source);
    assert!(result.is_err(), "Should fail when min > max");
}
