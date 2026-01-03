use core::str;
use std::collections::HashMap;
use log::*;
use rand::Rng;

use crate::{ast::{Expr, Function, Procedure, Stmt, Stmt::*, Type, BinaryOp, BinaryOp::*, UnaryOp, UnaryOp::*, FileMode}, log_error};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write, Seek, SeekFrom, BufRead};

#[derive(Debug, Clone)]
enum ControlFlow {
    Return(Value),  // Return value from function
}

type InterpreterResult<T> = Result<T, String>;

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
        data: Vec<Value>,
    },
}

#[derive(Debug)]
enum FileHandle {
    Read(BufReader<File>),
    Write(BufWriter<File>),
    Random(File),  // For RANDOM mode - can both read and write
}

pub struct Interpreter {
    variables: HashMap<String, Value>,
    variables_type: HashMap<String, Type>,
    functions: HashMap<String, Function>,
    procedures: HashMap<String, Procedure>,

    type_definitions: HashMap<String, Type>,
    open_files: HashMap<String, FileHandle>,  // Maps filename to file handle
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            variables_type: HashMap::new(),
            functions: HashMap::new(),
            procedures: HashMap::new(),
            type_definitions: HashMap::new(),
            open_files: HashMap::new(),
        }
    }

    pub fn evaluate_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Declare { name, type_name, initial_value } => {
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
                        let mut total_size = 1;

                        for (start_expr, end_expr) in dimensions {
                            let start_val = self.evaluate_expr(start_expr)?;
                            let end_val = self.evaluate_expr(end_expr)?;

                            let start = match start_val {
                                Value::Integer(i) => i,
                                _ => {
                                    let msg = format!("Invalid start index type: {:?}", start_val);
                                    log_error!("{}", msg);
                                    return Err(msg);
                                }
                            };
                            let end = match end_val {
                                Value::Integer(i) => i,
                                _ => {
                                    let msg = format!("Invalid end index type: {:?}", end_val);
                                    log_error!("{}", msg);
                                    return Err(msg);
                                }
                            };

                            if start < 1 || end < start {
                                let msg = format!("Invalid array dimensions: start index must be >= 1 and end index must be >= start index");
                                log_error!("{}", msg);
                                return Err(msg);
                            }

                            let size = (end - start + 1) as usize;
                            dim_size.push(size);
                            total_size *= size;
                        }

                        let default_value = self.default_value(element_type)?;
                        let data = vec![default_value; total_size];

                        self.variables.insert(name.clone(), Value::Array {
                            element_type: element_type.clone(),
                            dimensions: dim_size,
                            data,
                        });
                        self.variables_type.insert(name.clone(), Type::ARRAY { dimensions: dimensions.clone(), element_type: element_type.clone() });
                        Ok(())
                    }
                    _ => {
                        let msg = format!("Unsupported type: {:?}", type_name);
                        log_error!("{}", msg);
                        Err(msg)
                    }
                }
            }
            Stmt::Assign { name, indices, expression } => {
                let value = self.evaluate_expr(expression)?;
                if let Some(indices_exprs) = indices {
                    // Evaluate indices FIRST
                    let index_values : Vec<Value> = indices_exprs.iter()
                        .map(|expr| self.evaluate_expr(expr))
                        .collect::<Result<_, _>>()?;
            
                    let mut index_pos = Vec::new();
                    for idx_val in index_values {
                        match idx_val { 
                            Value::Integer(i) => {
                                if i < 1 {
                                    let msg = format!("Invalid index: {}", i);
                                    log_error!("{}", msg);
                                    return Err(msg);
                                }
                                index_pos.push((i - 1) as usize);
                            }
                            _ => {
                                let msg = format!("Invalid index type: {:?}", idx_val);
                                log_error!("{}", msg);
                                return Err(msg);
                            }
                        }
                    }
                    
                    // Get dimensions FIRST (immutable borrow)
                    let dimensions = match self.variables.get(name) {
                        Some(Value::Array { dimensions, .. }) => dimensions.clone(),
                        Some(_) => return Err(format!("Variable '{}' is not an array", name)),
                        None => return Err(format!("Array {} not found", name)),
                    };
                    
                    // Calculate index (can use immutable borrow now)
                    let flat_idx = self.calculate_array_index(index_pos, &dimensions)?;
                    
                    // NOW get mutable reference and update
                    let array = self.variables.get_mut(name)
                        .ok_or_else(|| format!("Array {} not found", name))?;
                    
                    match array {
                        Value::Array { data, .. } => {
                            if flat_idx >= data.len() {
                                let msg = format!("Index out of bounds: {} for array {}", flat_idx, name);
                                log_error!("{}", msg);
                                return Err(msg);
                            }
                            data[flat_idx] = value;
                            return Ok(());
                        }
                        _ => {
                            let msg = format!("Invalid array type: {:?}", array);
                            log_error!("{}", msg);
                            return Err(msg);
                        }
                    }
                } else {
                    // Simple variable assignment
                    self.variables.insert(name.clone(), value);
                    Ok(())
                }
            }
            Stmt::Output { exprs } => {
                for expr in exprs {
                    let value = self.evaluate_expr(expr)?;
                    print!("{}", self.value_to_string(&value));
                }
                println!();
                Ok(())
            }
            Stmt::Input { name } => {
                let var_type = self.variables_type.get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;

                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .map_err(|_| "Failed to read input")?;

                let input = input.trim();

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
            Stmt::If { condition, then_stmt, else_stmt } => {
                let condition_value = self.evaluate_expr(condition)?;

                let is_true = match condition_value {
                    Value::Boolean(b) => b,
                    Value::Integer(i) => i != 0,
                    Value::Real(r) => r != 0.0,
                    Value::String(s) => !s.is_empty(),
                    _ => {
                        let msg = format!("Invalid condition type: {:?}", condition_value);
                        log_error!("{}", msg);
                        return Err(msg);
                    },
                };

                if is_true {
                    for stmt in then_stmt {
                        self.evaluate_stmt(stmt)?;
                    }
                } else if let Some(else_stmt) = else_stmt {
                    for stmt in else_stmt {
                        self.evaluate_stmt(stmt)?;
                    }
                }

                Ok(())
            }
            Stmt::While { condition, body } => {
                loop {
                    let condition_value = self.evaluate_expr(condition)?;
                    let is_true = match condition_value {
                        Value::Boolean(b) => b,
                        Value::Integer(i) => i != 0,
                        Value::Real(r) => r != 0.0,
                        Value::String(s) => !s.is_empty(),
                        _ => {
                            let msg = format!("Invalid condition type: {:?}", condition_value);
                            log_error!("{}", msg);
                            return Err(msg);
                        },
                    };
                    
                    if !is_true {
                        break;
                    }
                    
                    for stmt in body {
                        self.evaluate_stmt(stmt)?;
                    }
                }
                Ok(())
            }
            Stmt::For { counter, start, end, step, body } => {
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
                        log_error!("{}", msg);
                        return Err(msg);
                    }
                };
                
                // Validate step
                if step_int == 0 {
                    let msg = format!("FOR loop step cannot be zero");
                    log_error!("{}", msg);
                    return Err(msg);
                }
                
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
                    
                    // Execute body
                    for stmt in body {
                        self.evaluate_stmt(stmt)?;
                    }
                    
                    // Increment counter
                    current += step_int;
                    self.variables.insert(counter.clone(), Value::Integer(current));
                }
                
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
            Stmt::RepeatUntil { body, condition } => {
                loop {
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
                            log_error!("{}", msg);
                            return Err(msg);
                        },
                    };

                    if is_true {
                        break;
                    }
                }
                Ok(())
            }
            Stmt::Case { expression, cases, otherwise } => {
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
            Stmt::FunctionDeclaration { function } => {
                let func_name = function.name.clone();

                if self.functions.contains_key(&func_name) {
                    let msg = format!("Function {} already declared", func_name);
                    log_error!("{}", msg);
                    return Err(msg);
                }

                self.functions.insert(func_name, function.clone());
                Ok(())
            }
            Stmt::ProcedureDeclaration { procedure } => {
                let proc_name = procedure.name.clone();

                if self.procedures.contains_key(&proc_name) {
                    let msg = format!("Procedure {} already declared", proc_name);
                    log_error!("{}", msg);
                    return Err(msg);
                }

                self.procedures.insert(proc_name, procedure.clone());
                Ok(())
            }
            Stmt::Call { name, args } => {
                // Clone the procedure data we need before we need mutable access
                let procedure = self.procedures.get(name)
                    .ok_or_else(|| format!("Procedure {} not found", name))?
                    .clone();  // Clone the entire procedure
            
                let arg_vals : Vec<Value> = if let Some(args_exprs) = args {
                    args_exprs.iter()
                        .map(|expr| self.evaluate_expr(expr))
                        .collect::<Result<_, _>>()?
                } else {
                    Vec::new()
                };
            
                if arg_vals.len() != procedure.params.len() {
                    let msg = format!("Procedure {} expects {} arguments, got {}", name, procedure.params.len(), arg_vals.len());
                    log_error!("{}", msg);
                    return Err(msg);
                }
            
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
                Ok(())
            }
            Stmt::Return { value } => {
                // RETURN should only be used inside functions
                // This case handles RETURN in the main program (which is an error)
                let msg = "RETURN statement outside of function".to_string();
                log_error!("{}", msg);
                Err(msg)
            }

            Stmt::OpenFile { filename, mode } => {
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("Filename must be a string, got {:?}", filename_val);
                        log_error!("{}", msg);
                        return Err(msg);
                    }
                };

                if self.open_files.contains_key(&filename_str) {
                    let msg = format!("File {} already open", filename_str);
                    log_error!("{}", msg);
                    return Err(msg);
                }

                let file: File = match mode {
                    FileMode::READ => {
                        OpenOptions::new().read(true).open(&filename_str).map_err(|_| format!("Failed to open file {} for reading", filename_str))?
                    }
                    FileMode::WRITE => {
                        OpenOptions::new().write(true).create(true).truncate(true).open(&filename_str).map_err(|_| format!("Failed to open file {} for writing", filename_str))?
                    }
                    FileMode::RANDOM => {
                        OpenOptions::new().read(true).write(true).create(true).open(&filename_str).map_err(|_| format!("Failed to open file {} for random access", filename_str))?
                    }
                };

                match mode {
                    FileMode::READ => {
                        self.open_files.insert(filename_str, FileHandle::Read(BufReader::new(file)));
                    }
                    FileMode::WRITE => {
                        self.open_files.insert(filename_str, FileHandle::Write(BufWriter::new(file)));
                    }
                    FileMode::RANDOM => {
                        self.open_files.insert(filename_str, FileHandle::Random(file));
                    }
                }
                
                Ok(())
            }
            Stmt::CloseFile { filename } => {
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("CLOSEFILE expects STRING filename, got {:?}", filename_val);
                        log_error!("{}", msg);
                        return Err(msg);
                    }
                };
                
                // Remove file from open_files (Rust will automatically close it)
                if self.open_files.remove(&filename_str).is_none() {
                    let msg = format!("File '{}' is not open", filename_str);
                    log_error!("{}", msg);
                    return Err(msg);
                }
                
                Ok(())
            }
            Stmt::ReadFile { filename, name } => {
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("READFILE expects STRING filename, got {:?}", filename_val);
                        log_error!("{}", msg);
                        return Err(msg);
                    }
                };
                
                // Get file handle
                let file_handle = self.open_files.get_mut(&filename_str)
                    .ok_or_else(|| format!("File '{}' is not open", filename_str))?;
                
                // Read a line from the file
                let mut line = String::new();
                match file_handle {
                    FileHandle::Read(reader) => {
                        reader.read_line(&mut line)
                            .map_err(|e| format!("Failed to read from file '{}': {}", filename_str, e))?;
                    },
                    FileHandle::Random(file) => {
                        // For random access files, read bytes until newline
                        let mut buffer = vec![0u8; 1];
                        loop {
                            match file.read_exact(&mut buffer) {
                                Ok(_) => {
                                    let ch = buffer[0] as char;
                                    if ch == '\n' {
                                        break;
                                    }
                                    line.push(ch);
                                },
                                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                                    break; // End of file
                                },
                                Err(e) => {
                                    return Err(format!("Failed to read from file '{}': {}", filename_str, e));
                                }
                            }
                        }
                    },
                    FileHandle::Write(_) => {
                        let msg = format!("Cannot read from file '{}' opened in WRITE mode", filename_str);
                        log_error!("{}", msg);
                        return Err(msg);
                    },
                }
                
                // Remove trailing newline
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                
                // Store in variable
                let var_type = self.variables_type.get(name)
                    .ok_or_else(|| format!("Variable '{}' not found", name))?;
                
                // Ensure variable is STRING type
                if !matches!(var_type, Type::STRING) {
                    let msg = format!("READFILE variable '{}' must be STRING type", name);
                    log_error!("{}", msg);
                    return Err(msg);
                }
                
                self.variables.insert(name.clone(), Value::String(line));
                Ok(())
            }
            Stmt::WriteFile { filename, exprs } => {
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("WRITEFILE expects STRING filename, got {:?}", filename_val);
                        log_error!("{}", msg);
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
                
                // Write to file
                match file_handle {
                    FileHandle::Write(writer) => {
                        writer.write_all(output.as_bytes())
                            .map_err(|e| format!("Failed to write to file '{}': {}", filename_str, e))?;
                        writer.flush()
                            .map_err(|e| format!("Failed to flush file '{}': {}", filename_str, e))?;
                    },
                    FileHandle::Random(file) => {
                        file.write_all(output.as_bytes())
                            .map_err(|e| format!("Failed to write to file '{}': {}", filename_str, e))?;
                        file.flush()
                            .map_err(|e| format!("Failed to flush file '{}': {}", filename_str, e))?;
                    },
                    FileHandle::Read(_) => {
                        let msg = format!("Cannot write to file '{}' opened in READ mode", filename_str);
                        log_error!("{}", msg);
                        return Err(msg);
                    },
                }
                
                Ok(())
            }
            Stmt::Seek { filename, address } => {
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("SEEK expects STRING filename, got {:?}", filename_val);
                        log_error!("{}", msg);
                        return Err(msg);
                    }
                };
                
                let address_val = self.evaluate_expr(address)?;
                let address_int = match address_val {
                    Value::Integer(i) => i,
                    _ => {
                        let msg = format!("SEEK expects INTEGER address, got {:?}", address_val);
                        log_error!("{}", msg);
                        return Err(msg);
                    }
                };
                
                // Get file handle (only RANDOM mode supports seek)
                let file_handle = self.open_files.get_mut(&filename_str)
                    .ok_or_else(|| format!("File '{}' is not open", filename_str))?;
                
                match file_handle {
                    FileHandle::Random(file) => {
                        file.seek(SeekFrom::Start(address_int as u64))
                            .map_err(|e| format!("Failed to seek in file '{}': {}", filename_str, e))?;
                    },
                    _ => {
                        let msg = format!("SEEK only works with files opened in RANDOM mode");
                        log_error!("{}", msg);
                        return Err(msg);
                    },
                }
                
                Ok(())
            }
            Stmt::GetRecord { filename, variable } => {
                // GetRecord reads a fixed-length record (typically for binary/random access files)
                // For simplicity, we'll treat it like ReadFile for now
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("GETRECORD expects STRING filename, got {:?}", filename_val);
                        log_error!("{}", msg);
                        return Err(msg);
                    }
                };
                
                // For now, treat GetRecord as ReadFile
                // TODO: Implement proper record-based reading
                self.evaluate_stmt(&Stmt::ReadFile {
                    filename: Box::new(Expr::String(filename_str.clone())),
                    name: variable.clone(),
                })
            }
            Stmt::PutRecord { filename, variable } => {
                // PutRecord writes a fixed-length record
                // For simplicity, we'll treat it like WriteFile for now
                let filename_val = self.evaluate_expr(filename)?;
                let filename_str = match filename_val {
                    Value::String(s) => s,
                    _ => {
                        let msg = format!("PUTRECORD expects STRING filename, got {:?}", filename_val);
                        log_error!("{}", msg);
                        return Err(msg);
                    }
                };
                
                // Get variable value (not used but kept for future record implementation)
                let _var_value = self.variables.get(variable)
                    .ok_or_else(|| format!("Variable '{}' not found", variable))?;
                
                // For now, treat PutRecord as WriteFile
                // TODO: Implement proper record-based writing
                self.evaluate_stmt(&Stmt::WriteFile {
                    filename: Box::new(Expr::String(filename_str)),
                    exprs: vec![Expr::Variable(variable.clone())],
                })
            }
            _ => {
                let msg = format!("Unsupported statement: {:?}", stmt);
                log_error!("{}", msg);
                Err(msg)
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
            _ => {
                let msg = format!("Unsupported type: {:?}", type_name);
                log_error!("{}", msg);
                Err(msg)
            }
        }
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
            Value::Array { .. } => format!("{:?}", value),
        }
    }

    pub fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(num) => {
                if num.contains('.') {
                    Ok(Value::Real(num.parse().map_err(|_| "Invalid real number")?))
                } else {
                    Ok(Value::Integer(num.parse().map_err(|_| "Invalid integer number")?))
                }
            }
            Expr::String(str) => Ok(Value::String(str.clone())),
            Expr::Char(ch) => {
                let c = ch.chars().nth(0).unwrap_or('\0');
                Ok(Value::Char(c))
            }
            Expr::Boolean(bool) => {
                match bool {
                    true => Ok(Value::Boolean(true)),
                    false => Ok(Value::Boolean(false)),
                }
            },
            Expr::Variable(var) => {
                self.variables.get(var)
                    .cloned()
                    .ok_or_else(|| format!("Variable {} not found", var))
            }
            Expr::BinaryOp(left, op, right) => {
                let left_val = self.evaluate_expr(left)?;
                let right_val = self.evaluate_expr(right)?;
                self.evaluate_binary_op(op.clone(), &left_val, &right_val)
            }
            Expr::UnaryOp(op, expr) => {
                self.evaluate_unary_op(op.clone(), expr)
            }
            Expr::FunctionCall { name, args } => {
                self.evaluate_function_call(name, &Some(args.clone()))
            }
            Expr::ArrayAccess { array, indices } => {
                // Evaluate indices first (before borrowing array)
                let index_vals : Vec<Value> = indices.iter()
                    .map(|idx| self.evaluate_expr(idx))
                    .collect::<Result<_, _>>()?;

                let array_val = self.variables.get(array)
                    .ok_or_else(|| format!("Array {} not found", array))?;

                let mut index_positions = Vec::new();
                for idx_val in index_vals {
                    match idx_val {
                        Value::Integer(i) => {
                            if i < 1 {
                                let msg = format!("Array index must be >= 1, got {}", i);
                                log_error!("{}", msg);
                                return Err(msg);
                            }
                            index_positions.push((i - 1) as usize);  // Convert 1-based to 0-based
                        }
                        _ => {
                            let msg = format!("Array index must be integer, got {:?}", idx_val);
                            log_error!("{}", msg);
                            return Err(msg);
                        }
                    }
                }

                match array_val {
                    Value::Array { element_type, dimensions, data } => {
                        let flat_index = self.calculate_array_index(index_positions, dimensions)?;
                        if flat_index >= data.len() {
                            let msg = format!("Array index out of bounds: {}", flat_index);
                            log_error!("{}", msg);
                            return Err(msg);
                        }
                        Ok(data[flat_index].clone())
                    }
                    _ => {
                        let msg = format!("Array access on non-array variable: {}", array);
                        log_error!("{}", msg);
                        Err(msg)
                    }
                }
            }
            _ => {
                let msg = format!("Unsupported expression: {:?}", expr);
                log_error!("Unsupported expression: {:?}", expr);
                Err(msg)
            }
        }
    }

    fn evaluate_function_call(&mut self, name: &str, args: &Option<Vec<Expr>>) -> Result<Value, String> {
        // Try built-in functions first
        if let Some(result) = self.evaluate_builtin_function(name, args) {
            return Ok(result);
        }
        
        // Try user-defined functions
        let function = self.functions.get(name)
            .ok_or_else(|| format!("Function '{}' not found", name))?
            .clone();  // Clone to avoid borrow issues
        
        // Evaluate arguments
        let arg_values: Vec<Value> = if let Some(arg_exprs) = args {
            arg_exprs.iter()
                .map(|expr| self.evaluate_expr(expr))
                .collect::<Result<_, _>>()?
        } else {
            Vec::new()
        };
        
        // Validate argument count
        if arg_values.len() != function.params.len() {
            let msg = format!(
                "Function '{}' expects {} arguments, got {}",
                name, function.params.len(), arg_values.len()
            );
            log_error!("{}", msg);
            return Err(msg);
        }
        
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
            if let Stmt::Return { value } = stmt {
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
        
        // Return the value (or default if no RETURN statement)
        Ok(return_value.unwrap_or_else(|| {
            // If no RETURN statement, return default value for return type
            self.default_value(&function.return_type).unwrap_or(Value::Integer(0))
        }))
    }

    fn evaluate_builtin_function(&mut self, name: &str, args: &Option<Vec<Expr>>) -> Option<Value> {
        match name {
            "MOD" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 2 {
                    let msg = format!("MOD requires 2 arguments, got {}", args_vec.len());
                    log_error!("{}", msg);
                    return None;
                }
                let arg1 = self.evaluate_expr(&args_vec[0]).ok()?;
                let arg2 = self.evaluate_expr(&args_vec[1]).ok()?;
                match (&arg1, &arg2) {
                    (Value::Integer(l), Value::Integer(r)) => {
                        if *r == 0 {
                            let msg = format!("Modulo by zero");
                            log_error!("{}", msg);
                            return None;
                        }
                        Some(Value::Integer(l % r))
                    }
                    _ => {
                        let msg = format!("MOD requires integer arguments, got {:?} and {:?}", arg1, arg2);
                        log_error!("{}", msg);
                        None
                    }
                }
            }
            "DIV" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 2 {
                    let msg = format!("DIV expects 2 arguments, got {}", args_vec.len());
                    log_error!("{}", msg);
                    return None;
                }
                let arg1 = self.evaluate_expr(&args_vec[0]).ok()?;
                let arg2 = self.evaluate_expr(&args_vec[1]).ok()?;
                match (&arg1, &arg2) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        if *y == 0 {
                            let msg = format!("Division by zero in DIV");
                            log_error!("{}", msg);
                            return None;
                        }
                        Some(Value::Integer(x / y))
                    }
                    _ => {
                        let msg = format!("DIV requires integer arguments, got {:?} and {:?}", arg1, arg2);
                        log_error!("{}", msg);
                        None
                    }
                }
            }
            "LENGTH" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    let msg = format!("LENGTH expects 1 argument, got {}", args_vec.len());
                    log_error!("{}", msg);
                    return None;
                }
                let str_val = self.evaluate_expr(&args_vec[0]).ok()?;
                match str_val {
                    Value::String(s) => Some(Value::Integer(s.len() as i32)),
                    _ => {
                        let msg = format!("LENGTH requires string argument, got {:?}", str_val);
                        log_error!("{}", msg);
                        None
                    }
                }
            }
            "UCASE" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    let msg = format!("UCASE expects 1 argument, got {}", args_vec.len());
                    log_error!("{}", msg);
                    return None;
                }
                let str_val = self.evaluate_expr(&args_vec[0]).ok()?;
                match str_val {
                    Value::String(s) => Some(Value::String(s.to_uppercase())),
                    Value::Char(c) => Some(Value::String(c.to_uppercase().to_string())),
                    _ => {
                        let msg = format!("UCASE requires string or char argument, got {:?}", str_val);
                        log_error!("{}", msg);
                        None
                    }
                }
            }
            "LCASE" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    let msg = format!("LCASE expects 1 argument, got {}", args_vec.len());
                    log_error!("{}", msg);
                    return None;
                }
                let str_val = self.evaluate_expr(&args_vec[0]).ok()?;
                match str_val {
                    Value::String(s) => Some(Value::String(s.to_lowercase())),
                    Value::Char(c) => Some(Value::String(c.to_lowercase().to_string())),
                    _ => {
                        let msg = format!("LCASE requires string or char argument, got {:?}", str_val);
                        log_error!("{}", msg);
                        None
                    }
                }
            }
            "SUBSTRING" | "MID" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 3 {
                    let msg = format!("{} expects 3 arguments (string, start, length), got {}", name, args_vec.len());
                    log_error!("{}", msg);
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
                        log_error!("{}", msg);
                        None
                    }
                }
            }
            "RIGHT" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 2 {
                    let msg = format!("RIGHT expects 2 arguments, got {}", args_vec.len());
                    log_error!("{}", msg);
                    return None;
                }
                let str_val = self.evaluate_expr(&args_vec[0]).ok()?;
                let length_val = self.evaluate_expr(&args_vec[1]).ok()?;
                match (&str_val, &length_val) {
                    (Value::String(s), Value::Integer(length)) => {
                        if *length < 0 {
                            let msg = format!("RIGHT requires non-negative length, got {}", length);
                            log_error!("{}", msg);
                            return None;
                        }
                        // Handle case where length > string length
                        let length = (*length as usize).min(s.len());
                        let start_idx = s.len().saturating_sub(length);
                        Some(Value::String(s[start_idx..].to_string()))
                    }
                    _ => {
                        let msg = format!("RIGHT expects (STRING, INTEGER) arguments, got {:?}, {:?}", str_val, length_val);
                        log_error!("{}", msg);
                        None
                    }
                }
            }
            "RANDOM" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 0 {
                    let msg = format!("RANDOM expects 0 argument, got {}", args_vec.len());
                    log_error!("{}", msg);
                    return None;
                }
                Some(Value::Real(rand::thread_rng().gen_range(0.0..=1.0)))
            }
            "RAND" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    let msg = format!("RAND expects 1 argument, got {}", args_vec.len());
                    log_error!("{}", msg);
                    return None;
                }
                let max_val = self.evaluate_expr(&args_vec[0]).ok()?;
                match &max_val {
                    Value::Integer(max) => Some(Value::Real(rand::thread_rng().gen_range(0.0..=*max as f64))),
                    _ => {
                        let msg = format!("RAND requires integer argument, got {:?}", max_val);
                        log_error!("{}", msg);
                        None
                    }
                }   
            }
            "ROUND" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 2 {
                    let msg = format!("ROUND expects 2 arguments, got {}", args_vec.len());
                    log_error!("{}", msg);
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
                        log_error!("{}", msg);
                        None
                    }
                }
            }
            "INT" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    let msg = format!("INT expects 1 argument, got {}", args_vec.len());
                    log_error!("{}", msg);
                    return None;
                }
                let val = self.evaluate_expr(&args_vec[0]).ok()?;
                match &val {
                    Value::Real(r) => Some(Value::Integer(r.floor() as i32)),
                    Value::Integer(i) => Some(Value::Integer(*i)),
                    _ => {
                        let msg = format!("INT requires numeric argument, got {:?}", val);
                        log_error!("{}", msg);
                        None
                    }
                }
            }
            "EOF" => {
                let args_vec = args.as_ref()?;
                if args_vec.len() != 1 {
                    let msg = format!("EOF expects 1 argument (filename), got {}", args_vec.len());
                    log_error!("{}", msg);
                    return None;
                }
                let filename_val = self.evaluate_expr(&args_vec[0]).ok()?;
                match filename_val {
                    Value::String(filename) => {
                        // Check if file is open
                        if let Some(file_handle) = self.open_files.get_mut(&filename) {
                            // Check if we're at EOF
                            match file_handle {
                                FileHandle::Read(reader) => {
                                    let buffer = reader.fill_buf().ok()?;
                                    Some(Value::Boolean(buffer.is_empty()))
                                },
                                FileHandle::Random(file) => {
                                    // For random access, check current position vs file size
                                    let pos = file.stream_position().ok()?;
                                    let metadata = file.metadata().ok()?;
                                    Some(Value::Boolean(pos >= metadata.len()))
                                },
                                FileHandle::Write(_) => {
                                    // Write mode - always false (can't be at EOF for writing)
                                    Some(Value::Boolean(false))
                                },
                            }
                        } else {
                            let msg = format!("File '{}' is not open", filename);
                            log_error!("{}", msg);
                            None
                        }
                    }
                    _ => {
                        let msg = format!("EOF expects STRING argument (filename), got {:?}", filename_val);
                        log_error!("{}", msg);
                        None
                    }
                }
            }
            _ => None,
        }
    }

    fn evaluate_unary_op(&mut self, op: UnaryOp, expr: &Expr) -> Result<Value, String> {
        match op {
            Negate => {
                let val = self.evaluate_expr(expr)?;
                match val {
                    Value::Integer(l) => Ok(Value::Integer(-l)),
                    Value::Real(l) => Ok(Value::Real(-l)),
                    _ => {
                        let msg = format!("Unsupported negation operation: {:?}", op);
                        log_error!("{}", msg);
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
                        log_error!("{}", msg);
                        Err(msg)
                    }
                }
            }
            _ => {
                let msg = format!("Unsupported unary operation: {:?}", op);
                log_error!("{}", msg);
                Err(msg)
            }
        }
    }

    fn evaluate_binary_op(&self, op: BinaryOp, left: &Value, right: &Value) -> Result<Value, String> {
        match op {
            Add => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
                    (Value::Real(l), Value::Real(r)) => Ok(Value::Real(l + r)),
                    (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
                    (Value::Char(l), Value::Char(r)) => Ok(Value::String(format!("{}{}", l, r))),
                    (Value::Real(l), Value::Integer(r)) => Ok(Value::Real(l + *r as f64)),
                    (Value::Integer(l), Value::Real(r)) => Ok(Value::Real(*l as f64 + r)),
                    _ => {
                        let msg = format!("Unsupported addition operation: {:?} with {:?} and {:?}", op, left, right);
                        log_error!("{}", msg);
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
                        log_error!("{}", msg);
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
                        log_error!("{}", msg);
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
            Div => {
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
                        log_error!("{}", msg);
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
                        log_error!("{}", msg);
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
                        log_error!("{}", msg);
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
                        log_error!("{}", msg);
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
                        log_error!("{}", msg);
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
                        log_error!("{}", msg);
                        Err(msg)
                    }
                }
            }
            And => {
                match (left, right) {
                    (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l && *r)),
                    _ => {
                        let msg = format!("Unsupported AND operation: {:?} with {:?} and {:?}", op, left, right);
                        log_error!("{}", msg);
                        Err(msg)
                    }
                }
            }
            Or => {
                match (left, right) {
                    (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l || *r)),
                    _ => {
                        let msg = format!("Unsupported OR operation: {:?} with {:?} and {:?}", op, left, right);
                        log_error!("{}", msg);
                        Err(msg)
                    }
                }
            }
            _ => {
                let msg = format!("Unsupported binary operation: {:?} with {:?} and {:?}", op, left, right);
                log_error!("{}", msg);
                Err(msg)
            }
        }
    }
}