mod lexer;
mod parser;
mod ast;
mod log;
mod interpreter;

use parser::Parser;
use interpreter::Interpreter;
use std::fs;
use std::env;

// ============================================================================
// PARSING TESTS (AST only, no execution)
// ============================================================================

fn test_expression_parse(input: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Testing Expression: '{}'", input);
    println!("{}", "=".repeat(60));
    
    let mut parser = Parser::new(input);
    match parser.parse_expression() {
        Ok(expr) => {
            println!("Parse Success!");
            println!("AST:\n{:#?}", expr);
        }
        Err(e) => {
            println!("Parse Error: {}", e);
        }
    }
}

fn test_statement_parse(input: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Testing Statement:");
    println!("{}", input);
    println!("{}", "=".repeat(60));
    
    let mut parser = Parser::new(input);
    match parser.parse_statement() {
        Ok(stmt) => {
            println!("Parse Success!");
            println!("AST:\n{:#?}", stmt);
        }
        Err(e) => {
            println!("Parse Error: {}", e);
        }
    }
}

fn test_statement_execute(input: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Executing Statement:");
    println!("{}", input);
    println!("{}", "=".repeat(60));
    
    let mut parser = Parser::new(input);
    match parser.parse_statement() {
        Ok(stmt) => {
            println!("Parse Success!");
            let mut interpreter = Interpreter::new();
            match interpreter.evaluate_stmt(&stmt) {
                Ok(()) => {
                    println!("Execution Success!");
                }
                Err(e) => {
                    println!("Execution Error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Parse Error: {}", e);
        }
    }
}

fn test_program_parse(input: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Testing Program:");
    println!("{}", "=".repeat(60));
    println!("{}", input);
    println!("{}", "=".repeat(60));
    
    let mut parser = Parser::new(input);
    match parser.parse_program() {
        Ok(statements) => {
            println!("Parse Success! Parsed {} statement(s)", statements.len());
            for (i, stmt) in statements.iter().enumerate() {
                println!("\n--- Statement {} ---", i + 1);
                println!("{:#?}", stmt);
            }
        }
        Err(e) => {
            println!("Parse Error: {}", e);
        }
    }
}

// ============================================================================
// EXECUTION TESTS (Parse + Execute with Interpreter)
// ============================================================================

fn test_expression_execute(input: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Testing Expression: '{}'", input);
    println!("{}", "=".repeat(60));
    
    let mut parser = Parser::new(input);
    match parser.parse_expression() {
        Ok(expr) => {
            println!("Parse Success!");
            let mut interpreter = Interpreter::new();
            match interpreter.evaluate_expr(&expr) {
                Ok(value) => {
                    println!("Execution Success!");
                    println!("Result: {:?}", value);
                }
                Err(e) => {
                    println!("Execution Error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Parse Error: {}", e);
        }
    }
}

fn test_program_execute(input: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Executing Program:");
    println!("{}", "=".repeat(60));
    println!("{}", input);
    println!("{}", "=".repeat(60));
    
    let mut parser = Parser::new(input);
    match parser.parse_program() {
        Ok(statements) => {
            println!("Parse Success! Parsed {} statement(s)", statements.len());
            
            let mut interpreter = Interpreter::new();
            for (i, stmt) in statements.iter().enumerate() {
                println!("\n--- Executing Statement {} ---", i + 1);
                match interpreter.evaluate_stmt(stmt) {
                    Ok(()) => {
                        println!("Statement {} executed successfully", i + 1);
                    }
                    Err(e) => {
                        println!("Execution Error at statement {}: {}", i + 1, e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("Parse Error: {}", e);
        }
    }
}

fn test_file_parse(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => {
            println!("\n{}", "=".repeat(60));
            println!("Testing file: {}", filename);
            println!("{}", "=".repeat(60));
            test_program_parse(&content);
        }
        Err(e) => {
            println!("Failed to read {}: {}", filename, e);
        }
    }
}

fn execute_file_silent(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => {
            let mut parser = Parser::new(&content);
            match parser.parse_program() {
                Ok(statements) => {
                    let mut interpreter = Interpreter::new();
                    for stmt in statements.iter() {
                        if let Err(e) = interpreter.evaluate_stmt(stmt) {
                            eprintln!("Error: {}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Parse Error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read {}: {}", filename, e);
        }
    }
}

fn test_file_execute(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => {
            println!("\n{}", "=".repeat(60));
            println!("Executing file: {}", filename);
            println!("{}", "=".repeat(60));
            test_program_execute(&content);
        }
        Err(e) => {
            println!("Failed to read {}: {}", filename, e);
        }
    }
}

// ============================================================================
// TEST SUITES
// ============================================================================

fn run_expression_tests() {
    println!("\n{}", "=".repeat(60));
    println!("EXPRESSION PARSE TESTS");
    println!("{}", "=".repeat(60));
    
    test_expression_parse("5");
    test_expression_parse("5.0");
    test_expression_parse("\"hello\"");
    test_expression_parse("'A'");
    test_expression_parse("5 + 3");
    test_expression_parse("5 * 3 + 2");
    test_expression_parse("5 + 3 * 2");
    test_expression_parse("(5 + 3)");
    test_expression_parse("LENGTH(\"test\")");
}

fn run_statement_tests() {
    println!("\n{}", "=".repeat(60));
    println!("STATEMENT PARSE TESTS");
    println!("{}", "=".repeat(60));
    
    test_statement_parse("DECLARE x : INTEGER");
    test_statement_parse("DECLARE y <- 10 : INTEGER");
    test_statement_parse("OUTPUT \"Hello\"");
    test_statement_parse("OUTPUT \"Hello\", x");
}

// ============================================================================
// MAIN
// ============================================================================

fn print_usage() {
    println!("Usage:");
    println!("  cargo run [mode] [input]");
    println!();
    println!("Modes:");
    println!("  expr <expression>     - Parse and show AST for expression");
    println!("  exec <code>           - Parse and execute code (multi-line supported)");
    println!("  stmt <statement>      - Parse and show AST for statement");
    println!("  execstmt <statement>   - Parse and execute single statement");
    println!("  file <filename>       - Execute file (output only)");
    println!("  run <filename>        - Parse and execute file");
    println!("  test                  - Run test suite");
    println!("  help                  - Show this help");
    println!();
    println!("Examples:");
    println!("  cargo run expr \"5 + 3\"");
    println!("  cargo run exec \"DECLARE x <- 10 : INTEGER\"");
    println!("  cargo run exec \"DECLARE x <- 10 : INTEGER\\nOUTPUT x\"");
    println!("  cargo run stmt \"DECLARE x : INTEGER\"");
    println!("  cargo run execstmt \"DECLARE x <- 10 : INTEGER\"");
    println!("  cargo run file code/example0.pseu");
    println!("  cargo run run code/example0.pseu");
    println!("  cargo run test");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    let mode = &args[1];
    
    // Set log level based on mode - suppress for file mode
    if mode == "file" {
        std::env::set_var("RUST_LOG", "error");
    }
    
    // Initialize logger
    log::init();
    
    match mode.as_str() {
        "expr" => {
            if args.len() < 3 {
                println!("Error: 'expr' mode requires an expression");
                println!("Example: cargo run expr \"5 + 3\"");
                return;
            }
            test_expression_parse(&args[2]);
        }
        "exec" => {
            if args.len() < 3 {
                println!("Error: 'exec' mode requires code to execute");
                println!("Example: cargo run exec \"DECLARE x <- 10 : INTEGER\"");
                return;
            }
            test_program_execute(&args[2]);
        }
        "stmt" => {
            if args.len() < 3 {
                println!("Error: 'stmt' mode requires a statement");
                println!("Example: cargo run stmt \"OUTPUT 5\"");
                return;
            }
            test_statement_parse(&args[2]);
        }
        "execstmt" => {
            if args.len() < 3 {
                println!("Error: 'execstmt' mode requires a statement");
                println!("Example: cargo run execstmt \"DECLARE x <- 10 : INTEGER\"");
                return;
            }
            test_statement_execute(&args[2]);
        }
        "file" => {
            if args.len() < 3 {
                println!("Error: 'file' mode requires a filename");
                println!("Example: cargo run file code/example0.pseu");
                return;
            }
            execute_file_silent(&args[2]);
        }
        "run" => {
            if args.len() < 3 {
                println!("Error: 'run' mode requires a filename");
                println!("Example: cargo run run code/example0.pseu");
                return;
            }
            test_file_execute(&args[2]);
        }
        "test" => {
            run_expression_tests();
            run_statement_tests();
        }
        "help" | "-h" | "--help" => {
            print_usage();
        }
        _ => {
            println!("Unknown mode: {}", mode);
            print_usage();
        }
    }
}