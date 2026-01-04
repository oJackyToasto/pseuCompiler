mod lexer;
mod parser;
mod ast;
mod log;
mod interpreter;
mod cli;

fn main() {
    // Initialize logger
    log::init();
    
    // Run CLI
    cli::run();
}
