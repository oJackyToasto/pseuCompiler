mod lexer;
mod parser;
mod ast;

use lexer::{Token, Lexer};
use parser::Parser;
use std::fs;

fn test_parser(input: &str) {
    println!("\n{}", "=".repeat(50));
    println!("Testing: '{}'", input);
    println!("{}", "=".repeat(50));
    
    let mut parser = Parser::new(input);
    match parser.parse_expression() {
        Ok(expr) => {
            println!("✅ Success!");
            println!("AST: {:#?}", expr);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
}

fn main() {
    // Test simple expressions
    test_parser("5");
    test_parser("x");
    test_parser("5 + 3");
    test_parser("5 * 3 + 2");  // Should be (5 * 3) + 2
    test_parser("5 + 3 * 2");  // Should be 5 + (3 * 2)
    test_parser("NOT x");
    test_parser("-5");
    test_parser("(5 + 3)");
    test_parser("LENGTH(x)");
    test_parser("arr[5]");
    test_parser("x >= 5");
    test_parser("x AND y");
    test_parser("NOT x OR y");
}