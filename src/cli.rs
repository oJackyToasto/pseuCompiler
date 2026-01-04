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
    println!("Type 'exit' or 'quit' to exit, or 'help' for help");
    println!();
    
    let mut interpreter = Interpreter::new();
    let mut line_buffer = String::new();
    
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        
        line_buffer.clear();
        match io::stdin().read_line(&mut line_buffer) {
            Ok(_) => {
                let line = line_buffer.trim();
                
                // Handle special commands
                if line.is_empty() {
                    continue;
                }
                
                if line == "exit" || line == "quit" {
                    println!("Goodbye!");
                    break;
                }
                
                if line == "help" {
                    println!("Commands:");
                    println!("  exit, quit  - Exit the interpreter");
                    println!("  help        - Show this help");
                    println!("  clear       - Clear the interpreter state");
                    println!();
                    println!("You can enter any pseudocode statement or expression.");
                    continue;
                }
                
                if line == "clear" {
                    interpreter = Interpreter::new();
                    println!("Interpreter state cleared.");
                    continue;
                }
                
                // Parse and execute using parse_program
                let mut parser = Parser::new(line);
                match parser.parse_program() {
                    Ok(statements) => {
                        for stmt in statements {
                            match interpreter.evaluate_stmt(&stmt) {
                                Ok(()) => {
                                    // Statement executed successfully
                                }
                                Err(e) => {
                                    eprintln!("Error: {}", e);
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
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
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
                    let mut interpreter = Interpreter::new();
                    for stmt in statements.iter() {
                        if let Err(e) = interpreter.evaluate_stmt(stmt) {
                            eprintln!("Error: {}", e);
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

