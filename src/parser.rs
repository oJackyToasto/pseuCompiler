use crate::ast::{Expr, BinaryOp, UnaryOp, Stmt, Type, FileMode, CaseBranch, TypeDeclarationVariant, TypeField, Function, Param, Procedure};
use crate::lexer::{Token, Lexer};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        Parser { tokens, pos: 0 }
    }

    fn next_token(&mut self) -> &Token {
        self.pos += 1;
        &self.tokens[self.pos]
    }
    
    fn current_token(&self) -> &Token {
        if self.pos >= self.tokens.len() {
            &self.tokens[self.tokens.len() - 1]  // Return last token (EOF)
        } else {
        &self.tokens[self.pos]
        }
    }
    
    fn advance(&mut self) {
        self.pos += 1;
    }
    
    fn parse_number(&mut self) -> Result<Expr, String> {
        if let Token::Number(n) = self.current_token() {
            let number = n.clone();
            self.advance();
            Ok(Expr::Number(number))
        } else {
            Err("Expected number".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<Expr, String> {
        match self.current_token() {
            Token::String(s) => {
                let string = s.clone();
                self.advance();
                Ok(Expr::String(string))
            }
            _ => Err("Expected string".to_string()),
        }
    }

    fn parse_variable(&mut self) -> Result<Expr, String> {
        match self.current_token() {
            Token::Identifier(v) => {
                let variable = v.clone();
                self.advance();
                Ok(Expr::Variable(variable))
            }
            _ => Err("Expected variable".to_string()),
        }
    }

    fn parse_char(&mut self) -> Result<Expr, String> {
        match self.current_token() {
            Token::Char(c) => {
                let char = c.clone();
                self.advance();
                Ok(Expr::Char(char))
            }
            _ => Err("Expected char".to_string()),
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.current_token() {
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(expr)))
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Negate, Box::new(expr)))
            }
            _ => self.parse_primary(),
        }
    }

    fn peek_binary_op(&self) -> Option<BinaryOp> {  // Return BinaryOp, not &BinaryOp
        match self.current_token() {
            Token::Plus => Some(BinaryOp::Add),
            Token::Minus => Some(BinaryOp::Subtract),
            Token::Multiply => Some(BinaryOp::Multiply),
            Token::Divide => Some(BinaryOp::Divide),
            Token::Modulus => Some(BinaryOp::Modulus),
            Token::Equals => Some(BinaryOp::Equals),
            Token::NotEquals => Some(BinaryOp::NotEquals),
            Token::LessThan => Some(BinaryOp::LessThan),
            Token::GreaterThan => Some(BinaryOp::GreaterThan),
            Token::LessThanOrEqual => Some(BinaryOp::LessThanOrEqual),
            Token::GreaterThanOrEqual => Some(BinaryOp::GreaterThanOrEqual),
            Token::And => Some(BinaryOp::And),
            Token::Or => Some(BinaryOp::Or),
            _ => None,
        }
    }

    pub fn parse_statement(&mut self) -> Result<Stmt, String> {
        match self.current_token() {
            Token::Keyword(kw) => match kw.as_str() {
                "DECLARE" => self.parse_declare(),
                "TYPE" => self.parse_type_declaration(),
                "IF" => self.parse_if(),
                "WHILE" => self.parse_while(),
                "FOR" => self.parse_for(),
                "REPEAT" => self.parse_repeat_until(),
                "CASE" => self.parse_case(),
                "FUNCTION" => self.parse_function_declaration(),
                "PROCEDURE" => self.parse_procedure_declaration(),
                "CALL" => self.parse_call(),
                "BREAK" => {
                    self.advance();
                    Ok(Stmt::Break)
                },
                "INPUT" => self.parse_input(),
                "OUTPUT" => self.parse_output(),
                "OPENFILE" => self.parse_openfile(),
                "CLOSEFILE" => self.parse_closefile(),
                "READFILE" => self.parse_readfile(),
                "WRITEFILE" => self.parse_writefile(),
                "SEEK" => self.parse_seek(),
                "GETRECORD" => self.parse_getrecord(),
                "PUTRECORD" => self.parse_putrecord(),
                "RETURN" => self.parse_return(),
                _ => Err(format!("Unexpected keyword: {}", kw)),
            }

            Token::Identifier(_) => {
                self.parse_assignment()
            },
            _ => Err("Expected statement".to_string()),
        }
    }

    fn parse_procedure_declaration(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("PROCEDURE".to_string()))?;
        
        // Parse procedure name
        let name = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err("Expected procedure name".to_string()),
        };
        
        // Expect opening parenthesis
        self.expect(Token::LeftParen)?;
        
        // Parse parameters (can be empty)
        let mut params = Vec::new();
        
        // Check if there are parameters (not immediately closing paren)
        if !matches!(self.current_token(), Token::RightParen) {
            loop {
                // Parse parameter name
                let param_name = match self.current_token() {
                    Token::Identifier(n) => {
                        let name = n.clone();
                        self.advance();
                        name
                    }
                    _ => return Err("Expected parameter name".to_string()),
                };
                
                // Expect colon
                self.expect(Token::Colon)?;
                
                // Parse parameter type
                let param_type = self.parse_type()?;
                
                params.push(Param {
                    name: param_name,
                    type_name: param_type,
                });
                
                // Check for more parameters or closing paren
                match self.current_token() {
                    Token::Comma => {
                        self.advance();
                        continue;
                    }
                    Token::RightParen => {
                        break;
                    }
                    _ => return Err("Expected comma or closing parenthesis".to_string()),
                }
            }
        }
        
        // Expect closing parenthesis
        self.expect(Token::RightParen)?;
        
        // Parse procedure body until ENDPROCEDURE
        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDPROCEDURE") {
            body.push(self.parse_statement()?);
        }
        
        // Expect ENDPROCEDURE
        self.expect(Token::Keyword("ENDPROCEDURE".to_string()))?;
        
        Ok(Stmt::ProcedureDeclaration {
            procedure: Procedure {
                name,
                params,
                body,
            },
        })
    }
    
    fn parse_call(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("CALL".to_string()))?;
        
        // Parse procedure name
        let name = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err("Expected procedure name after CALL".to_string()),
        };
        
        // Check for arguments
        let args = match self.current_token() {
            // CALL <identifier>() - empty parentheses
            Token::LeftParen => {
                self.advance();
                
                // Check if immediately closing paren (no args)
                if matches!(self.current_token(), Token::RightParen) {
                    self.advance();
                    Some(Vec::new()) // Empty args list
                } else {
                    // Parse argument expressions
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_expression()?);
                        
                        match self.current_token() {
                            Token::Comma => {
                                self.advance();
                                continue;
                            }
                            Token::RightParen => {
                                self.advance();
                                break;
                            }
                            _ => return Err("Expected comma or closing parenthesis in CALL arguments".to_string()),
                        }
                    }
                    Some(args)
                }
            }
            // CALL <identifier> - no parentheses (only when no params)
            _ => None,
        };
        
        Ok(Stmt::Call {
            name,
            args,
        })
    }
    
    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("RETURN".to_string()))?;
        
        // Check if there's a return value (expression)
        // If the next token is a newline, EOF, or ENDFUNCTION, there's no value
        let value = if matches!(
            self.current_token(),
            Token::Newline | Token::EOF
        ) || matches!(
            self.current_token(),
            Token::Keyword(kw) if kw == "ENDFUNCTION" || kw == "ENDIF" || kw == "ELSE"
        ) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        
        Ok(Stmt::Return { value })
    }

    fn parse_function_declaration(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("FUNCTION".to_string()))?;

        let name = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err("Expected function name".to_string()),
        };
        
        self.expect(Token::LeftParen)?;

        let mut params = Vec::new();

        if !matches!(self.current_token(), Token::RightParen) {
            loop {
                let param = match self.current_token() {
                    Token::Identifier(n) => {
                        let name = n.clone();
                        self.advance();
                        name
                    }
                    _ => return Err("Expected parameter name".to_string()),
                };
                
                self.expect(Token::Colon)?;

                let param_type = self.parse_type()?;

                params.push(Param {
                    name: param,
                    type_name: param_type,
                });

                match self.current_token() {
                    Token::Comma => {
                        self.advance();
                        continue;
                    }
                    Token::RightParen => {
                        break;
                    }
                    _ => return Err("Expected comma or closing parenthesis".to_string()),
                }
            }
        }

        self.expect(Token::RightParen)?;
        self.expect(Token::Keyword("RETURNS".to_string()))?;

        let return_type = self.parse_type()?;

        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDFUNCTION") {
            body.push(self.parse_statement()?);
        }

        self.expect(Token::Keyword("ENDFUNCTION".to_string()))?;

        Ok(Stmt::FunctionDeclaration {
            function: Function {
                name,
                params,
                return_type,
                body,
            },
        })
    }
    
    fn parse_define(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("DEFINE".to_string()))?;
        
        let name = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err("Expected identifier after DEFINE".to_string()),
        };
        
        self.expect(Token::LeftParen)?;
        
        let mut values = Vec::new();
        loop {
            match self.current_token() {
                Token::Identifier(v) | Token::Keyword(v) => {
                    values.push(v.clone());
                    self.advance();
                }
                _ => return Err("Expected enum value".to_string()),
            }
            
            match self.current_token() {
                Token::Comma => {
                    self.advance();
                    continue;
                }
                Token::RightParen => {
                    self.advance();
                    break;
                }
                _ => return Err("Expected comma or closing parenthesis".to_string()),
            }
        }
        
        self.expect(Token::Colon)?;
        
        let type_name = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err("Expected type name".to_string()),
        };
        
        Ok(Stmt::Define {
            name,
            values,
            type_name,
        })
    }

    fn parse_type_declaration(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("TYPE".to_string()))?;
        
        let name = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err("Expected type name".to_string()),
        };
        
        // Check for different TYPE syntaxes
        match self.current_token() {
            // TYPE <name> = (value1, value2, ...) - Enum
            Token::Equals => {
                self.advance();
                self.expect(Token::LeftParen)?;
                
                let mut values = Vec::new();
                loop {
                    match self.current_token() {
                        Token::Identifier(v) | Token::Keyword(v) => {
                            values.push(v.clone());
                            self.advance();
                        }
                        _ => return Err("Expected enum value".to_string()),
                    }
                    
                    match self.current_token() {
                        Token::Comma => {
                            self.advance();
                            continue;
                        }
                        Token::RightParen => {
                            self.advance();
                            break;
                        }
                        _ => return Err("Expected comma or closing parenthesis".to_string()),
                    }
                }
                
                Ok(Stmt::TypeDeclaration {
                    name,
                    variant: TypeDeclarationVariant::Enum { values },
                })
            }
            
            // TYPE <name> = ^<type> - Pointer
            Token::Caret => {
                self.advance();
                let points_to = self.parse_type()?;
                
                Ok(Stmt::TypeDeclaration {
                    name,
                    variant: TypeDeclarationVariant::Pointer {
                        points_to: Box::new(points_to),
                    },
                })
            }
            
            // TYPE <name> = SET OF <type> - Set
            Token::Keyword(kw) if kw == "SET" => {
                self.advance();
                self.expect(Token::Keyword("OF".to_string()))?;
                let element_type = self.parse_type()?;
                
                Ok(Stmt::TypeDeclaration {
                    name,
                    variant: TypeDeclarationVariant::Set {
                        element_type: Box::new(element_type),
                    },
                })
            }
            
            // TYPE <name> ... DECLARE ... ENDTYPE - Record (existing)
            _ => {
                let mut fields = Vec::new();
                
                while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDTYPE") {
                    if matches!(self.current_token(), Token::Keyword(kw) if kw == "DECLARE") {
                        self.advance();
                        
                        let field_name = match self.current_token() {
                            Token::Identifier(n) => {
                                let name = n.clone();
                                self.advance();
                                name
                            }
                            _ => return Err("Expected field name".to_string()),
                        };
                        
                        self.expect(Token::Colon)?;
                        let field_type = self.parse_type()?;
                        
                        fields.push(TypeField {
                            name: field_name,
                            type_name: field_type,
                        });
                    } else {
                        return Err("Expected DECLARE or ENDTYPE".to_string());
                    }
                }
                
                self.expect(Token::Keyword("ENDTYPE".to_string()))?;
                
                Ok(Stmt::TypeDeclaration {
                    name,
                    variant: TypeDeclarationVariant::Record { fields },
                })
            }
        }
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("IF".to_string()))?;

        let condition = self.parse_expression()?;

        self.expect(Token::Keyword("DO".to_string()))?;

        let mut then_stmt = Vec::new();

        while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDIF" || kw == "ELSE") {
            then_stmt.push(self.parse_statement()?);
        }

        let else_stmt = if matches!(self.current_token(), Token::Keyword(kw) if kw == "ELSE") {
            self.advance();

            if matches!(self.current_token(), Token::Keyword(kw) if kw == "IF") {
                let nested_if = self.parse_if()?;
                Some(vec![nested_if])
            } else {
                let mut else_body = Vec::new();
                while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDIF") {
                    else_body.push(self.parse_statement()?);
                }
                Some(else_body)
            }
        } else {
            None
        };

        self.expect(Token::Keyword("ENDIF".to_string()))?;

        Ok(Stmt::If {
            condition: Box::new(condition),
            then_stmt,
            else_stmt,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("WHILE".to_string()))?;
    
        let condition = self.parse_expression()?;
        self.expect(Token::Keyword("DO".to_string()))?;
    
        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDWHILE") {
            body.push(self.parse_statement()?);
        }
    
        self.expect(Token::Keyword("ENDWHILE".to_string()))?;
    
        Ok(Stmt::While {
            condition: Box::new(condition),
            body,
        })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("FOR".to_string()))?;

        // Parse counter variable name
        let counter = match self.current_token() {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err("Expected counter variable name in FOR loop".to_string()),
        };
        
        self.expect(Token::LeftArrow)?;
        
        let start = self.parse_expression()?;

        self.expect(Token::Keyword("TO".to_string()))?;

        let end = self.parse_expression()?;
        
        let step = if matches!(self.current_token(), Token::Keyword(kw) if kw == "STEP") {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::Keyword(kw) if kw == "NEXT") {
            body.push(self.parse_statement()?);
        }
        
        self.expect(Token::Keyword("NEXT".to_string()))?;
        
        let next_counter = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err("Expected counter variable name after NEXT".to_string()),
        };
        
        if next_counter != counter {
            return Err(format!("NEXT counter '{}' does not match FOR counter '{}'", next_counter, counter));
        }
        
        Ok(Stmt::For {
            counter,
            start: Box::new(start),
            end: Box::new(end),
            step,
            body,
        })
    }

    fn parse_repeat_until(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("REPEAT".to_string()))?;

        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::Keyword(kw) if kw == "UNTIL") {
            body.push(self.parse_statement()?);
        }

        self.expect(Token::Keyword("UNTIL".to_string()))?;

        let condition = self.parse_expression()?;

        Ok(Stmt::RepeatUntil {
                body,
                condition: Box::new(condition),
            }
        )
    }

    fn parse_case(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("CASE".to_string()))?;
        self.expect(Token::Keyword("OF".to_string()))?;
        
        let expression = self.parse_expression()?;
        
        let mut cases = Vec::new();
        let mut otherwise = None;
        
        while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDCASE") {
            if matches!(self.current_token(), Token::Keyword(kw) if kw == "OTHERWISE") {
                self.advance();
                self.expect(Token::Colon)?;
                
                let mut otherwise_body = Vec::new();
                while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDCASE") {
                    otherwise_body.push(self.parse_statement()?);
                }
                otherwise = Some(otherwise_body);
                break;
            }
            
            let value = self.parse_expression()?;
            
            self.expect(Token::Colon)?;
            
            let mut body = Vec::new();
            
            while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDCASE" || kw == "OTHERWISE") {
                let is_case_value = matches!(
                    self.current_token(),
                    Token::Identifier(_) | Token::Number(_) | Token::String(_) | Token::Char(_)
                );
                
                if is_case_value && self.pos + 1 < self.tokens.len() {
                    if matches!(self.tokens[self.pos + 1], Token::Colon) {
                        break;
                    }
                }
                
                body.push(self.parse_statement()?);
            }
            
            cases.push(CaseBranch {
                value: Box::new(value),
                body,
            });
        }
        
        self.expect(Token::Keyword("ENDCASE".to_string()))?;
        
        Ok(Stmt::Case {
            expression: Box::new(expression),
            cases,
            otherwise,
        })
    }

    fn parse_assignment(&mut self) -> Result<Stmt, String> {
        let name = match self.current_token() {
            Token::Identifier(n) => {
                let var_name = n.clone();
                self.advance();
                var_name
            },
            _ => return Err("Expected identifier".to_string()),
        };

        let has_index = matches!(self.current_token(), Token::LeftBracket);
        let index = if has_index {
            self.advance();
            let idx = self.parse_expression()?;
            self.expect(Token::RightBracket)?;
            Some(Box::new(idx))
        } else {
            None
        };

        self.expect(Token::LeftArrow)?;

        let value = self.parse_expression()?;

        Ok(Stmt::Assign {
            name,
            index,
            expression: Box::new(value),
        })
    }
    
    fn parse_input(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("INPUT".to_string()))?;

         match self.current_token() {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Stmt::Input { name })
            }
            _ => Err("Expected identifier".to_string()),
        }
    }
        
    fn parse_output(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("OUTPUT".to_string()))?;
        
        let mut exprs = Vec::new();

        exprs.push(self.parse_expression()?);
        
        while matches!(self.current_token(), Token::Comma) {
            self.advance();
            exprs.push(self.parse_expression()?);
        }

        Ok(Stmt::Output { exprs })  
    }

    fn parse_declare(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("DECLARE".to_string()))?;

        let mut declarations = vec![self.parse_one_declare()?];

        while matches!(self.current_token(), Token::Comma) {
            self.advance();
            declarations.push(self.parse_one_declare()?);
        }

        Ok(declarations.into_iter().next().unwrap())
    }

    fn parse_one_declare(&mut self) -> Result<Stmt, String> {
        let name = match self.current_token() {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err("Expected identifier".to_string()),
        };
        
        let initial_value = if matches!(self.current_token(), Token::LeftArrow) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        
        self.expect(Token::Colon)?;

        let type_name = self.parse_type()?;

        Ok(Stmt::Declare {
            name, 
            type_name,
            initial_value, 
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        if let Token::Keyword(kw) = self.current_token() {
            if kw == "ARRAY" {
                self.advance();
    
                let mut dimensions = Vec::new();
    
                while matches!(self.current_token(), Token::LeftBracket) {
                    self.advance();
                    let start = self.parse_expression()?;
                    self.expect(Token::Comma)?;
                    let end = self.parse_expression()?;
                    self.expect(Token::RightBracket)?;
                    dimensions.push((Box::new(start), Box::new(end)));
                }
    
                self.expect(Token::Keyword("OF".to_string()))?;
    
                let element_type = Box::new(self.parse_type()?);
                
                return Ok(Type::ARRAY {
                    dimensions,
                    element_type,
                });
            }
        }
        
        self.parse_simple_types()
    }
    
    fn parse_simple_types(&mut self) -> Result<Type, String> {
        let current_token = self.current_token();
        
        if let Token::Identifier(name) = current_token {
            let type_name = name.clone();
            self.advance();
            return Ok(Type::Custom(type_name));
        }
    
        if let Token::Keyword(kw) = current_token {
            let kw_str = kw.clone();
            self.advance();
            return match kw_str.as_str() {
                "INTEGER" => Ok(Type::INTEGER),
                "REAL" => Ok(Type::REAL),
                "STRING" => Ok(Type::STRING),
                "CHAR" => Ok(Type::CHAR),
                "BOOLEAN" | "BOOL" => Ok(Type::BOOLEAN),
                "DATE" => Ok(Type::DATE),
                _ => Err(format!("Unknown type: {}", kw_str)),
            };
        }
        
        Err("Expected type".to_string())
    }

    fn parse_openfile(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("OPENFILE".to_string()))?;
        
        let filename = self.parse_expression()?;
        
        self.expect(Token::Keyword("FOR".to_string()))?;
        
        let mode = match self.current_token() {
            Token::Keyword(kw) => {
                let kw_str = kw.clone();
                self.advance();
                match kw_str.as_str() {
                    "READ" => FileMode::READ,
                    "WRITE" => FileMode::WRITE,
                    "RANDOM" => FileMode::RANDOM,
                    _ => return Err(format!("Expected READ, WRITE, or RANDOM, found {}", kw_str)),
                }
            }
            _ => return Err("Expected READ, WRITE, or RANDOM after FOR".to_string()),
        };
        
        Ok(Stmt::OpenFile {
            filename: Box::new(filename),
            mode,
        })
    }

    fn parse_closefile(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("CLOSEFILE".to_string()))?;
        
        let filename = self.parse_expression()?;
        
        Ok(Stmt::CloseFile {
            filename: Box::new(filename),
        })
    }

    fn parse_readfile(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("READFILE".to_string()))?;
        
        let filename = self.parse_expression()?;
        self.expect(Token::Comma)?;
        
        let variable = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err("Expected variable name after comma in READFILE".to_string()),
        };
        
        Ok(Stmt::ReadFile {
            filename: Box::new(filename),
            name: variable,
        })
    }

    fn parse_writefile(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("WRITEFILE".to_string()))?;
        
        let filename = self.parse_expression()?;
        self.expect(Token::Comma)?;
        
        let mut exprs = Vec::new();
        exprs.push(self.parse_expression()?);
        
        while matches!(self.current_token(), Token::Comma) {
            self.advance();
            exprs.push(self.parse_expression()?);
        }
        
        Ok(Stmt::WriteFile {
            filename: Box::new(filename),
            exprs,
        })
    }

    fn parse_seek(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("SEEK".to_string()))?;
        
        let filename = self.parse_expression()?;
        self.expect(Token::Comma)?;
        
        let address = self.parse_expression()?;
        
        Ok(Stmt::Seek {
            filename: Box::new(filename),
            address: Box::new(address),
        })
    }

    fn parse_getrecord(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("GETRECORD".to_string()))?;
        
        let filename = self.parse_expression()?;
        self.expect(Token::Comma)?;
        
        let variable = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err("Expected variable name after comma in GETRECORD".to_string()),
        };
        
        Ok(Stmt::GetRecord {
            filename: Box::new(filename),
            variable,
        })
    }

    fn parse_putrecord(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("PUTRECORD".to_string()))?;
        
        let filename = self.parse_expression()?;
        self.expect(Token::Comma)?;
        
        let variable = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err("Expected variable name after comma in PUTRECORD".to_string()),
        };
        
        Ok(Stmt::PutRecord {
            filename: Box::new(filename),
            variable,
        })
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current_token() {
            Token::Number(_) => self.parse_number(),
            Token::String(_) => self.parse_string(),
            Token::Char(_) => self.parse_char(),
            Token::Identifier(name) | Token::Keyword(name) => {
                let var_name = name.clone();
                self.advance();
                
                // Check for field access (object.field)
                if matches!(self.current_token(), Token::Dot) {
                    self.advance();
                    let field = match self.current_token() {
                        Token::Identifier(f) => {
                            let f = f.clone();
                            self.advance();
                            f
                        }
                        _ => return Err("Expected field name after dot".to_string()),
                    };
                    return Ok(Expr::FieldAccess {
                        object: Box::new(Expr::Variable(var_name)),
                        field,
                    });
                }
                
                // Check for pointer dereference (var^)
                if matches!(self.current_token(), Token::Caret) {
                    self.advance();
                    return Ok(Expr::PointerDeref {
                        pointer: Box::new(Expr::Variable(var_name)),
                    });
                }
                
                // Check for function call or array access
                if let Token::LeftParen = self.current_token() {
                    return self.parse_function_call(var_name);
                }
                if let Token::LeftBracket = self.current_token() {
                    return self.parse_array_access(var_name);
                }
                
                Ok(Expr::Variable(var_name))
            },
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            },
            Token::Caret => {
                self.advance();
                let target = self.parse_primary()?;
                Ok(Expr::PointerRef {
                    target: Box::new(target),
                })
            }
            _ => Err("Expected primary expression".to_string()),
        }
    }

    fn parse_function_call(&mut self, name: String) -> Result<Expr, String> {
        self.expect(Token::LeftParen)?;
        let args = self.parse_function_call_args()?;
        self.expect(Token::RightParen)?;
        Ok(Expr::FunctionCall { name, args })
    }

    fn parse_array_access(&mut self, name: String) -> Result<Expr, String> {
        self.expect(Token::LeftBracket)?;
        let index = self.parse_expression()?;
        self.expect(Token::RightBracket)?;
        Ok(Expr::ArrayAccess { array: name, index: Box::new(index) })
    }

    fn parse_function_call_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        
        if let Token::RightParen = self.current_token() {
            return Ok(args);
        }
        
        args.push(self.parse_expression()?);
        
        while let Token::Comma = self.current_token() {
            self.advance();
            args.push(self.parse_expression()?);
        }
        
        Ok(args)
    }

    pub fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_binary_expression(0) 
    }

    fn parse_binary_expression(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
    
        while let Some(op) = self.peek_binary_op() {
            let prec = op.precedence();
            if prec < min_prec {
                break;
            }
            self.advance();
            let right = self.parse_binary_expression(prec + 1)?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !matches!(self.current_token(), Token::EOF) {
            // Skip newlines between statements
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.current_token() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, found {:?}", expected, self.current_token()))
        }
    }
}