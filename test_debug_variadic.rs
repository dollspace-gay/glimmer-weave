use glimmer_weave::*;

fn main() {
    let source = r#"
chant max(...values) then
    weave current_max as values[0]
    yield current_max
end

max(3, 7, 2)
"#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    println!("Tokens: {:?}", tokens);
    
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => println!("AST: {:?}", ast),
        Err(e) => println!("Parse error: {:?}", e),
    }
}
