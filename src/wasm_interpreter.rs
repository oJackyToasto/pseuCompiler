use core::str;
use std::collections::HashMap;
use rand::Rng;

use crate::ast::{Expr, Function, Procedure, Stmt, Type, BinaryOp, BinaryOp::*, UnaryOp, UnaryOp::*, FileMode, TypeDeclarationVariant, Span};

#[derive(Debug, Clone)]
enum _ControlFlow {
    Return(Value),  // Return value from function
}

type _InterpreterResult<T> = Result<T, String>;

/// Error context for better error messages
#[derive(Debug, Clone)]
struct ErrorContext {
    _operation: String,
    call_stack: Vec<String>,
    context: Vec<String>,  // Current context (e.g., "in FOR loop", "in IF block")
    variables_in_scope: Vec<String>,
}

impl ErrorContext {
    fn new(operation: String) -> Self {
        Self {
            _operation: operation,
            call_stack: Vec::new(),
            context: Vec::new(),
            variables_in_scope: Vec::new(),
        }
    }

    fn format(&self, message: &str) -> String {
        let mut error = format!("error: {}\n", message);
        
        if !self.call_stack.is_empty() {
            error.push_str("  |\n");
            error.push_str("  | Call stack:\n");
            for (i, call) in self.call_stack.iter().enumerate() {
                if i == self.call_stack.len() - 1 {
                    error.push_str(&format!("  |   {}\n", call));
                } else {
                    error.push_str(&format!("  |   {}\n", call));
                }
            }
        }
        
        if !self.context.is_empty() {
            error.push_str("  |\n");
            error.push_str("  | Context:\n");
            for ctx in &self.context {
                error.push_str(&format!("  |   {}\n", ctx));
            }
        }
        
        if !self.variables_in_scope.is_empty() {
            error.push_str("  |\n");
            error.push_str(&format!("  | Available variables: {:?}\n", self.variables_in_scope));
        }
        
        error
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i32),
    Real(f64),
    String(String),
    Char(char),
    Boolean(bool),
    Date(String),
    Record {
        type_name: String,
        fields: HashMap<String, Value>,
    },
    Enum {
        type_name: String,
        value: String,
    },
    Pointer {
        points_to: Box<Type>,
        target: Box<Value>,
    },
    Set {
        element_type: Box<Type>,
        elements: Vec<Value>,
    },
    Array {
        element_type: Box<Type>,
        dimensions: Vec<usize>,
        start_indices: Vec<i32>,
        data: Vec<Value>,
    },
}

/// Virtual file handle for WASM - stores file content and position
#[derive(Debug, Clone)]
struct VirtualFileHandle {
    content: String,
    position: usize,
    mode: FileMode,
}

pub struct WasmInterpreter {
    variables: HashMap<String, Value>,
    variables_type: HashMap<String, Type>,
    functions: HashMap<String, Function>,
    procedures: HashMap<String, Procedure>,

    type_definitions: HashMap<String, Type>,
    open_files: HashMap<String, VirtualFileHandle>,  // Maps filename to virtual file handle
    virtual_files: HashMap<String, String>,  // Virtual file system: filename -> content
    
    // Traceback support
    call_stack: Vec<String>,  // Function/procedure call stack
    context_stack: Vec<String>,  // Statement context (FOR, WHILE, IF, etc.)
    
    // Output buffer to capture OUTPUT statements
    output_buffer: String,
    
    // Input queue for INPUT statements (future: callback system)
    input_queue: Vec<String>,
    
    // Constants - locked variables that cannot be reassigned
    constants: std::collections::HashSet<String>,
}

