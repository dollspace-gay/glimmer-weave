// Test that all example files parse correctly

use glimmer_weave::{Lexer, Parser};
use std::fs;
use std::path::Path;

fn test_example_file(path: &Path) -> Result<(), String> {
    let filename = path.file_name().unwrap().to_str().unwrap();
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", filename, e))?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize_positioned();

    let mut parser = Parser::new(tokens);
    parser.parse()
        .map_err(|e| format!("Parse error in {}: {:?}", filename, e))?;

    Ok(())
}

#[test]
fn test_all_examples() {
    let examples_dir = Path::new("examples");

    let mut files: Vec<_> = fs::read_dir(examples_dir)
        .expect("Failed to read examples directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "gw")
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .collect();

    files.sort();

    let mut errors = Vec::new();

    for path in &files {
        println!("Testing: {}", path.display());
        if let Err(e) = test_example_file(path) {
            errors.push(e);
        }
    }

    if !errors.is_empty() {
        eprintln!("\n{} example(s) failed to parse:", errors.len());
        for error in &errors {
            eprintln!("  - {}", error);
        }
        panic!("{} example files have errors", errors.len());
    }

    println!("\n✓ All {} examples parsed successfully!", files.len());
}

#[test]
fn debug_arrow_with_generics() {
    let source = r#"chant sum(borrow list as List<Number>) -> Number then
    yield 42
end"#;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize_positioned();

    println!("\nTokens:");
    for (i, tok) in tokens.iter().enumerate() {
        println!("[{}]: {:?}", i, tok.token);
    }

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(_) => println!("\n✓ Parse succeeded"),
        Err(e) => println!("\n✗ Parse failed: {:?}", e),
    }
}
