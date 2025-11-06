/// Test for the pipeline example program

use glimmer_weave::{Evaluator, Lexer, Parser};
use std::fs;

#[test]
fn test_pipeline_example() {
    // Read the example file
    let source = fs::read_to_string("examples/18_pipeline.gw")
        .expect("Failed to read pipeline example");

    // Parse and evaluate
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("Parse error");

    let mut evaluator = Evaluator::new();
    let result = evaluator.eval(&ast).expect("Runtime error");

    // Result should be a list: [49, 30, 30, 10]
    match result {
        glimmer_weave::eval::Value::List(items) => {
            assert_eq!(items.len(), 4, "Expected 4 results");

            // result1: 49
            match &items[0] {
                glimmer_weave::eval::Value::Number(n) => assert_eq!(*n, 49.0),
                _ => panic!("Expected Number for result1"),
            }

            // result2: 30
            match &items[1] {
                glimmer_weave::eval::Value::Number(n) => assert_eq!(*n, 30.0),
                _ => panic!("Expected Number for result2"),
            }

            // result3: 30
            match &items[2] {
                glimmer_weave::eval::Value::Number(n) => assert_eq!(*n, 30.0),
                _ => panic!("Expected Number for result3"),
            }

            // length: 10
            match &items[3] {
                glimmer_weave::eval::Value::Number(n) => assert_eq!(*n, 10.0),
                _ => panic!("Expected Number for length"),
            }
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}
