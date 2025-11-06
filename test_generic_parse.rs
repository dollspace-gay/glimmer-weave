use glimmer_weave::{lexer::Lexer, parser::Parser};

fn main() {
    // Test generic function syntax
    let source1 = r#"
        chant identity<T>(x as T) as T then
            yield x
        end
    "#;
    
    let mut lexer = Lexer::new(source1);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => println!("✓ Generic function parsed successfully: {} nodes", ast.len()),
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
    
    // Test generic struct syntax
    let source2 = r#"
        form Box<T> with
            value as T
        end
    "#;
    
    let mut lexer = Lexer::new(source2);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => println!("✓ Generic struct parsed successfully: {} nodes", ast.len()),
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
    
    // Test generic function call
    let source3 = r#"
        chant identity<T>(x as T) as T then
            yield x
        end
        
        bind result to identity<Number>(42)
    "#;
    
    let mut lexer = Lexer::new(source3);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => println!("✓ Generic function call parsed successfully: {} nodes", ast.len()),
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
}
