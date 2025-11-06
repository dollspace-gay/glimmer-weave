use glimmer_weave::{lexer::Lexer, parser::Parser};

fn main() {
    println!("Testing Generic Syntax Parsing\n");
    
    // Test 1: Generic function syntax
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
    
    // Test 2: Generic struct syntax
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
    
    // Test 3: Generic function call with type arguments
    let source3 = r#"
        bind result to identity<Number>(42)
    "#;
    
    let mut lexer = Lexer::new(source3);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => println!("✓ Generic function call parsed successfully: {} nodes", ast.len()),
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
    
    // Test 4: Generic struct instantiation
    let source4 = r#"
        bind my_box to Box<Number> { value: 42 }
    "#;
    
    let mut lexer = Lexer::new(source4);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => println!("✓ Generic struct instantiation parsed successfully: {} nodes", ast.len()),
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
    
    println!("\n✨ Generic syntax parsing complete!");
}
