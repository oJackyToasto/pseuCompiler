use crate::ast::{Stmt, Type, Span};

#[derive(Debug, Clone)]
pub struct VariableSymbol {
    pub name: String,
    pub type_name: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConstantSymbol {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    pub name: String,
    pub params: Vec<ParamInfo>,
    pub return_type: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ProcedureSymbol {
    pub name: String,
    pub params: Vec<ParamInfo>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeSymbol {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub type_name: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub variables: Vec<VariableSymbol>,
    pub constants: Vec<ConstantSymbol>,
    pub functions: Vec<FunctionSymbol>,
    pub procedures: Vec<ProcedureSymbol>,
    pub types: Vec<TypeSymbol>,
}

#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub after_declare: bool,
    pub after_function: bool,
    pub in_array_decl: bool,
    pub in_assignment: bool,
    pub after_if: bool,
    pub after_for: bool,
    pub in_function_call: bool,
    pub is_start_of_line: bool,
    pub prefix: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompletionItemKind {
    Keyword,
    Function,
    Variable,
    Constant,
    Type,
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: String,
}

pub struct SymbolExtractor;

impl SymbolExtractor {
    pub fn extract_symbols(statements: &[Stmt]) -> SymbolTable {
        let mut table = SymbolTable {
            variables: Vec::new(),
            constants: Vec::new(),
            functions: Vec::new(),
            procedures: Vec::new(),
            types: Vec::new(),
        };

        for stmt in statements {
            Self::extract_from_stmt(stmt, &mut table);
        }

        table
    }

    fn extract_from_stmt(stmt: &Stmt, table: &mut SymbolTable) {
        match stmt {
            Stmt::Declare { name, type_name, span, .. } => {
                table.variables.push(VariableSymbol {
                    name: name.clone(),
                    type_name: Some(type_name.clone()),
                    span: span.clone(),
                });
            }
            Stmt::Constant { name, span, .. } => {
                table.constants.push(ConstantSymbol {
                    name: name.clone(),
                    span: span.clone(),
                });
            }
            Stmt::FunctionDeclaration { function, span } => {
                let params: Vec<ParamInfo> = function.params.iter()
                    .map(|p| ParamInfo {
                        name: p.name.clone(),
                        type_name: Some(p.type_name.clone()),
                    })
                    .collect();

                table.functions.push(FunctionSymbol {
                    name: function.name.clone(),
                    params,
                    return_type: Some(function.return_type.clone()),
                    span: span.clone(),
                });

                for body_stmt in &function.body {
                    Self::extract_from_stmt(body_stmt, table);
                }
            }
            Stmt::ProcedureDeclaration { procedure, span } => {
                let params: Vec<ParamInfo> = procedure.params.iter()
                    .map(|p| ParamInfo {
                        name: p.name.clone(),
                        type_name: Some(p.type_name.clone()),
                    })
                    .collect();

                table.procedures.push(ProcedureSymbol {
                    name: procedure.name.clone(),
                    params,
                    span: span.clone(),
                });

                for body_stmt in &procedure.body {
                    Self::extract_from_stmt(body_stmt, table);
                }
            }
            Stmt::TypeDeclaration { name, span, .. } => {
                table.types.push(TypeSymbol {
                    name: name.clone(),
                    span: span.clone(),
                });
            }
            _ => {
            }
        }
    }
}

pub struct ContextAnalyzer;

impl ContextAnalyzer {
    pub fn analyze_context(code: &str, line: usize, column: usize) -> CompletionContext {
        let lines: Vec<&str> = code.split('\n').collect();
        let current_line = if line > 0 && line <= lines.len() {
            lines[line - 1]
        } else {
            ""
        };
        
        let before_cursor = if column > 0 && column <= current_line.len() + 1 {
            &current_line[..(column - 1).min(current_line.len())]
        } else {
            current_line
        };
        
        let after_cursor = if column > 0 && column <= current_line.len() + 1 {
            &current_line[(column - 1).min(current_line.len())..]
        } else {
            ""
        };

        let previous_lines: String = if line > 1 {
            lines[0..(line - 1)].join("\n")
        } else {
            String::new()
        };
        
        let full_context = format!("{}\n{}", previous_lines, before_cursor);

        let prefix = Self::extract_prefix(before_cursor);

        let after_declare = Self::matches_pattern(before_cursor, r"DECLARE\s+[\w,]*\s*:\s*$");
        let after_function = Self::matches_pattern(before_cursor, r"FUNCTION\s+\w+\s*\([^)]*\)\s*RETURNS\s*$");
        let in_array_decl = Self::matches_pattern(&full_context, r"ARRAY\s*\[")
            && Self::matches_pattern(before_cursor, r":\s*ARRAY");
        let in_assignment = before_cursor.contains("<-") && !after_cursor.trim_start().starts_with('<');
        let after_if = Self::matches_pattern(before_cursor, r"IF\s+.+\s+(THEN|DO)\s*$");
        let after_for = Self::matches_pattern(before_cursor, r"FOR\s+\w+\s*<-\s*.+\s+TO\s*$");
        let in_function_call = Self::matches_pattern(after_cursor, r"^\s*[A-Z_][A-Z0-9_]*\s*\(")
            || Self::matches_pattern(before_cursor, r"[A-Z_][A-Z0-9_]*\s*\($");
        let is_start_of_line = before_cursor.trim().is_empty();

        CompletionContext {
            after_declare,
            after_function,
            in_array_decl,
            in_assignment,
            after_if,
            after_for,
            in_function_call,
            is_start_of_line,
            prefix,
        }
    }

    fn extract_prefix(text: &str) -> String {
        text.trim()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .last()
            .unwrap_or("")
            .to_string()
    }

    fn matches_pattern(text: &str, pattern: &str) -> bool {
        if pattern == r"DECLARE\s+[\w,]*\s*:\s*$" {
            return text.trim_end().ends_with(':') && text.contains("DECLARE");
        }
        if pattern == r"FUNCTION\s+\w+\s*\([^)]*\)\s*RETURNS\s*$" {
            return text.contains("FUNCTION") && text.contains("RETURNS") && text.trim_end().ends_with("RETURNS");
        }
        if pattern == r"ARRAY\s*\[" {
            return text.contains("ARRAY") && text.contains('[');
        }
        if pattern == r":\s*ARRAY" {
            return text.trim_end().ends_with("ARRAY") && text.contains(':');
        }
        if pattern == r"IF\s+.+\s+(THEN|DO)\s*$" {
            return text.contains("IF") && (text.trim_end().ends_with("THEN") || text.trim_end().ends_with("DO"));
        }
        if pattern == r"FOR\s+\w+\s*<-\s*.+\s+TO\s*$" {
            return text.contains("FOR") && text.contains("<-") && text.contains("TO") && text.trim_end().ends_with("TO");
        }
        if pattern == r"^\s*[A-Z_][A-Z0-9_]*\s*\(" {
            return text.trim_start().chars().next().map(|c| c.is_uppercase() || c == '_').unwrap_or(false)
                && text.contains('(');
        }
        if pattern == r"[A-Z_][A-Z0-9_]*\s*\($" {
            return text.trim_end().ends_with('(') && text.len() > 1;
        }
        false
    }
}

pub const KEYWORDS: &[&str] = &[
    "DECLARE", "CONSTANT", "FUNCTION", "PROCEDURE", "ENDFUNCTION", "ENDPROCEDURE",
    "IF", "THEN", "ELSE", "ENDIF", "WHILE", "DO", "ENDWHILE",
    "FOR", "TO", "NEXT", "REPEAT", "UNTIL",
    "RETURN", "CALL", "INPUT", "OUTPUT",
    "OPENFILE", "CLOSEFILE", "READFILE", "WRITEFILE", "SEEK",
    "GETRECORD", "PUTRECORD",
    "INTEGER", "REAL", "STRING", "CHAR", "BOOLEAN", "ARRAY", "OF",
    "AND", "OR", "NOT", "TRUE", "FALSE",
    "TYPE", "ENDTYPE", "CASE", "ENDCASE", "OTHERWISE",
    "RETURNS"
];

pub const TYPES: &[&str] = &[
    "INTEGER", "REAL", "STRING", "CHAR", "BOOLEAN", "ARRAY"
];

#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: &'static str,
    pub description: &'static str,
    pub params: &'static [&'static str],
}

pub const BUILTIN_FUNCTIONS: &[BuiltinFunction] = &[
    BuiltinFunction { name: "LENGTH", description: "Returns the length of a string", params: &["string"] },
    BuiltinFunction { name: "UCASE", description: "Converts a string to uppercase", params: &["string"] },
    BuiltinFunction { name: "LCASE", description: "Converts a string to lowercase", params: &["string"] },
    BuiltinFunction { name: "SUBSTRING", description: "Extracts a substring from a string", params: &["string", "start", "length"] },
    BuiltinFunction { name: "RIGHT", description: "Returns the rightmost characters of a string", params: &["string", "count"] },
    BuiltinFunction { name: "MID", description: "Extracts characters from the middle of a string", params: &["string", "start", "count"] },
    BuiltinFunction { name: "ROUND", description: "Rounds a number to the nearest integer", params: &["number", "decimals"] },
    BuiltinFunction { name: "RANDOM", description: "Returns a random number between 0 and 1", params: &[] },
    BuiltinFunction { name: "RAND", description: "Returns a random real number in the range 0 to x (not inclusive of x)", params: &["x"] },
    BuiltinFunction { name: "EOF", description: "Checks if end of file has been reached", params: &["file"] },
    BuiltinFunction { name: "MOD", description: "Returns the remainder of division", params: &["dividend", "divisor"] },
];

pub struct CompletionProvider;

impl CompletionProvider {
    pub fn get_completions(
        code: &str,
        line: usize,
        column: usize,
        statements: &[Stmt],
    ) -> Vec<CompletionItem> {
        let context = ContextAnalyzer::analyze_context(code, line, column);
        let symbols = SymbolExtractor::extract_symbols(statements);
        let mut suggestions = Vec::new();

        let prefix_lower = context.prefix.to_lowercase();
        let matches_prefix = |item: &str| -> bool {
            if context.prefix.is_empty() {
                true
            } else {
                item.to_lowercase().starts_with(&prefix_lower)
            }
        };

        // Always include keywords and built-in functions (available everywhere)
        for &keyword in KEYWORDS {
            if matches_prefix(keyword) {
                // Special handling for CASE keyword - should insert "CASE OF "
                let insert_text = if keyword == "CASE" {
                    "CASE OF ".to_string()
                } else {
                    keyword.to_string()
                };
                
                suggestions.push(CompletionItem {
                    label: keyword.to_string(),
                    kind: CompletionItemKind::Keyword,
                    detail: Some("Keyword".to_string()),
                    documentation: Some(Self::get_keyword_documentation(keyword)),
                    insert_text,
                });
            }
        }

        // Always include built-in functions
        for func in BUILTIN_FUNCTIONS {
            if matches_prefix(func.name) {
                suggestions.push(CompletionItem {
                    label: func.name.to_string(),
                    kind: CompletionItemKind::Function,
                    detail: Some("Built-in Function".to_string()),
                    documentation: Some(func.description.to_string()),
                    insert_text: format!("{}(", func.name),
                });
            }
        }

        // Always include variables (without scope filtering for now)
        for variable in &symbols.variables {
            if matches_prefix(&variable.name) {
                let detail = if let Some(ref type_name) = variable.type_name {
                    format!("Variable: {:?}", type_name)
                } else {
                    "Variable".to_string()
                };
                suggestions.push(CompletionItem {
                    label: variable.name.clone(),
                    kind: CompletionItemKind::Variable,
                    detail: Some(detail),
                    documentation: Some(format!("Variable: {}", variable.name)),
                    insert_text: variable.name.clone(),
                });
            }
        }

        // Always include constants (without scope filtering for now)
        for constant in &symbols.constants {
            if matches_prefix(&constant.name) {
                suggestions.push(CompletionItem {
                    label: constant.name.clone(),
                    kind: CompletionItemKind::Constant,
                    detail: Some("Constant".to_string()),
                    documentation: Some(format!("Constant: {}", constant.name)),
                    insert_text: constant.name.clone(),
                });
            }
        }

        if context.after_declare {
            for &type_name in TYPES {
                if matches_prefix(type_name) {
                    suggestions.push(CompletionItem {
                        label: type_name.to_string(),
                        kind: CompletionItemKind::Type,
                        detail: Some("Type".to_string()),
                        documentation: Some(format!("Data type: {}", type_name)),
                        insert_text: type_name.to_string(),
                    });
                }
            }
            if matches_prefix("ARRAY") {
                suggestions.push(CompletionItem {
                    label: "ARRAY".to_string(),
                    kind: CompletionItemKind::Keyword,
                    detail: Some("Type".to_string()),
                    documentation: Some("Array type declaration".to_string()),
                    insert_text: "ARRAY".to_string(),
                });
            }
        }
        else if context.after_function {
            for &type_name in TYPES {
                if matches_prefix(type_name) {
                    suggestions.push(CompletionItem {
                        label: type_name.to_string(),
                        kind: CompletionItemKind::Type,
                        detail: Some("Return Type".to_string()),
                        documentation: Some(format!("Return type: {}", type_name)),
                        insert_text: type_name.to_string(),
                    });
                }
            }
        }
        // Start of line or after keyword - suggest functions and procedures
        // (keywords and built-ins already added above)
        else if context.is_start_of_line || Self::is_after_delimiter(&context.prefix, code, line, column) {
            // Suggest functions and procedures
            for func in &symbols.functions {
                if matches_prefix(&func.name) {
                    let detail = if let Some(ref ret_type) = func.return_type {
                        format!("Function: {:?}", ret_type)
                    } else {
                        "Function".to_string()
                    };
                    suggestions.push(CompletionItem {
                        label: func.name.clone(),
                        kind: CompletionItemKind::Function,
                        detail: Some(detail),
                        documentation: Some(Self::format_function_documentation(func)),
                        insert_text: format!("{}(", func.name),
                    });
                }
            }

            for proc in &symbols.procedures {
                if matches_prefix(&proc.name) {
                    suggestions.push(CompletionItem {
                        label: proc.name.clone(),
                        kind: CompletionItemKind::Function,
                        detail: Some("Procedure".to_string()),
                        documentation: Some(Self::format_procedure_documentation(proc)),
                        insert_text: format!("{}(", proc.name),
                    });
                }
            }
        }
        // In assignment or expression - suggest user-defined functions and procedures
        // (keywords, built-ins, variables, and constants already added above)
        else {


            // Suggest user-defined functions and procedures
            for func in &symbols.functions {
                if matches_prefix(&func.name) {
                    let detail = if let Some(ref ret_type) = func.return_type {
                        format!("Function: {:?}", ret_type)
                    } else {
                        "Function".to_string()
                    };
                    suggestions.push(CompletionItem {
                        label: func.name.clone(),
                        kind: CompletionItemKind::Function,
                        detail: Some(detail),
                        documentation: Some(Self::format_function_documentation(func)),
                        insert_text: format!("{}(", func.name),
                    });
                }
            }

            for proc in &symbols.procedures {
                if matches_prefix(&proc.name) {
                    suggestions.push(CompletionItem {
                        label: proc.name.clone(),
                        kind: CompletionItemKind::Function,
                        detail: Some("Procedure".to_string()),
                        documentation: Some(Self::format_procedure_documentation(proc)),
                        insert_text: format!("{}(", proc.name),
                    });
                }
            }

        }

        // Sort suggestions
        suggestions.sort_by(|a, b| a.label.cmp(&b.label));
        suggestions
    }

