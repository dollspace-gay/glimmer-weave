use glimmer_weave::lexer::Lexer;

fn main() {
    let source = r#"
        chant factorial(n) then
            should n <= 1 then
                yield 1
            otherwise
                yield n * factorial(n - 1)
            end
        end

        factorial(10)
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }
}
