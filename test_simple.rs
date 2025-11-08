use glimmer_weave::{Lexer, Parser};

fn main() {
    let source = r#"chant double_all(borrow mut list as List<Number>) then
    yield list
end"#;
    
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize_positioned();
    
    println!("Tokens:");
    for (i, tok) in tokens.iter().enumerate() {
        println!("[{}] pos={}: {:?}", i, tok.position, tok.token);
    }
    
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(_) => println!("\n✓ Parse succeeded"),
        Err(e) => println!("\n✗ Parse failed: {:?}", e),
    }
}
