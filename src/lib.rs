mod lexer;
mod parser;
mod ast;
mod wasm_interpreter;
mod language_service;

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use crate::wasm_interpreter::WasmInterpreter;
use crate::parser::Parser;
use crate::language_service::{CompletionProvider, HoverProvider, CompletionItemKind};

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

#[derive(Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: String, // "keyword", "function", "variable", "constant", "type"
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: String,
}

#[derive(Serialize, Deserialize)]
pub struct CompletionResult {
    pub items: Vec<CompletionItem>,
}

#[derive(Serialize, Deserialize)]
pub struct HoverInfo {
    pub contents: String,
}

#[derive(Serialize, Deserialize)]
pub struct StatementInfo {
    pub is_input: bool,
    pub input_var_name: Option<String>,
    pub line: usize,
}

#[wasm_bindgen]
pub struct PseudocodeEngine {
    interpreter: WasmInterpreter,
    #[wasm_bindgen(skip)]
    parsed_statements: Vec<crate::ast::Stmt>,
    #[wasm_bindgen(skip)]
    current_statement_index: usize,
}

#[wasm_bindgen]
impl PseudocodeEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> PseudocodeEngine {
        PseudocodeEngine {
            interpreter: WasmInterpreter::new(),
            parsed_statements: Vec::new(),
            current_statement_index: 0,
        }
    }

    /// Parse code and prepare for step-by-step execution
    #[wasm_bindgen]
    pub fn parse_for_execution(&mut self, code: &str) -> JsValue {
        // Clear previous state
        self.interpreter.clear_output();
        self.parsed_statements.clear();
        self.current_statement_index = 0;
        
        // Parse the code
        let mut parser = Parser::new(code);
        match parser.parse_program() {
            Ok(stmts) => {
                self.parsed_statements = stmts;
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

    /// Get information about the next statement to execute
    #[wasm_bindgen]
    pub fn get_next_statement_info(&self) -> JsValue {
        if self.current_statement_index >= self.parsed_statements.len() {
            return serde_wasm_bindgen::to_value(&StatementInfo {
                is_input: false,
                input_var_name: None,
                line: 0,
            }).unwrap();
        }
        
        let stmt = &self.parsed_statements[self.current_statement_index];
        let (is_input, input_var_name) = match stmt {
            crate::ast::Stmt::Input { name, .. } => (true, Some(name.clone())),
            _ => (false, None),
        };
        
        let line = get_stmt_span(stmt).map(|s| s.line).unwrap_or(0);
        
        serde_wasm_bindgen::to_value(&StatementInfo {
            is_input,
            input_var_name,
            line,
        }).unwrap()
    }

    /// Execute the next statement and return output since last call
    #[wasm_bindgen]
    pub fn execute_next_statement(&mut self) -> JsValue {
        if self.current_statement_index >= self.parsed_statements.len() {
            return serde_wasm_bindgen::to_value(&ExecutionResult {
                output: String::new(),
                errors: Vec::new(),
            }).unwrap();
        }
        
        // Get output before execution
        let output_before = self.interpreter.get_output().to_string();
        
        // Execute the statement
        let stmt = &self.parsed_statements[self.current_statement_index];
        let mut errors = Vec::new();
        let output_after = if let Err(e) = self.interpreter.evaluate_stmt(stmt) {
            let line = get_stmt_span(stmt).map(|s| s.line).unwrap_or(1);
            errors.push(ErrorInfo {
                message: e,
                line,
                column: 1,
            });
            self.interpreter.get_output().to_string()
        } else {
            self.interpreter.get_output().to_string()
        };
        
        // Calculate new output (difference)
        let new_output = if output_after.len() > output_before.len() {
            output_after[output_before.len()..].to_string()
        } else {
            String::new()
        };
        
        self.current_statement_index += 1;
        
        serde_wasm_bindgen::to_value(&ExecutionResult {
            output: new_output,
            errors,
        }).unwrap()
    }

    /// Check if there are more statements to execute
    #[wasm_bindgen]
    pub fn has_more_statements(&self) -> bool {
        self.current_statement_index < self.parsed_statements.len()
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

    /// Clear the input queue
    #[wasm_bindgen]
    pub fn clear_inputs(&mut self) {
        self.interpreter.clear_inputs();
    }

    /// Get all INPUT statements from code (variable names in order)
    #[wasm_bindgen]
    pub fn get_input_statements(&self, code: &str) -> JsValue {
        let mut parser = Parser::new(code);
        let statements = match parser.parse_program() {
            Ok(stmts) => stmts,
            Err(_) => {
                // Return empty array if parse fails
                return serde_wasm_bindgen::to_value(&Vec::<String>::new()).unwrap();
            }
        };

        let mut input_vars = Vec::new();
        extract_input_statements(&statements, &mut input_vars);
        
        serde_wasm_bindgen::to_value(&input_vars).unwrap()
    }

    /// Get autocomplete suggestions at a given position
    #[wasm_bindgen]
    pub fn get_completions(&self, code: &str, line: usize, column: usize) -> JsValue {
        // Try to parse the code (best effort - collect symbols even if parse fails)
        let mut parser = Parser::new(code);
        let statements = match parser.parse_program() {
            Ok(stmts) => stmts,
            Err(_) => {
                // Even if parsing fails, we can still provide keywords and built-in functions
                // Use empty statements vector - CompletionProvider will still return keywords/built-ins
                Vec::new()
            }
        };

        let items = CompletionProvider::get_completions(code, line, column, &statements);
        
        // Convert to WASM-compatible format
        let wasm_items: Vec<CompletionItem> = items.into_iter()
            .map(|item| CompletionItem {
                label: item.label,
                kind: match item.kind {
                    CompletionItemKind::Keyword => "keyword".to_string(),
                    CompletionItemKind::Function => "function".to_string(),
                    CompletionItemKind::Variable => "variable".to_string(),
                    CompletionItemKind::Constant => "constant".to_string(),
                    CompletionItemKind::Type => "type".to_string(),
                },
                detail: item.detail,
                documentation: item.documentation,
                insert_text: item.insert_text,
            })
            .collect();

        serde_wasm_bindgen::to_value(&CompletionResult {
            items: wasm_items,
        }).unwrap()
    }

    /// Get hover information at a given position
    #[wasm_bindgen]
    pub fn get_hover(&self, code: &str, line: usize, column: usize) -> JsValue {
        // Try to parse the code (best effort)
        let mut parser = Parser::new(code);
        let statements = match parser.parse_program() {
            Ok(stmts) => stmts,
            Err(_) => {
                // Return empty hover if parse fails
                return serde_wasm_bindgen::to_value(&HoverInfo {
                    contents: String::new(),
                }).unwrap();
            }
        };

        if let Some(contents) = HoverProvider::get_hover_info(code, line, column, &statements) {
            serde_wasm_bindgen::to_value(&HoverInfo {
                contents,
            }).unwrap()
        } else {
            serde_wasm_bindgen::to_value(&HoverInfo {
                contents: String::new(),
            }).unwrap()
        }
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

// Helper function to extract all INPUT statements from AST
fn extract_input_statements(statements: &[crate::ast::Stmt], input_vars: &mut Vec<String>) {
    for stmt in statements {
        match stmt {
            crate::ast::Stmt::Input { name, .. } => {
                input_vars.push(name.clone());
            }
            crate::ast::Stmt::If { then_stmt, else_stmt, .. } => {
                extract_input_statements(then_stmt, input_vars);
                if let Some(else_stmts) = else_stmt {
                    extract_input_statements(else_stmts, input_vars);
                }
            }
            crate::ast::Stmt::While { body, .. } => {
                extract_input_statements(body, input_vars);
            }
            crate::ast::Stmt::For { body, .. } => {
                extract_input_statements(body, input_vars);
            }
            crate::ast::Stmt::RepeatUntil { body, .. } => {
                extract_input_statements(body, input_vars);
            }
            crate::ast::Stmt::Case { cases, otherwise, .. } => {
                for case in cases {
                    extract_input_statements(&case.body, input_vars);
                }
                if let Some(otherwise_body) = otherwise {
                    extract_input_statements(otherwise_body, input_vars);
                }
            }
            crate::ast::Stmt::FunctionDeclaration { function, .. } => {
                extract_input_statements(&function.body, input_vars);
            }
            crate::ast::Stmt::ProcedureDeclaration { procedure, .. } => {
                extract_input_statements(&procedure.body, input_vars);
            }
            _ => {}
        }
    }
}

