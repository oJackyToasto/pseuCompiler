mod lexer;
mod parser;
mod ast;
mod wasm_interpreter;

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use crate::wasm_interpreter::WasmInterpreter;
use crate::parser::Parser;

// Initialize panic hook for better error messages in the browser
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[derive(Serialize, Deserialize)]
pub struct ExecutionResult {
    pub output: String,
    pub errors: Vec<ErrorInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorInfo {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SyntaxCheckResult {
    pub valid: bool,
    pub errors: Vec<ErrorInfo>,
}

#[wasm_bindgen]
pub struct PseudocodeEngine {
    interpreter: WasmInterpreter,
}

#[wasm_bindgen]
impl PseudocodeEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> PseudocodeEngine {
        PseudocodeEngine {
            interpreter: WasmInterpreter::new(),
        }
    }

    /// Execute pseudocode and return results
    #[wasm_bindgen]
    pub fn execute(&mut self, code: &str) -> JsValue {
        // Clear previous output
        self.interpreter.clear_output();
        
        // Parse the code
        let mut parser = Parser::new(code);
        let statements = match parser.parse_program() {
            Ok(stmts) => stmts,
            Err(e) => {
                // Extract line number from error if possible
                let error_info = ErrorInfo {
                    message: e.clone(),
                    line: 1,
                    column: 1,
                };
                return serde_wasm_bindgen::to_value(&ExecutionResult {
                    output: String::new(),
                    errors: vec![error_info],
                }).unwrap();
            }
        };

        // Execute statements
        let mut errors = Vec::new();
        for stmt in &statements {
            if let Err(e) = self.interpreter.evaluate_stmt(stmt) {
                // Try to extract line number from error message
                let line = if let Some(span) = get_stmt_span(stmt) {
                    span.line
                } else {
                    1
                };
                errors.push(ErrorInfo {
                    message: e,
                    line,
                    column: 1,
                });
            }
        }

        let output = self.interpreter.get_output().to_string();
        
        serde_wasm_bindgen::to_value(&ExecutionResult {
            output,
            errors,
        }).unwrap()
    }

    /// Check syntax without executing
    #[wasm_bindgen]
    pub fn check_syntax(&self, code: &str) -> JsValue {
        let mut parser = Parser::new(code);
        match parser.parse_program() {
            Ok(_) => {
                serde_wasm_bindgen::to_value(&SyntaxCheckResult {
                    valid: true,
                    errors: Vec::new(),
                }).unwrap()
            }
            Err(e) => {
                let error_info = ErrorInfo {
                    message: e,
                    line: 1,
                    column: 1,
                };
                serde_wasm_bindgen::to_value(&SyntaxCheckResult {
                    valid: false,
                    errors: vec![error_info],
                }).unwrap()
            }
        }
    }

    /// Set a virtual file in the file system
    #[wasm_bindgen]
    pub fn set_virtual_file(&mut self, filename: String, content: String) {
        self.interpreter.set_virtual_file(filename, content);
    }

    /// Get a virtual file from the file system
    #[wasm_bindgen]
    pub fn get_virtual_file(&self, filename: &str) -> Option<String> {
        self.interpreter.get_virtual_file(filename).cloned()
    }

    /// Add input to the input queue
    #[wasm_bindgen]
    pub fn add_input(&mut self, input: String) {
        self.interpreter.add_input(input);
    }
}

// Helper function to extract span from statement
fn get_stmt_span(stmt: &crate::ast::Stmt) -> Option<crate::ast::Span> {
    match stmt {
        crate::ast::Stmt::Declare { span, .. } => Some(span.clone()),
        crate::ast::Stmt::Assign { span, .. } => Some(span.clone()),
        crate::ast::Stmt::Output { span, .. } => Some(span.clone()),
        crate::ast::Stmt::Input { span, .. } => Some(span.clone()),
        crate::ast::Stmt::If { span, .. } => Some(span.clone()),
        crate::ast::Stmt::While { span, .. } => Some(span.clone()),
        crate::ast::Stmt::For { span, .. } => Some(span.clone()),
        crate::ast::Stmt::FunctionDeclaration { span, .. } => Some(span.clone()),
        crate::ast::Stmt::ProcedureDeclaration { span, .. } => Some(span.clone()),
        _ => None,
    }
}