impl WasmInterpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            variables_type: HashMap::new(),
            functions: HashMap::new(),
            procedures: HashMap::new(),
            type_definitions: HashMap::new(),
            open_files: HashMap::new(),
            virtual_files: HashMap::new(),
            call_stack: Vec::new(),
            context_stack: Vec::new(),
            output_buffer: String::new(),
            input_queue: Vec::new(),
            constants: std::collections::HashSet::new(),
        }
    }
    
    /// Set a virtual file in the file system
    pub fn set_virtual_file(&mut self, filename: String, content: String) {
        self.virtual_files.insert(filename, content);
    }
    
    /// Get a virtual file from the file system
    pub fn get_virtual_file(&self, filename: &str) -> Option<&String> {
        self.virtual_files.get(filename)
    }
    
    /// Get the output buffer
    pub fn get_output(&self) -> &str {
        &self.output_buffer
    }
    
    /// Clear the output buffer
    pub fn clear_output(&mut self) {
        self.output_buffer.clear();
    }
    
    /// Clear the input queue
    pub fn clear_inputs(&mut self) {
        self.input_queue.clear();
    }
    
    /// Add input to the input queue
    pub fn add_input(&mut self, input: String) {
        self.input_queue.push(input);
    }

    /// Push a function/procedure call onto the call stack
    fn push_call(&mut self, name: &str, args: Option<&[Value]>) {
        let call_str = if let Some(args) = args {
            let arg_strs: Vec<String> = args.iter().map(|v| format!("{:?}", v)).collect();
            format!("{}({})", name, arg_strs.join(", "))
        } else {
            format!("{}()", name)
        };
        self.call_stack.push(call_str);
    }

    /// Pop a function/procedure call from the call stack
    fn pop_call(&mut self) {
        self.call_stack.pop();
    }

    /// Push a context (e.g., "in FOR loop", "in IF block")
    fn push_context(&mut self, context: String) {
        self.context_stack.push(context);
    }

    /// Pop a context
    fn pop_context(&mut self) {
        self.context_stack.pop();
    }

    /// Create an error with full context
    fn error_with_context(&self, message: &str, operation: &str) -> String {
        let mut ctx = ErrorContext::new(operation.to_string());
        ctx.call_stack = self.call_stack.clone();
        ctx.context = self.context_stack.clone();
        ctx.variables_in_scope = self.variables.keys().cloned().collect();
        ctx.format(message)
    }

    pub fn evaluate_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Declare { name, type_name, initial_value, span } => {
                match type_name {
                    Type::INTEGER | Type::REAL | Type::BOOLEAN | Type::CHAR | Type::STRING => {
                        let value = if let Some(expr) = initial_value {
                            self.evaluate_expr(expr)?
                        } else {
                            self.default_value(type_name)?
                        };
                        self.variables.insert(name.clone(), value);
                        self.variables_type.insert(name.clone(), type_name.clone());
                        Ok(())
                    }
                    Type::ARRAY { dimensions, element_type } => {
                        let mut dim_size = Vec::new();
                        let mut start_indices = Vec::new();
                        let mut total_size = 1;

                        for (start_expr, end_expr) in dimensions {
                            let start_val = self.evaluate_expr(start_expr)?;
                            let end_val = self.evaluate_expr(end_expr)?;

                            let start = match start_val {
                                Value::Integer(i) => i,
                                _ => {
                                    let msg = format!("Invalid start index type: {:?}", start_val);
                                    eprintln!("Error at line {}: {}", span.line, msg);
                                    return Err(msg);
                                }
                            };
                            let end = match end_val {
                                Value::Integer(i) => i,
                                _ => {
                                    let msg = format!("Invalid end index type: {:?}", end_val);
                                    eprintln!("Error at line {}: {}", span.line, msg);
                                    return Err(msg);
                                }
                            };

                            if start < 0 || end < start {
                                let msg = format!("Invalid array dimensions: start index must be >= 0 and end index must be >= start index");
                                eprintln!("Error at line {}: {}", span.line, msg);
                                return Err(msg);
                            }

                            let size = (end - start + 1) as usize;
                            dim_size.push(size);
                            start_indices.push(start);
                            total_size *= size;
                        }

                        let default_value = self.default_value(element_type)?;
                        let data = vec![default_value; total_size];

                        self.variables.insert(name.clone(), Value::Array {
                            element_type: element_type.clone(),
                            dimensions: dim_size,
                            start_indices: start_indices.clone(),
                            data,
                        });
                        self.variables_type.insert(name.clone(), Type::ARRAY { dimensions: dimensions.clone(), element_type: element_type.clone() });
                        Ok(())
                    }
                    Type::Custom(custom_name) => {
                        // Resolve the custom type and clone it to release the borrow
                        let resolved_type = self.type_definitions.get(custom_name)
                            .ok_or_else(|| format!("Type {} not found", custom_name))?
                            .clone();
                        let value = if let Some(expr) = initial_value {
                            self.evaluate_expr(expr)?
                        } else {
                            self.default_value(&resolved_type)?
                        };
                        self.variables.insert(name.clone(), value);
                        self.variables_type.insert(name.clone(), resolved_type);
                        Ok(())
                    }
                    Type::Record { .. } | Type::Enum { .. } | Type::Pointer { .. } | Type::Set { .. } => {
                        let value = if let Some(expr) = initial_value {
                            self.evaluate_expr(expr)?
                        } else {
                            self.default_value(type_name)?
                        };
                        self.variables.insert(name.clone(), value);
                        self.variables_type.insert(name.clone(), type_name.clone());
                        Ok(())
                    }
                    _ => {
                        let msg = format!("Unsupported type: {:?}", type_name);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            Stmt::Define { name, values, type_name, span } => {
                let type_def = self.type_definitions.get(type_name)
                    .ok_or_else(|| format!("Type {} not found", type_name))?;
                
                let value = match type_def {
                    Type::Set { element_type } => {
                        // Parse string values into Value types based on element_type
                        let mut set_elements = Vec::new();
                        for val_str in values {
                            let parsed_value = self.parse_value_string(&val_str, element_type)?;
                            set_elements.push(parsed_value);
                        }
                        Value::Set {
                            element_type: element_type.clone(),
                            elements: set_elements,
                        }
                    }
                    _ => {
                        let msg = format!("Define statement for type {} is not supported", type_name);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        return Err(msg);
                    }
                };
                
                self.variables.insert(name.clone(), value);
                self.variables_type.insert(name.clone(), type_def.clone());
                Ok(())
            }
            Stmt::Constant { name, value, span } => {
                let constant_value = if let Some(expr) = value {
                    // CONSTANT x <- expr (assign and lock)
                    self.evaluate_expr(expr)?
                } else {
                    // CONSTANT x (lock with current value)
                    self.variables.get(name)
                        .ok_or_else(|| {
                            let msg = format!("Constant '{}' cannot be locked: variable does not exist", name);
                            eprintln!("Error at line {}: {}", span.line, msg);
                            msg
                        })?
                        .clone()
                };
                
                // Store the constant value
                self.variables.insert(name.clone(), constant_value.clone());
                
                // Infer type from value if not already set
                if !self.variables_type.contains_key(name) {
                    let inferred_type = match constant_value {
                        Value::Integer(_) => Type::INTEGER,
                        Value::Real(_) => Type::REAL,
                        Value::Boolean(_) => Type::BOOLEAN,
                        Value::Char(_) => Type::CHAR,
                        Value::String(_) => Type::STRING,
                        Value::Array { element_type, .. } => Type::ARRAY {
                            dimensions: vec![],
                            element_type: element_type.clone(),
                        },
                        _ => {
                            let msg = format!("Cannot infer type for constant '{}'", name);
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return Err(msg);
                        }
                    };
                    self.variables_type.insert(name.clone(), inferred_type);
                }
                
                // Mark as constant (locked)
                self.constants.insert(name.clone());
                Ok(())
            }
            Stmt::Assign { name, indices, expression, span } => {
                // Check if trying to assign to a constant
                if self.constants.contains(name) {
                    let msg = format!("Cannot assign to constant '{}' - constants are locked", name);
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return Err(msg);
                }
                let value = self.evaluate_expr(expression)?;

                // Check if this is a field access assignment (obj.field)
                if let Some(dot_pos) = name.find('.') {
                    let (obj_name, field_name) = name.split_at(dot_pos);
                    let field_name = &field_name[1..]; // Skip the dot
                    
                    // Get the record
                    let record = self.variables.get_mut(obj_name)
                        .ok_or_else(|| format!("Variable '{}' not found", obj_name))?;
                    
                    match record {
                        Value::Record { fields, .. } => {
                            // Update the field
                            fields.insert(field_name.to_string(), value);
                            return Ok(());
                        }
                        _ => {
                            let msg = format!("Field access on non-record variable: {}", obj_name);
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return Err(msg);
                        }
                    }
                }
                
                // Check if this is a pointer dereference assignment (ptr^)
                if name.ends_with('^') {
                    let ptr_name = &name[..name.len() - 1];
                    
                    // Get the pointer variable
                    let ptr = self.variables.get_mut(ptr_name)
                        .ok_or_else(|| format!("Pointer variable '{}' not found", ptr_name))?;
                    
                    match ptr {
                        Value::Pointer { target, .. } => {
                            // Update the value the pointer points to
                            **target = value;
                            return Ok(());
                        }
                        _ => {
                            let msg = format!("Pointer dereference assignment on non-pointer variable: {}", ptr_name);
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return Err(msg);
                        }
                    }
                }
                

                if let Some(indices_exprs) = indices {
                    // Evaluate indices FIRST
                    let index_values : Vec<Value> = indices_exprs.iter()
                        .map(|expr| self.evaluate_expr(expr))
                        .collect::<Result<_, _>>()?;
                    
                    // Check if it's an array (sets are immutable, so no assignment)
                    let (dimensions, start_indices) = match self.variables.get(name) {
                        Some(Value::Array { dimensions, start_indices, .. }) => (dimensions.clone(), start_indices.clone()),
                        Some(Value::Set { .. }) => {
                            let msg = format!("Cannot assign to set '{}' - sets are immutable", name);
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return Err(msg);
                        }
                        Some(_) => return Err(format!("Variable '{}' is not an array", name)),
                        None => return Err(format!("Array {} not found", name)),
                    };
                    
                    if index_values.len() != start_indices.len() {
                        let msg = format!("Index dimension mismatch: expected {} dimensions, got {}", start_indices.len(), index_values.len());
                        eprintln!("Error at line {}: {}", span.line, msg);
                        return Err(msg);
                    }
                
                    let mut index_pos = Vec::new();
                    for (idx_val, start_idx) in index_values.iter().zip(start_indices.iter()) {
                        match idx_val { 
                            Value::Integer(i) => {
                                if *i < *start_idx {
                                    let msg = format!("Invalid index: must be >= {}, got {}", start_idx, i);
                                    eprintln!("Error at line {}: {}", span.line, msg);
                                    return Err(msg);
                                }
                                // Convert user index to 0-based internal index
                                index_pos.push((i - start_idx) as usize);
                            }
                            _ => {
                                let msg = format!("Invalid index type: {:?}", idx_val);
                                eprintln!("Error at line {}: {}", span.line, msg);
                                return Err(msg);
                            }
                        }
                    }
                    
                    // Calculate index (can use immutable borrow now)
                    let flat_idx = self.calculate_array_index(index_pos, &dimensions)?;
                    
                    // NOW get mutable reference and update
                    let array = self.variables.get_mut(name)
                        .ok_or_else(|| format!("Array {} not found", name))?;
                    
                    match array {
                        Value::Array { data, .. } => {
                            if flat_idx >= data.len() {
                                let msg = format!("Index out of bounds: {} for array {}", flat_idx, name);
                                eprintln!("Error at line {}: {}", span.line, msg);
                                return Err(msg);
                            }
                            data[flat_idx] = value;
                            return Ok(());
                        }
                        _ => {
                            let msg = format!("Invalid array type: {:?}", array);
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return Err(msg);
                        }
                    }
                } else {
                    // Simple variable assignment
                    self.variables.insert(name.clone(), value);
                    Ok(())
                }
            }
            Stmt::Output { exprs, span: _ } => {
                for expr in exprs {
                    let value = self.evaluate_expr(expr)?;
                    self.output_buffer.push_str(&self.value_to_string(&value));
                }
                self.output_buffer.push('\n');
                Ok(())
            }
            Stmt::Input { name, span } => {
                let var_type = self.variables_type.get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;

                // Get input from queue, or return error if empty
                let input = if let Some(input_val) = self.input_queue.pop() {
                    input_val
                } else {
                    return Err(format!("INPUT at line {}: No input available. Use add_input() to provide input values.", span.line));
                };

                let input = input.trim();
                
                // Don't echo input - terminal handles echo in interactive mode
                // (This allows for true terminal-like experience)

                let value = match var_type {
                    Type::INTEGER => {
                        Value::Integer(input.parse().map_err(|_| format!("Invalid integer: '{}'", input))?)
                    }
                    Type::REAL => {
                        Value::Real(input.parse().map_err(|_| format!("Invalid real number: '{}'", input))?)
                    }
                    Type::STRING => {
                        Value::String(input.to_string())
                    }
                    Type::CHAR => {
                        if input.len() == 1 {
                            Value::Char(input.chars().next().unwrap())
                        } else {
                            return Err(format!("Invalid char: expected single character, got '{}'", input));
                        }
                    }
                    Type::BOOLEAN => {
                        match input.to_lowercase().as_str() {
                            "true" | "1" | "yes" => Value::Boolean(true),
                            "false" | "0" | "no" => Value::Boolean(false),
                            _ => return Err(format!("Invalid boolean: '{}' (expected true/false)", input)),
                        }
                    }
                    _ => return Err(format!("Input not supported for type: {:?}", var_type)),
                };
                self.variables.insert(name.clone(), value);
                Ok(())
            }
            Stmt::If { condition, then_stmt, else_stmt, span: _ } => {
                let condition_value = self.evaluate_expr(condition)?;

                let is_true = match condition_value {
                    Value::Boolean(b) => b,
                    Value::Integer(i) => i != 0,
                    Value::Real(r) => r != 0.0,
                    Value::String(s) => !s.is_empty(),
                    _ => {
                        let msg = format!("Invalid condition type: {:?}", condition_value);
                        return Err(self.error_with_context(&msg, "IF condition evaluation"));
                    },
                };

                // Push context
                self.push_context(format!("in IF block (condition: {})", is_true));

                if is_true {
                    for stmt in then_stmt {
                        self.evaluate_stmt(stmt)?;
                    }
                } else if let Some(else_stmt) = else_stmt {
                    for stmt in else_stmt {
                        self.evaluate_stmt(stmt)?;
                    }
                }

                // Pop context
                self.pop_context();
                Ok(())
            }
            Stmt::While { condition, body, span: _ } => {
                // Push context
                self.push_context("in WHILE loop".to_string());
                
                let mut iteration = 0;
                loop {
                    iteration += 1;
                    let condition_value = self.evaluate_expr(condition)?;
                    let is_true = match condition_value {
                        Value::Boolean(b) => b,
                        Value::Integer(i) => i != 0,
                        Value::Real(r) => r != 0.0,
                        Value::String(s) => !s.is_empty(),
                        _ => {
                            let msg = format!("Invalid condition type: {:?}", condition_value);
                            self.pop_context();
                            return Err(self.error_with_context(&msg, "WHILE condition evaluation"));
                        },
                    };
                    
                    if !is_true {
                        break;
                    }
                    
                    // Update context with iteration
                    self.context_stack.pop();
                    self.push_context(format!("in WHILE loop (iteration {})", iteration));
                    
                    for stmt in body {
                        self.evaluate_stmt(stmt)?;
                    }
                }
                
                // Pop context
                self.pop_context();
                Ok(())
            }
            Stmt::For { counter, start, end, step, body, span: _ } => {
                // Evaluate start and end values
                let start_val = self.evaluate_expr(start)?;
                let end_val = self.evaluate_expr(end)?;
                
                // Get step value (default to 1 if not provided)
                let step_val = if let Some(step_expr) = step {
                    self.evaluate_expr(step_expr)?
                } else {
                    Value::Integer(1)  // Default step is 1
                };
                
                // Convert to integers (FOR loops typically use integers)
                let (start_int, end_int, step_int) = match (start_val, end_val, step_val) {
                    (Value::Integer(s), Value::Integer(e), Value::Integer(st)) => (s, e, st),
                    _ => {
                        let msg = format!("FOR loop requires integer values for start, end, and step");
                        return Err(self.error_with_context(&msg, "FOR loop initialization"));
                    }
                };
                
                // Validate step
                if step_int == 0 {
                    let msg = format!("FOR loop step cannot be zero");
                    return Err(self.error_with_context(&msg, "FOR loop initialization"));
                }
                
                // Push context
                self.push_context(format!("in FOR loop ({} = {} TO {})", counter, start_int, end_int));
                
                // Save the original value and type of counter if it exists (for scoping)
                let original_counter = self.variables.get(counter).cloned();
                let original_counter_type = self.variables_type.get(counter).cloned();
                
                // Automatically declare counter as INTEGER (always set type for FOR loop counter)
                self.variables_type.insert(counter.clone(), Type::INTEGER);
                
                // Initialize counter
                let mut current = start_int;
                self.variables.insert(counter.clone(), Value::Integer(current));

                // Execute loop
                loop {
                    // Check if we should continue based on step direction
                    let should_continue = if step_int > 0 {
                        current <= end_int
                    } else {
                        current >= end_int
                    };
                    
                    if !should_continue {
                        break;
                    }
                    
                    // Update context with current counter value
                    self.context_stack.pop();
                    self.push_context(format!("in FOR loop ({} = {})", counter, current));
                    
                    // Execute body
                    for stmt in body {
                        self.evaluate_stmt(stmt)?;
                    }
                    
                    // Increment counter
                    current += step_int;
                    self.variables.insert(counter.clone(), Value::Integer(current));
                }
                
                // Pop context
                self.pop_context();
                
                // Restore original counter value and type (if it existed) or remove it
                if let Some(orig) = original_counter {
                    self.variables.insert(counter.clone(), orig);
                    if let Some(orig_type) = original_counter_type {
                        self.variables_type.insert(counter.clone(), orig_type);
                    }
                } else {
                    self.variables.remove(counter);
                    self.variables_type.remove(counter);
                }
                
                Ok(())
            }
            Stmt::RepeatUntil { body, condition, span: _ } => {
                // Push context
                self.push_context("in REPEAT...UNTIL loop".to_string());
                
                let mut iteration = 0;
                loop {
                    iteration += 1;
                    
                    // Update context with iteration
                    self.context_stack.pop();
                    self.push_context(format!("in REPEAT...UNTIL loop (iteration {})", iteration));
                    
                    for stmt in body {
                        self.evaluate_stmt(stmt)?;
                    }
                    let condition_value = self.evaluate_expr(condition)?;
                    let is_true = match condition_value {
                        Value::Boolean(b) => b,
                        Value::Integer(i) => i != 0,
                        Value::Real(r) => r != 0.0,
                        Value::String(s) => !s.is_empty(),
                        _ => {
                            let msg = format!("Invalid condition type: {:?}", condition_value);
                            self.pop_context();
                            return Err(self.error_with_context(&msg, "REPEAT...UNTIL condition evaluation"));
                        },
                    };

                    if is_true {
                        break;
                    }
                }
                
                // Pop context
                self.pop_context();
                Ok(())
            }
            Stmt::Case { expression, cases, otherwise, span: _ } => {
                let expr_value = self.evaluate_expr(expression)?;

                let mut matched = false;
                for case in cases {
                    let case_value = self.evaluate_expr(&case.value)?;

                    if &expr_value == &case_value {
                        matched = true;
                        for stmt in case.body.clone() {
                            self.evaluate_stmt(&stmt)?;
                        }
                        break;
                    }
                }

                if !matched {
                    if let Some(ref otherwise_stmts) = otherwise {
                        for stmt in otherwise_stmts {
                            self.evaluate_stmt(stmt)?;
                        }
                    }
                }
                Ok(())
            }
            Stmt::FunctionDeclaration { function, span } => {
                let func_name = function.name.clone();

                if self.functions.contains_key(&func_name) {
                    let msg = format!("Function {} already declared", func_name);
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return Err(msg);
                }

                self.functions.insert(func_name, function.clone());
                Ok(())
            }
            Stmt::ProcedureDeclaration { procedure, span } => {
                let proc_name = procedure.name.clone();

                if self.procedures.contains_key(&proc_name) {
                    let msg = format!("Procedure {} already declared", proc_name);
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return Err(msg);
                }

                self.procedures.insert(proc_name, procedure.clone());
                Ok(())
            }
            Stmt::Call { name, args, span: _ } => {
                // Clone the procedure data we need before we need mutable access
                let procedure = self.procedures.get(name)
                    .ok_or_else(|| {
                        let msg = format!("Procedure {} not found", name);
                        self.error_with_context(&msg, "procedure call")
                    })?
                    .clone();  // Clone the entire procedure
            
                let arg_vals : Vec<Value> = if let Some(args_exprs) = args {
                    args_exprs.iter()
                        .map(|expr| self.evaluate_expr(expr))
                        .collect::<Result<_, _>>()
                        .map_err(|e| {
                            let msg = format!("Error evaluating procedure arguments: {}", e);
                            self.error_with_context(&msg, "evaluating procedure arguments")
                        })?
                } else {
                    Vec::new()
                };
            
                if arg_vals.len() != procedure.params.len() {
                    let msg = format!("Procedure {} expects {} arguments, got {}", name, procedure.params.len(), arg_vals.len());
                    return Err(self.error_with_context(&msg, "procedure call"));
                }
            
                // Push procedure call onto call stack
                self.push_call(name, Some(&arg_vals));
            
                let saved_vars = self.variables.clone();
                let saved_vars_type = self.variables_type.clone();
            
                for (param, arg_val) in procedure.params.iter().zip(arg_vals) {
                    self.variables.insert(param.name.clone(), arg_val.clone());
                    self.variables_type.insert(param.name.clone(), param.type_name.clone());
                }
            
                for stmt in &procedure.body {
                    self.evaluate_stmt(stmt)?;
                }
            
                self.variables = saved_vars;
                self.variables_type = saved_vars_type;
                
                // Pop procedure call from call stack
                self.pop_call();
                Ok(())
            }
            Stmt::Return { value: _value, span } => {
                // RETURN should only be used inside functions
                // This case handles RETURN in the main program (which is an error)
                let msg = "RETURN statement outside of function".to_string();
                eprintln!("Error at line {}: {}", span.line, msg);
                Err(msg)
            }

            Stmt::OpenFile { filename, mode, span } => {
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("Filename must be a string, got {:?}", filename_val);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        return Err(msg);
                    }
                };

                if self.open_files.contains_key(&filename_str) {
                    let msg = format!("File {} already open", filename_str);
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return Err(msg);
                }

                // Get file content from virtual file system, or create empty file
                let content = match mode {
                    FileMode::READ => {
                        self.virtual_files.get(&filename_str)
                            .ok_or_else(|| format!("File '{}' not found in virtual file system", filename_str))?
                            .clone()
                    }
                    FileMode::WRITE | FileMode::RANDOM => {
                        // Create new file or use existing
                        self.virtual_files.get(&filename_str)
                            .cloned()
                            .unwrap_or_else(String::new)
                    }
                };

                // Create virtual file handle
                let file_handle = VirtualFileHandle {
                    content,
                    position: 0,
                    mode: mode.clone(),
                };

                self.open_files.insert(filename_str, file_handle);
                
                Ok(())
            }
            Stmt::CloseFile { filename, span: _ } => {
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("CLOSEFILE expects STRING filename, got {:?}", filename_val);
                        return Err(msg);
                    }
                };
                
                // Save file content back to virtual file system if it was modified
                if let Some(file_handle) = self.open_files.remove(&filename_str) {
                    // Update virtual file system with current content
                    self.virtual_files.insert(filename_str, file_handle.content);
                    Ok(())
                } else {
                    Err(format!("File '{}' is not open", filename_str))
                }
            }
            Stmt::ReadFile { filename, name, span: _ } => {
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("READFILE expects STRING filename, got {:?}", filename_val);
                        return Err(msg);
                    }
                };
                
                // Get file handle
                let file_handle = self.open_files.get_mut(&filename_str)
                    .ok_or_else(|| format!("File '{}' is not open", filename_str))?;
                
                // Check mode
                match file_handle.mode {
                    FileMode::WRITE => {
                        return Err(format!("Cannot read from file '{}' opened in WRITE mode", filename_str));
                    }
                    _ => {}
                }
                
                // Read a line from the virtual file
                let content = &file_handle.content;
                let pos = file_handle.position;
                
                if pos >= content.len() {
                    // EOF - return empty string
                    let var_type = self.variables_type.get(name)
                        .ok_or_else(|| format!("Variable '{}' not found", name))?;
                    if !matches!(var_type, Type::STRING) {
                        return Err(format!("READFILE variable '{}' must be STRING type", name));
                    }
                    self.variables.insert(name.clone(), Value::String(String::new()));
                    return Ok(());
                }
                
                // Find next newline or end of file
                let remaining = &content[pos..];
                let line_end = remaining.find('\n')
                    .map(|i| pos + i + 1)
                    .unwrap_or(content.len());
                
                let line = content[pos..line_end].trim_end_matches('\r').trim_end_matches('\n').to_string();
                file_handle.position = line_end;
                
                // Store in variable
                let var_type = self.variables_type.get(name)
                    .ok_or_else(|| format!("Variable '{}' not found", name))?;
                
                // Ensure variable is STRING type
                if !matches!(var_type, Type::STRING) {
                    return Err(format!("READFILE variable '{}' must be STRING type", name));
                }
                
                self.variables.insert(name.clone(), Value::String(line));
                Ok(())
            }
            Stmt::WriteFile { filename, exprs, span: _ } => {
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("WRITEFILE expects STRING filename, got {:?}", filename_val);
                        return Err(msg);
                    }
                };
                
                // Evaluate all expressions and convert to strings FIRST (before borrowing file handle)
                let mut output = String::new();
                for expr in exprs {
                    let value = self.evaluate_expr(expr)?;
                    output.push_str(&self.value_to_string(&value));
                }
                
                // Get file handle AFTER evaluating expressions
                let file_handle = self.open_files.get_mut(&filename_str)
                    .ok_or_else(|| format!("File '{}' is not open", filename_str))?;
                
                // Check mode
                match file_handle.mode {
                    FileMode::READ => {
                        return Err(format!("Cannot write to file '{}' opened in READ mode", filename_str));
                    }
                    _ => {}
                }
                
                // Write to virtual file (append at current position)
                if file_handle.mode == FileMode::RANDOM {
                    // For random access, insert at position
                    let pos = file_handle.position.min(file_handle.content.len());
                    file_handle.content.insert_str(pos, &output);
                    file_handle.position += output.len();
                } else {
                    // For write mode, append to end
                    file_handle.content.push_str(&output);
                    file_handle.position = file_handle.content.len();
                }
                
                Ok(())
            }
            Stmt::Seek { filename, address, span: _ } => {
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("SEEK expects STRING filename, got {:?}", filename_val);
                        return Err(msg);
                    }
                };
                
                let address_val = self.evaluate_expr(address)?;
                let address_int = match address_val {
                    Value::Integer(i) => i,
                    _ => {
                        let msg = format!("SEEK expects INTEGER address, got {:?}", address_val);
                        return Err(msg);
                    }
                };
                
                // Get file handle (only RANDOM mode supports seek)
                let file_handle = self.open_files.get_mut(&filename_str)
                    .ok_or_else(|| format!("File '{}' is not open", filename_str))?;
                
                if file_handle.mode != FileMode::RANDOM {
                    return Err(format!("SEEK only works with files opened in RANDOM mode"));
                }
                
                // Set position (clamp to file size)
                let max_pos = file_handle.content.len();
                file_handle.position = (address_int as usize).min(max_pos);
                
                Ok(())
            }
            Stmt::GetRecord { filename, variable, span: _ } => {
                // GetRecord reads a fixed-length record (for binary/random access files)
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("GETRECORD expects STRING filename, got {:?}", filename_val);
                        return Err(msg);
                    }
                };
                
                let file_handle = self.open_files.get_mut(&filename_str)
                    .ok_or_else(|| format!("File '{}' is not open", filename_str))?;
                
                if file_handle.mode != FileMode::RANDOM {
                    return Err(format!("GETRECORD only works with files opened in RANDOM mode"));
                }
                
                // Read fixed-length record (256 bytes for simplicity)
                let record_size = 256;
                let content = &file_handle.content;
                let pos = file_handle.position;
                
                if pos >= content.len() {
                    return Err(format!("End of file reached in GETRECORD"));
                }
                
                let end_pos = (pos + record_size).min(content.len());
                let record = content[pos..end_pos].to_string();
                
                // Update position
                file_handle.position = end_pos;
                
                // Store in variable (simplified - assumes string representation)
                self.variables.insert(variable.clone(), Value::String(record));
                
                Ok(())
            }
            Stmt::PutRecord { filename, variable, span: _ } => {
                // PutRecord writes a fixed-length record (for binary/random access files)
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("PUTRECORD expects STRING filename, got {:?}", filename_val);
                        return Err(msg);
                    }
                };
                
                // Get variable value to write
                let var_value = self.variables.get(variable)
                    .ok_or_else(|| format!("Variable '{}' not found", variable))?;
                
                // Convert variable to string representation
                let record_data = self.value_to_string(var_value);
                
                // Get file handle
                let file_handle = self.open_files.get_mut(&filename_str)
                    .ok_or_else(|| format!("File '{}' is not open", filename_str))?;
                
                if file_handle.mode != FileMode::RANDOM {
                    return Err(format!("PUTRECORD only works with files opened in RANDOM mode"));
                }
                
                // Write fixed-length record (pad or truncate to fixed size)
                // For simplicity, we'll use a fixed size of 256 bytes
                let record_size = 256;
                let mut record = record_data.as_bytes().to_vec();
                record.truncate(record_size);
                record.resize(record_size, 0); // Pad with zeros
                
                let record_str = String::from_utf8_lossy(&record).to_string();
                let pos = file_handle.position;
                let content = &mut file_handle.content;
                
                // Insert or replace at position
                if pos >= content.len() {
                    // Append
                    content.push_str(&record_str);
                } else {
                    // Replace existing content
                    let end_pos = (pos + record_size).min(content.len());
                    content.replace_range(pos..end_pos, &record_str);
                }
                
                file_handle.position += record_size;
                
                Ok(())
            }

            Stmt::TypeDeclaration { name, variant, span: _ } => {
                let type_def = match variant {
                    TypeDeclarationVariant::Record { fields } => {
                        Type::Record {
                            name: name.clone(),
                            fields: fields.clone(),
                        }
                    }
                    TypeDeclarationVariant::Enum { values } => {
                        Type::Enum {
                            name: name.clone(),
                            values: values.clone(),
                        }
                    }
                    TypeDeclarationVariant::Pointer { points_to } => {
                        Type::Pointer {
                            points_to: points_to.clone(),
                        }
                    }
                    TypeDeclarationVariant::Set { element_type } => {
                        Type::Set {
                            element_type: element_type.clone(),
                        }
                    }
                };
                
                self.type_definitions.insert(name.clone(), type_def);
                Ok(())
            }
        }
    }

    fn parse_value_string(&self, val_str: &str, element_type: &Type) -> Result<Value, String> {
        match element_type {
            Type::INTEGER => {
                val_str.parse::<i32>()
                    .map(Value::Integer)
                    .map_err(|_| format!("Invalid integer: {}", val_str))
            }
            Type::REAL => {
                val_str.parse::<f64>()
                    .map(Value::Real)
                    .map_err(|_| format!("Invalid real: {}", val_str))
            }
            Type::STRING => {
                Ok(Value::String(val_str.to_string()))
            }
            Type::CHAR => {
                // Remove quotes if present ('A' -> A)
                let ch = val_str.trim_matches('\'').chars().next()
                    .ok_or_else(|| format!("Invalid char: {}", val_str))?;
                Ok(Value::Char(ch))
            }
            Type::BOOLEAN => {
                match val_str.to_uppercase().as_str() {
                    "TRUE" => Ok(Value::Boolean(true)),
                    "FALSE" => Ok(Value::Boolean(false)),
                    _ => Err(format!("Invalid boolean: {}", val_str))
                }
            }
            _ => {
                Err(format!("Unsupported element type for set: {:?}", element_type))
            }
        }
    }

    fn calculate_array_index(&self, indices: Vec<usize>, dimensions: &[usize]) -> Result<usize, String> {
        if indices.len() != dimensions.len() {
            return Err(format!(
                "Index dimension mismatch: expected {} dimensions, got {}",
                dimensions.len(),
                indices.len()
            ));
        }
        
        // Check bounds
        for (i, (idx, dim_size)) in indices.iter().zip(dimensions.iter()).enumerate() {
            if *idx >= *dim_size {
                return Err(format!(
                    "Index {} out of bounds: {} >= {}",
                    i, idx, dim_size
                ));
            }
        }
        
        // Calculate flat index using row-major order
        // For [i, j, k] with dimensions [d1, d2, d3]:
        // flat_index = i * (d2 * d3) + j * d3 + k
        let mut flat_index = 0;
        let mut stride = 1;
        
        for (idx, dim_size) in indices.iter().zip(dimensions.iter()).rev() {
            flat_index += idx * stride;
            stride *= dim_size;
        }
        
        Ok(flat_index)
    }

    fn default_value(&self, type_name: &Type) -> Result<Value, String> {
        match type_name {
            Type::INTEGER => Ok(Value::Integer(0)),
            Type::REAL => Ok(Value::Real(0.0)),
            Type::BOOLEAN => Ok(Value::Boolean(false)),
            Type::CHAR => Ok(Value::Char('\0')),
            Type::STRING => Ok(Value::String("".to_string())),
            Type::DATE => Ok(Value::Date("".to_string())),
            
            Type::Custom(name) => {
                let resolved_type = self.type_definitions.get(name)
                    .ok_or_else(|| format!("Type {} not found", name))?;
                self.default_value(resolved_type)
            }
            
            Type::Record { name, fields } => {
                let mut field_values = HashMap::new();
                for field in fields {
                    field_values.insert(field.name.clone(), self.default_value(&field.type_name)?);
                }
                Ok(Value::Record {
                    type_name: name.clone(),
                    fields: field_values,
                })
            }

            Type::Enum { name, values } => {
                if values.is_empty() {
                    let msg = format!("Enum type {} has no values", name);
                    eprintln!("Error: {}", msg);
                    return Err(msg);
                }
                Ok(Value::Enum {
                    type_name: name.clone(),
                    value: values[0].clone(),
                })
            }

            Type::Pointer { points_to } => {
                let target_value = self.default_value(points_to)?;
                Ok(Value::Pointer {
                    points_to: points_to.clone(),
                    target: Box::new(target_value),
                })
            }

            Type::Set { element_type } => {
                Ok(Value::Set {
                    element_type: element_type.clone(),
                    elements: Vec::new(),
                })
            }
            
            _ => {
                let msg = format!("Unsupported type: {:?}", type_name);
                eprintln!("Error: {}", msg);
                Err(msg)
            }
        }
    }

    fn format_array_with_dimensions(&self, data: &[Value], dimensions: &[usize], dim_index: usize) -> String {
        if dimensions.is_empty() || data.is_empty() {
            return "[]".to_string();
        }
        
        let current_dim = dimensions[dim_index];
        let remaining_dims = &dimensions[dim_index + 1..];
        
        // Calculate how many elements per sub-array at this dimension
        let elements_per_sub = if remaining_dims.is_empty() {
            1
        } else {
            remaining_dims.iter().product::<usize>()
        };
        
        let mut result = String::new();
        result.push('[');
        
        for i in 0..current_dim {
            let start_idx = i * elements_per_sub;
            let end_idx = (i + 1) * elements_per_sub;
            
            if start_idx >= data.len() {
                break;
            }
            
            let slice = &data[start_idx..end_idx.min(data.len())];
            
            if remaining_dims.is_empty() {
                // Last dimension - just format the values
                if !slice.is_empty() {
                    result.push_str(&self.value_to_string(&slice[0]));
                    for val in slice.iter().skip(1) {
                        result.push_str(", ");
                        result.push_str(&self.value_to_string(val));
                    }
                }
            } else {
                // Recursively format sub-arrays
                result.push_str(&self.format_array_with_dimensions(slice, dimensions, dim_index + 1));
            }
            
            if i < current_dim - 1 {
                result.push_str(", ");
            }
        }
        
        result.push(']');
        result
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Integer(i) => i.to_string(),
            Value::Real(r) => r.to_string(),
            Value::String(s) => s.clone(),
            Value::Char(c) => c.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Date(d) => d.clone(),
            Value::Record { .. } => format!("{:?}", value), // For now, use debug format for complex types
            Value::Enum { value, .. } => value.clone(),
            Value::Pointer { .. } => format!("{:?}", value),
            Value::Set { .. } => format!("{:?}", value),
            Value::Array { dimensions, data, .. } => {
                self.format_array_with_dimensions(data, dimensions, 0)
            },
        }
    }

    pub fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(num, _) => {
                if num.contains('.') {
                    Ok(Value::Real(num.parse().map_err(|_| "Invalid real number")?))
                } else {
                    Ok(Value::Integer(num.parse().map_err(|_| "Invalid integer number")?))
                }
            }
            Expr::String(str, _) => Ok(Value::String(str.clone())),
            Expr::Char(ch, _) => {
                let c = ch.chars().nth(0).unwrap_or('\0');
                Ok(Value::Char(c))
            }
            Expr::Boolean(bool, _) => {
                match bool {
                    true => Ok(Value::Boolean(true)),
                    false => Ok(Value::Boolean(false)),
                }
            },
            Expr::Variable(var, _) => {
                self.variables.get(var)
                    .cloned()
                    .ok_or_else(|| {
                        let msg = format!("Variable '{}' not found", var);
                        self.error_with_context(&msg, "variable access")
                    })
            }
            Expr::BinaryOp(left, op, right, span) => {
                let left_val = self.evaluate_expr(left)?;
                let right_val = self.evaluate_expr(right)?;
                self.evaluate_binary_op(op.clone(), &left_val, &right_val, span.clone())
            }
            Expr::UnaryOp(op, expr, span) => {
                self.evaluate_unary_op(op.clone(), expr, span.clone())
            }
            Expr::FunctionCall { name, args, span } => {
                self.evaluate_function_call(name, &Some(args.clone()), span.clone())
            }
            Expr::ArrayAccess { array, indices, span } => {
                // Evaluate indices first (before borrowing array)
                let index_vals : Vec<Value> = indices.iter()
                    .map(|idx| self.evaluate_expr(idx))
                    .collect::<Result<_, _>>()?;
            
                let array_val = self.variables.get(array)
                    .ok_or_else(|| {
                        let msg = format!("Variable '{}' not found", array);
                        self.error_with_context(&msg, "array access")
                    })?;
            
                match array_val {
                    Value::Array { dimensions, start_indices, data, .. } => {
                        if index_vals.len() != start_indices.len() {
                            let msg = format!("Index dimension mismatch: expected {} dimensions, got {}", start_indices.len(), index_vals.len());
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return Err(msg);
                        }
                        
                        let mut index_positions = Vec::new();
                        for (idx_val, start_idx) in index_vals.iter().zip(start_indices.iter()) {
                            match idx_val {
                                Value::Integer(i) => {
                                    if *i < *start_idx {
                                        let msg = format!("Index must be >= {}, got {}", start_idx, i);
                                        return Err(self.error_with_context(&msg, "array index validation"));
                                    }
                                    // Convert user index to 0-based internal index
                                    index_positions.push((i - start_idx) as usize);
                                }
                                _ => {
                                    let msg = format!("Index must be integer, got {:?}", idx_val);
                                    eprintln!("Error at line {}: {}", span.line, msg);
                                    return Err(msg);
                                }
                            }
                        }
                        
                        let flat_index = self.calculate_array_index(index_positions, dimensions)?;
                        if flat_index >= data.len() {
                            let msg = format!("Array index out of bounds: {}", flat_index);
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return Err(msg);
                        }
                        Ok(data[flat_index].clone())
                    }
                    Value::Set { elements, .. } => {
                        // Sets use 1-based indexing (no start index stored)
                        if index_vals.len() != 1 {
                            let msg = format!("Set access requires exactly 1 index, got {}", index_vals.len());
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return Err(msg);
                        }
                        let index = match &index_vals[0] {
                            Value::Integer(i) => {
                                if *i < 1 {
                                    let msg = format!("Set index must be >= 1, got {}", i);
                                    eprintln!("Error at line {}: {}", span.line, msg);
                                    return Err(msg);
                                }
                                (i - 1) as usize  // Convert 1-based to 0-based
                            }
                            _ => {
                                let msg = format!("Set index must be integer, got {:?}", index_vals[0]);
                                eprintln!("Error at line {}: {}", span.line, msg);
                                return Err(msg);
                            }
                        };
                        if index >= elements.len() {
                            let msg = format!("Set index out of bounds: {}", index);
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return Err(msg);
                        }
                        Ok(elements[index].clone())
                    }
                    Value::Enum { .. } => {
                        // Enums don't support indexed access - they're single values
                        let msg = format!("Cannot use indexed access on enum value: {}", array);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                    _ => {
                        let msg = format!("Indexed access on unsupported type: {}", array);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            Expr::FieldAccess { object, field, span } => {
                let object_val = self.evaluate_expr(object)?;
                match object_val {
                    Value::Record { type_name, fields } => {
                        fields.get(field)
                            .cloned()
                            .ok_or_else(|| format!("Field '{}' not found in record of type '{}'", field, type_name))
                    }
                    _ => {
                        let msg = format!("Field access on non-record value: {:?}", object_val);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            Expr::PointerRef { target, span } => {
                // Match on the expression to extract variable name
                match target.as_ref() {
                    Expr::Variable(var_name, _) => {
                        // Get the variable's type
                        let var_type = self.variables_type.get(var_name)
                            .ok_or_else(|| format!("Variable '{}' not found for pointer reference", var_name))?;
                        
                        // Get the variable's value
                        let var_value = self.variables.get(var_name)
                            .ok_or_else(|| format!("Variable '{}' not found", var_name))?;
                        
                        Ok(Value::Pointer {
                            points_to: Box::new(var_type.clone()),
                            target: Box::new(var_value.clone()),
                        })
                    }
                    _ => {
                        let msg = format!("Pointer reference (^) can only be applied to variables, got {:?}", target);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            // Expr::SetAccess { set, element } => {
            //     // Treat element as an index (like array access)
            //     let set_val = self.evaluate_expr(set)?;
            //     let index_val = self.evaluate_expr(element)?;
                
            //     let index = match index_val {
            //         Value::Integer(i) => {
            //             if i < 1 {
            //                 let msg = format!("Set index must be >= 1, got {}", i);
            //                 // Error:("{}", msg);
            //                 return Err(msg);
            //             }
            //             (i - 1) as usize  // Convert 1-based to 0-based
            //         }
            //         _ => {
            //             let msg = format!("Set index must be integer, got {:?}", index_val);
            //             // Error:("{}", msg);
            //             return Err(msg);
            //         }
            //     };
                
            //     match set_val {
            //         Value::Set { elements, .. } => {
            //             if index >= elements.len() {
            //                 let msg = format!("Set index out of bounds: {}", index);
            //                 // Error:("{}", msg);
            //                 return Err(msg);
            //             }
            //             Ok(elements[index].clone())
            //         }
            //         _ => {
            //             let msg = format!("Set access on non-set value: {:?}", set_val);
            //             // Error:("{}", msg);
            //             Err(msg)
            //         }
            //     }
            // }
            // Expr::EnumAccess { enum_type, value } => {
            //     // Get the enum type definition
            //     let type_def = self.type_definitions.get(enum_type)
            //         .ok_or_else(|| format!("Enum type {} not found", enum_type))?;
                
            //     match type_def {
            //         Type::Enum { values, .. } => {
            //             // Check if value is a valid enum value name
            //             if values.contains(value) {
            //                 Ok(Value::Enum {
            //                     type_name: enum_type.clone(),
            //                     value: value.clone(),
            //                 })
            //             } else {
            //                 // Try to interpret as index
            //                 if let Ok(idx) = value.parse::<usize>() {
            //                     if idx >= 1 && idx <= values.len() {
            //                         Ok(Value::Enum {
            //                             type_name: enum_type.clone(),
            //                             value: values[idx - 1].clone(),  // 1-based to 0-based
            //                         })
            //                     } else {
            //                         let msg = format!("Enum index out of bounds: {}", idx);
            //                         // Error:(msg, span.line);
            //                         Err(msg)
            //                     }
            //                 } else {
            //                     let msg = format!("Invalid enum value '{}' for type '{}'", value, enum_type);
            //                     // Error:(msg, span.line);
            //                     Err(msg)
            //                 }
            //             }
            //         }
            //         _ => {
            //             let msg = format!("EnumAccess on non-enum type: {}", enum_type);
            //             // Error:("{}", msg);
            //             Err(msg)
            //         }
            //     }
            // }
            Expr::PointerDeref { pointer, span } => {
                // var^ dereferences the pointer
                let ptr_val = self.evaluate_expr(pointer)?;
                match ptr_val {
                    Value::Pointer { target, .. } => {
                        Ok(*target)  // Return the value the pointer points to
                    }
                    _ => {
                        let msg = format!("Pointer dereference (^) can only be applied to pointer values, got {:?}", ptr_val);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
        }
    }

    fn evaluate_function_call(&mut self, name: &str, args: &Option<Vec<Expr>>, span: Span) -> Result<Value, String> {
        // Try built-in functions first
        if let Some(result) = self.evaluate_builtin_function(name, args, span) {
            return Ok(result);
        }
        
        // Try user-defined functions
        let function = self.functions.get(name)
            .ok_or_else(|| {
                let msg = format!("Function '{}' not found", name);
                self.error_with_context(&msg, "function call")
            })?
            .clone();  // Clone to avoid borrow issues
        
        // Evaluate arguments
        let arg_values: Vec<Value> = if let Some(arg_exprs) = args {
            arg_exprs.iter()
                .map(|expr| self.evaluate_expr(expr))
                .collect::<Result<_, _>>()
                .map_err(|e| {
                    let msg = format!("Error evaluating function arguments: {}", e);
                    self.error_with_context(&msg, "evaluating function arguments")
                })?
        } else {
            Vec::new()
        };
        
        // Validate argument count
        if arg_values.len() != function.params.len() {
            let msg = format!(
                "Function '{}' expects {} arguments, got {}",
                name, function.params.len(), arg_values.len()
            );
            return Err(self.error_with_context(&msg, "function call"));
        }
        
        // Push function call onto call stack
        self.push_call(name, Some(&arg_values));
        
        // Save current variable state (for scoping)
        let saved_variables = self.variables.clone();
        let saved_variable_types = self.variables_type.clone();
        
        // Bind parameters to argument values
        for (param, arg_value) in function.params.iter().zip(arg_values.iter()) {
            self.variables.insert(param.name.clone(), arg_value.clone());
            self.variables_type.insert(param.name.clone(), param.type_name.clone());
        }
        
        // Execute function body
        let mut return_value: Option<Value> = None;
        for stmt in &function.body {
            // Check if this is a RETURN statement
            if let Stmt::Return { value, span: _ } = stmt {
                // Evaluate return expression if provided
                return_value = Some(if let Some(expr) = value {
                    self.evaluate_expr(expr)?
                } else {
                    // Default return value based on return type
                    self.default_value(&function.return_type)?
                });
                break; // Exit function
            } else {
                // Execute other statements normally
                self.evaluate_stmt(stmt)?;
            }
        }
        
        // Restore variable state
        self.variables = saved_variables;
        self.variables_type = saved_variable_types;
        
        // Pop function call from call stack
        self.pop_call();
        
        // Return the value (or default if no RETURN statement)
        Ok(return_value.unwrap_or_else(|| {
            // If no RETURN statement, return default value for return type
            self.default_value(&function.return_type).unwrap_or(Value::Integer(0))
        }))
    }

    fn evaluate_builtin_function(&mut self, name: &str, args: &Option<Vec<Expr>>, span: Span) -> Option<Value> {
        match name {
            "MOD" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 2 {
                    let msg = format!("MOD requires 2 arguments, got {}", args_vec.len());
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return None;
                }
                let arg1 = self.evaluate_expr(&args_vec[0]).ok()?;
                let arg2 = self.evaluate_expr(&args_vec[1]).ok()?;
                match (&arg1, &arg2) {
                    (Value::Integer(l), Value::Integer(r)) => {
                        if *r == 0 {
                            let msg = format!("Modulo by zero");
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return None;
                        }
                        Some(Value::Integer(l % r))
                    }
                    _ => {
                        let msg = format!("MOD requires integer arguments, got {:?} and {:?}", arg1, arg2);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        None
                    }
                }
            }
            "DIV" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 2 {
                    let msg = format!("DIV expects 2 arguments, got {}", args_vec.len());
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return None;
                }
                let arg1 = self.evaluate_expr(&args_vec[0]).ok()?;
                let arg2 = self.evaluate_expr(&args_vec[1]).ok()?;
                match (&arg1, &arg2) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        if *y == 0 {
                            let msg = format!("Division by zero in DIV");
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return None;
                        }
                        Some(Value::Integer(x / y))
                    }
                    _ => {
                        let msg = format!("DIV requires integer arguments, got {:?} and {:?}", arg1, arg2);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        None
                    }
                }
            }
            "LENGTH" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    let msg = format!("LENGTH expects 1 argument, got {}", args_vec.len());
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return None;
                }
                let str_val = self.evaluate_expr(&args_vec[0]).ok()?;
                match str_val {
                    Value::String(s) => Some(Value::Integer(s.len() as i32)),
                    _ => {
                        let msg = format!("LENGTH requires string argument, got {:?}", str_val);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        None
                    }
                }
            }
            "UCASE" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    let msg = format!("UCASE expects 1 argument, got {}", args_vec.len());
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return None;
                }
                let str_val = self.evaluate_expr(&args_vec[0]).ok()?;
                match str_val {
                    Value::String(s) => Some(Value::String(s.to_uppercase())),
                    Value::Char(c) => Some(Value::String(c.to_uppercase().to_string())),
                    _ => {
                        let msg = format!("UCASE requires string or char argument, got {:?}", str_val);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        None
                    }
                }
            }
            "LCASE" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    let msg = format!("LCASE expects 1 argument, got {}", args_vec.len());
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return None;
                }
                let str_val = self.evaluate_expr(&args_vec[0]).ok()?;
                match str_val {
                    Value::String(s) => Some(Value::String(s.to_lowercase())),
                    Value::Char(c) => Some(Value::String(c.to_lowercase().to_string())),
                    _ => {
                        let msg = format!("LCASE requires string or char argument, got {:?}", str_val);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        None
                    }
                }
            }
            "SUBSTRING" | "MID" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 3 {
                    let msg = format!("{} expects 3 arguments (string, start, length), got {}", name, args_vec.len());
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return None;
                }
                let str_val = self.evaluate_expr(&args_vec[0]).ok()?;
                let start_val = self.evaluate_expr(&args_vec[1]).ok()?;
                let length_val = self.evaluate_expr(&args_vec[2]).ok()?;
                
                match (&str_val, &start_val, &length_val) {
                    (Value::String(s), Value::Integer(start), Value::Integer(length)) => {
                        // 1-based indexing: convert to 0-based
                        let start_idx = (start - 1) as usize;
                        let end_idx = (start_idx + *length as usize).min(s.len());
                        if start_idx >= s.len() {
                            Some(Value::String(String::new()))
                        } else {
                            Some(Value::String(s[start_idx..end_idx].to_string()))
                        }
                    }
                    _ => {
                        let msg = format!("{} expects (STRING, INTEGER, INTEGER) arguments, got {:?}, {:?}, {:?}", name, str_val, start_val, length_val);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        None
                    }
                }
            }
            "RIGHT" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 2 {
                    let msg = format!("RIGHT expects 2 arguments, got {}", args_vec.len());
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return None;
                }
                let str_val = self.evaluate_expr(&args_vec[0]).ok()?;
                let length_val = self.evaluate_expr(&args_vec[1]).ok()?;
                match (&str_val, &length_val) {
                    (Value::String(s), Value::Integer(length)) => {
                        if *length < 0 {
                            let msg = format!("RIGHT requires non-negative length, got {}", length);
                            eprintln!("Error at line {}: {}", span.line, msg);
                            return None;
                        }
                        // Handle case where length > string length
                        let length = (*length as usize).min(s.len());
                        let start_idx = s.len().saturating_sub(length);
                        Some(Value::String(s[start_idx..].to_string()))
                    }
                    _ => {
                        let msg = format!("RIGHT expects (STRING, INTEGER) arguments, got {:?}, {:?}", str_val, length_val);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        None
                    }
                }
            }
            "RANDOM" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 0 {
                    let msg = format!("RANDOM expects 0 argument, got {}", args_vec.len());
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return None;
                }
                Some(Value::Real(rand::thread_rng().gen_range(0.0..=1.0)))
            }
            "RAND" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    let msg = format!("RAND expects 1 argument, got {}", args_vec.len());
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return None;
                }
                let max_val = self.evaluate_expr(&args_vec[0]).ok()?;
                match &max_val {
                    Value::Integer(max) => Some(Value::Real(rand::thread_rng().gen_range(0.0..=*max as f64))),
                    _ => {
                        let msg = format!("RAND requires integer argument, got {:?}", max_val);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        None
                    }
                }   
            }
            "ROUND" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 2 {
                    let msg = format!("ROUND expects 2 arguments, got {}", args_vec.len());
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return None;
                }
                let val = self.evaluate_expr(&args_vec[0]).ok()?;
                let precision = self.evaluate_expr(&args_vec[1]).ok()?;
                match (&val, &precision) {
                    (Value::Real(r), Value::Integer(p)) => {
                        // Round to p decimal places
                        let multiplier = 10_f64.powi(*p as i32);
                        Some(Value::Real((r * multiplier).round() / multiplier))
                    }
                    (Value::Real(r), _) => {
                        // If precision is not integer, just round to nearest integer
                        Some(Value::Integer(r.round() as i32))
                    }
                    (Value::Integer(i), _) => {
                        // If already integer, return as-is
                        Some(Value::Integer(*i))
                    }
                    _ => {
                        let msg = format!("ROUND requires numeric argument, got {:?}", val);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        None
                    }
                }
            }
            "INT" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    let msg = format!("INT expects 1 argument, got {}", args_vec.len());
                    eprintln!("Error at line {}: {}", span.line, msg);
                    return None;
                }
                let val = self.evaluate_expr(&args_vec[0]).ok()?;
                match &val {
                    Value::Real(r) => Some(Value::Integer(r.floor() as i32)),
                    Value::Integer(i) => Some(Value::Integer(*i)),
                    _ => {
                        let msg = format!("INT requires numeric argument, got {:?}", val);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        None
                    }
                }
            }
            "EOF" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    return None;
                }
                let filename_val = self.evaluate_expr(&args_vec[0]).ok()?;
                match filename_val {
                    Value::String(filename) => {
                        // Check if file is open
                        if let Some(file_handle) = self.open_files.get(&filename) {
                            // Check if we're at EOF (position >= content length)
                            match file_handle.mode {
                                FileMode::WRITE => {
                                    // Write mode - always false (can't be at EOF for writing)
                                    Some(Value::Boolean(false))
                                }
                                _ => {
                                    // Read or Random mode - check position
                                    Some(Value::Boolean(file_handle.position >= file_handle.content.len()))
                                }
                            }
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }
            _ => None,
        }
    }

    fn evaluate_unary_op(&mut self, op: UnaryOp, expr: &Expr, span: Span) -> Result<Value, String> {
        match op {
            Negate => {
                let val = self.evaluate_expr(expr)?;
                match val {
                    Value::Integer(l) => Ok(Value::Integer(-l)),
                    Value::Real(l) => Ok(Value::Real(-l)),
                    _ => {
                        let msg = format!("Unsupported negation operation: {:?}", op);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            Not => {
                let val = self.evaluate_expr(expr)?;
                match val {
                    Value::Boolean(l) => Ok(Value::Boolean(!l)),
                    _ => {
                        let msg = format!("Unsupported NOT operation: {:?}", op);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
        }
    }

    fn evaluate_binary_op(&self, op: BinaryOp, left: &Value, right: &Value, span: Span) -> Result<Value, String> {
        match op {
            Add => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
                    (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l + r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
                    (Value::String(l), Value::Integer(r)) => Ok(Value::String(format!("{}{}", l, r.to_string()))),
                    (Value::Integer(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l.to_string(), r))),
                    (Value::String(l), Value::Real(r)) => Ok(Value::String(format!("{}{}", l, r.to_string()))),
                    (Value::Real(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l.to_string(), r))),
                    (Value::Char(l), Value::Char(r)) => Ok(Value::String(format!("{}{}", l, r))),
                    (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l + *r as f64)),
                    (Value::Integer(l), Value::Real(r)) => Ok(Value::Real(*l as f64 + r)),
                    _ => {
                        let msg = format!("Unsupported addition operation: {:?} with {:?} and {:?}", op, left, right);
                        Err(msg)
                    }
                }
            }
            Subtract => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
                    (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l - r)),
                    (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l - *r as f64)),
                    (Value::Integer(l), Value::Real(r)) => Ok(Value::Real(*l as f64 - r)),
                    _ => {
                        let msg = format!("Unsupported subtraction operation: {:?} with {:?} and {:?}", op, left, right);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            Multiply => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
                    (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l * r)),
                    (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l * *r as f64)),
                    (Value::Integer(l), Value::Real(r)) => Ok(Value::Real(*l as f64 * r)),
                    _ => {
                        let msg = format!("Unsupported multiplication operation: {:?} with {:?} and {:?}", op, left, right);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            Divide => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if *b == 0 {
                            return Err("Division by zero".to_string());
                        }
                        Ok(Value::Real(*a as f64 / *b as f64))
                    }
                    (Value::Real(a), Value::Real(b)) => {
                        if *b == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        Ok(Value::Real(a / b))
                    }
                    (Value::Integer(a), Value::Real(b)) => {
                        if *b == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        Ok(Value::Real(*a as f64 / b))
                    }
                    (Value::Real(a), Value::Integer(b)) => {
                        if *b == 0 {
                            return Err("Division by zero".to_string());
                        }
                        Ok(Value::Real(a / *b as f64))
                    }
                    _ => Err("Invalid operands for division".to_string()),
                }
            }
            _Div => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if *b == 0 {
                            return Err("Division by zero in DIV".to_string());
                        }
                        Ok(Value::Integer(a / b))
                    }
                    _ => Err("DIV requires integer operands".to_string()),
                }
            }
            Modulus => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if *b == 0 {
                            return Err("Modulo by zero".to_string());
                        }
                        Ok(Value::Integer(a % b))
                    }
                    _ => Err("Modulus requires integer operands".to_string()),
                }
            }

            Equals => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l == r)),
                    (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l == r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l == r)),
                    (Value::Char(l), Value::Char(r)) => Ok(Value::Boolean(l == r)),
                    (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(*l == (*r as f64))),
                    (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((*l as f64) == *r)),
                    _ => {
                        let msg = format!("Unsupported equality operation: {:?} with {:?} and {:?}", op, left, right);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            NotEquals => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l != r)),
                    (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l != r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l != r)),
                    (Value::Char(l), Value::Char(r)) => Ok(Value::Boolean(l != r)),
                    (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(*l != (*r as f64))),
                    (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((*l as f64) != *r)),
                    _ => {
                        let msg = format!("Unsupported not equals operation: {:?} with {:?} and {:?}", op, left, right);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            LessThan => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
                    (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l < r)),
                    (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(*l < (*r as f64))),
                    (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((*l as f64) < *r)),
                    _ => {
                        let msg = format!("Unsupported less than operation: {:?} with {:?} and {:?}", op, left, right);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            GreaterThan => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
                    (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l > r)),
                    (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(*l > (*r as f64))),
                    (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((*l as f64) > *r)),
                    _ => {
                        let msg = format!("Unsupported greater than operation: {:?} with {:?} and {:?}", op, left, right);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            LessThanOrEqual => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
                    (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l <= r)),
                    (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(*l <= (*r as f64))),
                    (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((*l as f64) <= *r)),
                    _ => {
                        let msg = format!("Unsupported less than or equal operation: {:?} with {:?} and {:?}", op, left, right);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            GreaterThanOrEqual => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
                    (Value::Real(l), Value::Real(r)) => Ok(Value::Boolean(l >= r)),
                    (Value::Real(l), Value::Integer(r)) => Ok(Value::Boolean(*l >= (*r as f64))),
                    (Value::Integer(l), Value::Real(r)) => Ok(Value::Boolean((*l as f64) >= *r)),
                    _ => {
                        let msg = format!("Unsupported greater than or equal operation: {:?} with {:?} and {:?}", op, left, right);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            And => {
                match (left, right) {
                    (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l && *r)),
                    _ => {
                        let msg = format!("Unsupported AND operation: {:?} with {:?} and {:?}", op, left, right);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
            Or => {
                match (left, right) {
                    (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l || *r)),
                    _ => {
                        let msg = format!("Unsupported OR operation: {:?} with {:?} and {:?}", op, left, right);
                        eprintln!("Error at line {}: {}", span.line, msg);
                        Err(msg)
                    }
                }
            }
        }
    }
}