/// Tests for trait system parsing (Phase 1)
/// Verifies that aspect definitions and embody statements can be parsed correctly

use glimmer_weave::{Lexer, Parser};

fn parse_source(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize_positioned();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    Ok(())
}

#[test]
fn test_parse_simple_aspect_definition() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end
    "#;

    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse simple aspect: {:?}", result);
}

#[test]
fn test_parse_aspect_with_multiple_methods() {
    let source = r#"
        aspect Comparable then
            chant compare(self, other) -> Number
            chant is_equal(self, other) -> Truth
        end
    "#;

    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse aspect with multiple methods: {:?}", result);
}

#[test]
fn test_parse_generic_aspect() {
    let source = r#"
        aspect Container<T> then
            chant add(self, item: T)
            chant get(self, index: Number) -> T
        end
    "#;

    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse generic aspect: {:?}", result);
}

#[test]
fn test_parse_simple_embody_statement() {
    let source = r#"
        embody Display for Number then
            chant show(self) -> Text then
                yield to_text(self)
            end
        end
    "#;

    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse simple embody: {:?}", result);
}

#[test]
fn test_parse_embody_with_generic_trait() {
    let source = r#"
        embody Container<Number> for NumberList then
            chant add(self, item: Number) then
                yield list_append(self, item)
            end

            chant get(self, index: Number) -> Number then
                yield list_get(self, index)
            end
        end
    "#;

    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse embody with generic trait: {:?}", result);
}

#[test]
fn test_parse_complete_trait_example() {
    let source = r#"
        aspect Display then
            chant show(self) -> Text
        end

        embody Display for Number then
            chant show(self) -> Text then
                yield to_text(self)
            end
        end

        chant print_value(x: Number) then
            bind text to x.show()
            reveal(text)
        end
    "#;

    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse complete trait example: {:?}", result);
}

#[test]
fn test_parse_aspect_with_method_parameters() {
    let source = r#"
        aspect Formatter then
            chant format(self, width: Number, padding: Text) -> Text
        end
    "#;

    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse aspect with method parameters: {:?}", result);
}

#[test]
fn test_parse_multi_param_generic_aspect() {
    let source = r#"
        aspect Mapper<T, U> then
            chant map(self, func: Function) -> U
        end
    "#;

    let result = parse_source(source);
    assert!(result.is_ok(), "Failed to parse multi-param generic aspect: {:?}", result);
}
