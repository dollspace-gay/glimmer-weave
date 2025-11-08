use glimmer_weave::{Lexer, Parser};

fn main() {
    let src = std::fs::read_to_string("test_bind_call.gw").unwrap();
    let mut lexer = Lexer::new(&src);
    let tokens = lexer.tokenize_positioned();
    
    println!("Tokens:");
    for (i, tok) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, tok.token);
    }
    
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => println!("\nParse OK: {} nodes", ast.len()),
        Err(e) => println!("\nParse error: {:?}", e),
    }
}
