use glimmer_weave::{Lexer, Parser};

fn main() {
    let src = r#"
chant sum(borrow list as List<Number>) then
    yield 42
end
"#;

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
