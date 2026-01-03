mod lexer;
mod parser;
mod ast;

use parser::Parser;
use std::fs;

fn test_expression(input: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Testing Expression: '{}'", input);
    println!("{}", "=".repeat(60));
    
    let mut parser = Parser::new(input);
    match parser.parse_expression() {
        Ok(expr) => {
            println!("✅ Success!");
            println!("AST:\n{:#?}", expr);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn test_statement(input: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Testing Statement:");
    println!("{}", input);
    println!("{}", "=".repeat(60));
    
    let mut parser = Parser::new(input);
    match parser.parse_statement() {
        Ok(stmt) => {
            println!("Success!");
            println!("AST:\n{:#?}", stmt);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn test_program(input: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Testing Program:");
    println!("{}", "=".repeat(60));
    println!("{}", input);
    println!("{}", "=".repeat(60));
    
    let mut parser = Parser::new(input);
    match parser.parse_program() {
        Ok(statements) => {
            println!("Success! Parsed {} statement(s)", statements.len());
            for (i, stmt) in statements.iter().enumerate() {
                println!("\n--- Statement {} ---", i + 1);
                println!("{:#?}", stmt);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn test_file(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => {
            println!("\n{}", "=".repeat(60));
            println!("Testing file: {}", filename);
            println!("{}", "=".repeat(60));
            test_program(&content);
        }
        Err(e) => {
            println!("Failed to read {}: {}", filename, e);
        }
    }
}

fn test_stuff() {
    println!("\n{}", "=".repeat(60));
    println!("EXPRESSION TESTS");
    println!("{}", "=".repeat(60));
    
    // Test simple expressions
    test_expression("5");
    test_expression("x");
    test_expression("5 + 3");
    test_expression("5 * 3 + 2");  // Should be (5 * 3) + 2
    test_expression("5 + 3 * 2");  // Should be 5 + (3 * 2)
    test_expression("NOT x");
    test_expression("-5");
    test_expression("(5 + 3)");
    test_expression("LENGTH(x)");
    test_expression("arr[5]");
    test_expression("x >= 5");
    test_expression("x AND y");
    test_expression("NOT x OR y");
    
    println!("\n{}", "=".repeat(60));
    println!("STATEMENT TESTS");
    println!("{}", "=".repeat(60));
    
    // Test individual statements
    test_statement("DECLARE x : INTEGER");
    test_statement("DECLARE y <- 10 : INTEGER");
    test_statement("x <- 5");
    test_statement("OUTPUT x");
    test_statement("OUTPUT \"Hello\", x");
    test_statement("INPUT x");
    test_statement("BREAK");
    
    // Test control flow
    test_statement("IF x > 5 THEN\n    OUTPUT x\nENDIF");
    test_statement("WHILE x < 10 DO\n    x <- x + 1\nENDWHILE");
    test_statement("FOR i <- 1 TO 10\n    OUTPUT i\nNEXT i");
}

fn main() {
    test_file("code/example0.pseu");
}