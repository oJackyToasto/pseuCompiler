use std::fs;
use std::io::{self, Write};
use std::env;
use crate::parser::Parser;
use crate::interpreter::Interpreter;

pub fn run() {
    let args: Vec<String> = env::args().collect();
    
    // Handle help
    if args.len() == 1 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return;
    }
    
    if args.len() < 2 {
        eprintln!("Error: Missing command");
        print_help();
        std::process::exit(1);
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "eval" => {
            if args.len() == 2 {
                // Interactive mode
                run_interactive();
            } else if args.len() == 3 {
                // Execute file
                let filename = &args[2];
                execute_file(filename);
            } else {
                eprintln!("Error: 'eval' command takes 0 or 1 argument");
                eprintln!("Usage: pseudocode eval [filename]");
                std::process::exit(1);
            }
        }
        "check" => {
            if args.len() != 3 {
                eprintln!("Error: 'check' command requires a filename");
                eprintln!("Usage: pseudocode check <filename>");
                std::process::exit(1);
            }
            let filename = &args[2];
            check_syntax(filename);
        }
        "compile" => {
            if args.len() != 3 {
                eprintln!("Error: 'compile' command requires a filename");
                eprintln!("Usage: pseudocode compile <filename>");
                std::process::exit(1);
            }
            let filename = &args[2];
            compile_file(filename);
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("Usage: pseudocode <command> [arguments]");
    println!();
    println!("Commands:");
    println!("  eval [filename]    Execute pseudocode interactively or from a file");
    println!("                     - 'pseudocode eval'          : Interactive mode (like Python)");
    println!("                     - 'pseudocode eval file.pseu': Execute file");
    println!();
    println!("  check <filename>   Check syntax without executing");
    println!("                     - 'pseudocode check file.pseu'");
    println!();
    println!("  compile <filename> Compile pseudocode (coming soon)");
    println!("                     - 'pseudocode compile file.pseu'");
    println!();
    println!("  --help, -h         Show this help message");
    println!();
    println!("Examples:");
    println!("  pseudocode eval");
    println!("  pseudocode eval program.pseu");
    println!("  pseudocode check program.pseu");
}

fn run_interactive() {
    println!("Pseudocode Interactive Interpreter");
    println!("Type 'exit' or 'quit' to exit, or 'help' for help");
    println!("Press Enter on an empty line to finish multiline input");
    println!();
    
    let mut interpreter = Interpreter::new();
    
    loop {
        // Accumulate multiline input
        let mut input_buffer = String::new();
        let mut line_count = 0;
        
        loop {
            // Show prompt (>>> for first line, ... for continuation)
            if line_count == 0 {
                print!(">>> ");
            } else {
                print!("... ");
            }
            io::stdout().flush().unwrap();
            
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(_) => {
                    let trimmed = line.trim();
                    
                    // Empty line on continuation means "finish input"
                    if line_count > 0 && trimmed.is_empty() {
                        break;
                    }
                    
                    // Empty line on first line means skip
                    if line_count == 0 && trimmed.is_empty() {
                        continue;
                    }
                    
                    // Add line to buffer
                    if !input_buffer.is_empty() {
                        input_buffer.push('\n');
                    }
                    input_buffer.push_str(&line);
                    line_count += 1;
                    
                    // Try to parse to see if we have a complete statement
                    let mut test_parser = Parser::new(&input_buffer.trim());
                    match test_parser.parse_program() {
                        Ok(_) => {
                            // Complete statement, break and execute
                            break;
                        }
                        Err(e) => {
                            // Check if error suggests we need more input
                            let error_lower = e.to_lowercase();
                            if error_lower.contains("unexpected end of file") ||
                               error_lower.contains("was not closed") ||
                               error_lower.contains("expected") && (
                                   error_lower.contains("endfunction") ||
                                   error_lower.contains("endprocedure") ||
                                   error_lower.contains("endtype") ||
                                   error_lower.contains("endif") ||
                                   error_lower.contains("endwhile") ||
                                   error_lower.contains("next") ||
                                   error_lower.contains("until") ||
                                   error_lower.contains("endcase")
                               ) {
                                // Likely incomplete, continue reading
                                continue;
                            } else {
                                // Other parse error, might be complete but invalid
                                // Try one more line in case it's a syntax issue
                                // If still error after next line, break and show error
                                if line_count > 1 {
                                    break;
                                }
                                continue;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    return;
                }
            }
        }
        
        let input = input_buffer.trim();
        
        // Handle special commands FIRST (before parsing)
        // Check on first line to allow early exit
        if line_count == 1 {
            if input == "exit" || input == "quit" {
                println!("Goodbye!");
                break;
            }
            
            if input == "help" {
                println!("Commands:");
                println!("  exit, quit  - Exit the interpreter");
                println!("  help        - Show this help");
                println!("  clear       - Clear the interpreter state");
                println!();
                println!("You can enter any pseudocode statement or expression.");
                println!("For multiline input, press Enter on an empty line to finish.");
                continue;
            }
            
            if input == "clear" {
                interpreter = Interpreter::new();
                println!("Interpreter state cleared.");
                continue;
            }
        }
        
        // Parse and execute
        let mut parser = Parser::new(input);
        match parser.parse_program() {
            Ok(statements) => {
                for stmt in statements {
                    match interpreter.evaluate_stmt(&stmt) {
                        Ok(()) => {
                            // Statement executed successfully
                        }
                        Err(_e) => {
                            // Error already logged by log_error! macro with line numbers
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Parse Error: {}", e);
            }
        }
    }
}

fn execute_file(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => {
            let mut parser = Parser::new(&content);
            match parser.parse_program() {
                Ok(statements) => {
                    let mut interpreter = Interpreter::with_source_file(filename);
                    for stmt in statements.iter() {
                        if let Err(_e) = interpreter.evaluate_stmt(stmt) {
                            // Error already logged by log_error! macro with line numbers
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Parse Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to read file '{}': {}", filename, e);
            std::process::exit(1);
        }
    }
}

fn check_syntax(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => {
            let mut parser = Parser::new(&content);
            match parser.parse_program() {
                Ok(statements) => {
                    println!("Syntax check passed!");
                    println!("Found {} statement(s)", statements.len());
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Syntax Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to read file '{}': {}", filename, e);
            std::process::exit(1);
        }
    }
}

fn compile_file(_filename: &str) {
    eprintln!("Error: Compiler not yet implemented");
    eprintln!("This feature will be available in a future version.");
    std::process::exit(1);
}