    fn is_after_delimiter(prefix: &str, _code: &str, _line: usize, _column: usize) -> bool {
        prefix.is_empty()
    }

    fn get_keyword_documentation(keyword: &str) -> String {
        match keyword {
            "DECLARE" => "Declares a variable or array".to_string(),
            "CONSTANT" => "Declares a constant value".to_string(),
            "FUNCTION" => "Defines a function".to_string(),
            "PROCEDURE" => "Defines a procedure".to_string(),
            "IF" => "Conditional statement".to_string(),
            "WHILE" => "While loop".to_string(),
            "FOR" => "For loop".to_string(),
            "RETURN" => "Returns a value from a function".to_string(),
            "OUTPUT" => "Outputs a value".to_string(),
            "INPUT" => "Reads input from user".to_string(),
            "ARRAY" => "Array type declaration".to_string(),
            "CASE" => "CASE OF <identifier> - Switch statement".to_string(),
            _ => format!("Keyword: {}", keyword),
        }
    }

    fn format_function_documentation(func: &FunctionSymbol) -> String {
        let params: String = func.params.iter()
            .map(|p| {
                if let Some(ref t) = p.type_name {
                    format!("{}: {:?}", p.name, t)
                } else {
                    p.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        
        let return_info = if let Some(ref ret_type) = func.return_type {
            format!(" → {:?}", ret_type)
        } else {
            String::new()
        };
        
        format!("Function {}({}){}", func.name, params, return_info)
    }

    fn format_procedure_documentation(proc: &ProcedureSymbol) -> String {
        let params: String = proc.params.iter()
            .map(|p| {
                if let Some(ref t) = p.type_name {
                    format!("{}: {:?}", p.name, t)
                } else {
                    p.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        
        format!("Procedure {}({})", proc.name, params)
    }
}

pub struct HoverProvider;
impl HoverProvider {
    pub fn get_hover_info(
        code: &str,
        line: usize,
        column: usize,
        statements: &[Stmt],
    ) -> Option<String> {
        let symbols = SymbolExtractor::extract_symbols(statements);
        
        let lines: Vec<&str> = code.split('\n').collect();
        let current_line = if line > 0 && line <= lines.len() {
            lines[line - 1]
        } else {
            return None;
        };
        
        let before_cursor = if column > 0 && column <= current_line.len() + 1 {
            &current_line[..(column - 1).min(current_line.len())]
        } else {
            current_line
        };

        let word = ContextAnalyzer::extract_prefix(before_cursor);
        if word.is_empty() {
            return None;
        }

        if KEYWORDS.contains(&word.as_str()) {
            return Some(format!("**{}**\n\n{}", word, CompletionProvider::get_keyword_documentation(&word)));
        }

        if let Some(func) = BUILTIN_FUNCTIONS.iter().find(|f| f.name.eq_ignore_ascii_case(&word)) {
            let params = if func.params.is_empty() {
                "no parameters".to_string()
            } else {
                func.params.join(", ")
            };
            return Some(format!("**{}({})**\n\n{}", func.name, params, func.description));
        }

        if let Some(variable) = symbols.variables.iter().find(|v| v.name == word) {
            let type_info = if let Some(ref type_name) = variable.type_name {
                format!(": {:?}", type_name)
            } else {
                String::new()
            };
            return Some(format!("**Variable:** `{}{}`", variable.name, type_info));
        }

        if let Some(constant) = symbols.constants.iter().find(|c| c.name == word) {
            return Some(format!("**Constant:** `{}`", constant.name));
        }

        if let Some(func) = symbols.functions.iter().find(|f| f.name == word) {
            return Some(format!("**Function:** {}", CompletionProvider::format_function_documentation(func)));
        }

        if let Some(proc) = symbols.procedures.iter().find(|p| p.name == word) {
            return Some(format!("**Procedure:** {}", CompletionProvider::format_procedure_documentation(proc)));
        }

        if let Some(type_sym) = symbols.types.iter().find(|t| t.name == word) {
            return Some(format!("**Type:** `{}`", type_sym.name));
        }

        None
    }
}

