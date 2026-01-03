use crate::ast::{Expr, BinaryOp, UnaryOp, Stmt, Type, FileMode, CaseBranch, TypeDeclarationVariant, TypeField, Function, Param, Procedure};
use crate::lexer::{Token, Lexer, TokenWithPos};

pub struct Parser {
    tokens: Vec<Token>,
    token_positions: Vec<(usize, usize)>, // (line, column) for each token
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let tokens_with_pos = lexer.tokenize_with_pos();
        
        let mut tokens = Vec::new();
        let mut positions = Vec::new();
        
        for TokenWithPos { token, line, column } in tokens_with_pos {
            positions.push((line, column));
            tokens.push(token);
        }
        
        Parser { 
            tokens, 
            token_positions: positions,
            pos: 0,
        }
    }
    
    fn get_position(&self) -> (usize, usize) {
        if self.pos < self.token_positions.len() {
            self.token_positions[self.pos]
        } else if !self.token_positions.is_empty() {
            self.token_positions[self.token_positions.len() - 1]
        } else {
            (1, 1)
        }
    }
    
    fn error_with_pos(&self, msg: &str) -> String {
        let (line, column) = self.get_position();
        format!("{} at line {}:{}", msg, line, column)
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
            Err(self.error_with_pos("Expected number"))
        }
    }

    fn parse_string(&mut self) -> Result<Expr, String> {
        match self.current_token() {
            Token::String(s) => {
                let string = s.clone();
                self.advance();
                Ok(Expr::String(string))
            }
            _ => Err(self.error_with_pos("Expected string")),
        }
    }

    fn parse_variable(&mut self) -> Result<Expr, String> {
        match self.current_token() {
            Token::Identifier(v) => {
                let variable = v.clone();
                self.advance();
                Ok(Expr::Variable(variable))
            }
            _ => Err(self.error_with_pos("Expected variable")),
        }
    }

    fn parse_char(&mut self) -> Result<Expr, String> {
        match self.current_token() {
            Token::Char(c) => {
                let char = c.clone();
                self.advance();
                Ok(Expr::Char(char))
            }
            _ => Err(self.error_with_pos("Expected char")),
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
                _ => Err(self.error_with_pos(&format!("Unexpected keyword: {}", kw))),
            }

            Token::Identifier(_) => {
                self.parse_assignment()
            },
            _ => Err(self.error_with_pos("Expected statement")),
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
            _ => return Err(self.error_with_pos("Expected procedure name")),
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
                    _ => return Err(self.error_with_pos("Expected parameter name")),
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
                    _ => return Err(self.error_with_pos("Expected comma or closing parenthesis")),
                }
            }
        }
        
        // Expect closing parenthesis
        self.expect(Token::RightParen)?;
        
        // Parse procedure body until ENDPROCEDURE
        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDPROCEDURE") {
            // Skip leading newlines (whitespace)
            while matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
            
            // Check if we hit the end keyword
            if matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDPROCEDURE") {
                break;
            }
            
            body.push(self.parse_statement()?);
            
            // Consume trailing newline (statement terminator)
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
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
            _ => return Err(self.error_with_pos("Expected procedure name after CALL")),
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
                            _ => return Err(self.error_with_pos("Expected comma or closing parenthesis in CALL arguments")),
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
            _ => return Err(self.error_with_pos("Expected function name")),
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
                    _ => return Err(self.error_with_pos("Expected parameter name")),
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
                    _ => return Err(self.error_with_pos("Expected comma or closing parenthesis")),
                }
            }
        }

        self.expect(Token::RightParen)?;
        self.expect(Token::Keyword("RETURNS".to_string()))?;

        let return_type = self.parse_type()?;

        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDFUNCTION") {
            // Skip leading newlines (whitespace)
            while matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
            
            // Check if we hit the end keyword
            if matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDFUNCTION") {
                break;
            }
            
            body.push(self.parse_statement()?);
            
            // Consume trailing newline (statement terminator)
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
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
            _ => return Err(self.error_with_pos("Expected identifier after DEFINE")),
        };
        
        self.expect(Token::LeftParen)?;
        
        let mut values = Vec::new();
        loop {
            match self.current_token() {
                Token::Identifier(v) | Token::Keyword(v) => {
                    values.push(v.clone());
                    self.advance();
                }
                _ => return Err(self.error_with_pos("Expected enum value")),
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
                _ => return Err(self.error_with_pos("Expected comma or closing parenthesis")),
            }
        }
        
        self.expect(Token::Colon)?;
        
        let type_name = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err(self.error_with_pos("Expected type name")),
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
            _ => return Err(self.error_with_pos("Expected type name")),
        };
        
        // Check for different TYPE syntaxes
        match self.current_token() {
            // TYPE <name> = ... - Can be Enum, Pointer, or Set
            Token::Equals => {
                self.advance();
                
                // Check what comes after =
                match self.current_token() {
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
                    
                    // TYPE <name> = (value1, value2, ...) - Enum
                    Token::LeftParen => {
                        self.advance();
                        
                        let mut values = Vec::new();
                        loop {
                            match self.current_token() {
                                Token::Identifier(v) | Token::Keyword(v) => {
                                    values.push(v.clone());
                                    self.advance();
                                }
                                _ => return Err(self.error_with_pos("Expected enum value")),
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
                                _ => return Err(self.error_with_pos("Expected comma or closing parenthesis")),
                            }
                        }
                        
                        Ok(Stmt::TypeDeclaration {
                            name,
                            variant: TypeDeclarationVariant::Enum { values },
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
                    
                    _ => return Err(self.error_with_pos("Expected ^, (, or SET after = in TYPE declaration")),
                }
            }
            
            // TYPE <name> = ^<type> - Pointer (without =, direct syntax)
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
            
            // TYPE <name> = SET OF <type> - Set (without =, direct syntax)
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
                    // Skip leading newlines (whitespace)
                    while matches!(self.current_token(), Token::Newline) {
                        self.advance();
                    }
                    
                    // Check if we hit the end keyword
                    if matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDTYPE") {
                        break;
                    }
                    
                    if matches!(self.current_token(), Token::Keyword(kw) if kw == "DECLARE") {
                        self.advance();
                        
                        let field_name = match self.current_token() {
                            Token::Identifier(n) => {
                                let name = n.clone();
                                self.advance();
                                name
                            }
                            _ => return Err(self.error_with_pos("Expected field name")),
                        };
                        
                        self.expect(Token::Colon)?;
                        let field_type = self.parse_type()?;
                        
                        fields.push(TypeField {
                            name: field_name,
                            type_name: field_type,
                        });
                        
                        // Consume trailing newline after DECLARE statement
                        if matches!(self.current_token(), Token::Newline) {
                            self.advance();
                        }
                    } else {
                        return Err(self.error_with_pos("Expected DECLARE or ENDTYPE"));
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

        self.expect(Token::Keyword("THEN".to_string()))?;

        let mut then_stmt = Vec::new();

        while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDIF" || kw == "ELSE") {
            // Skip leading newlines (whitespace)
            while matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
            
            // Check if we hit the end keyword
            if matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDIF" || kw == "ELSE") {
                break;
            }
            
            then_stmt.push(self.parse_statement()?);
            
            // Consume trailing newline (statement terminator)
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
        }

        let else_stmt = if matches!(self.current_token(), Token::Keyword(kw) if kw == "ELSE") {
            self.advance();

            if matches!(self.current_token(), Token::Keyword(kw) if kw == "IF") {
                let nested_if = self.parse_if()?;
                Some(vec![nested_if])
            } else {
                let mut else_body = Vec::new();
                while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDIF") {
                    // Skip leading newlines (whitespace)
                    while matches!(self.current_token(), Token::Newline) {
                        self.advance();
                    }
                    
                    // Check if we hit the end keyword
                    if matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDIF") {
                        break;
                    }
                    
                    else_body.push(self.parse_statement()?);
                    
                    // Consume trailing newline (statement terminator)
                    if matches!(self.current_token(), Token::Newline) {
                        self.advance();
                    }
                }
                Some(else_body)
            }
        } else {
            None
        };

        self.expect(Token::Keyword("ENDIF".to_string()))?;
        
        // Skip a single newline after ENDIF (if present)
        // This allows the outer IF to find its ENDIF when there's a nested IF
        if matches!(self.current_token(), Token::Newline) {
            self.advance();
        }

        Ok(Stmt::If {
            condition: Box::new(condition),
            then_stmt,
            else_stmt,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Keyword("WHILE".to_string()))?;
    
        let condition = self.parse_expression()?;
    
        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDWHILE") {
            // Skip leading newlines (whitespace)
            while matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
            
            // Check if we hit the end keyword
            if matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDWHILE") {
                break;
            }
            
            body.push(self.parse_statement()?);
            
            // Consume trailing newline (statement terminator)
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
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
            _ => return Err(self.error_with_pos("Expected counter variable name in FOR loop")),
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
            // Skip leading newlines (whitespace)
            while matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
            
            // Check if we hit the end keyword
            if matches!(self.current_token(), Token::Keyword(kw) if kw == "NEXT") {
                break;
            }
            
            body.push(self.parse_statement()?);
            
            // Consume trailing newline (statement terminator)
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
        }
        
        self.expect(Token::Keyword("NEXT".to_string()))?;
        
        let next_counter = match self.current_token() {
            Token::Identifier(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err(self.error_with_pos("Expected counter variable name after NEXT")),
        };
        
        if next_counter != counter {
            return Err(self.error_with_pos(&format!("NEXT counter '{}' does not match FOR counter '{}'", next_counter, counter)));
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
            // Skip leading newlines (whitespace)
            while matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
            
            // Check if we hit the end keyword
            if matches!(self.current_token(), Token::Keyword(kw) if kw == "UNTIL") {
                break;
            }
            
            body.push(self.parse_statement()?);
            
            // Consume trailing newline (statement terminator)
            if matches!(self.current_token(), Token::Newline) {
                self.advance();
            }
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
                    // Skip leading newlines (whitespace)
                    while matches!(self.current_token(), Token::Newline) {
                        self.advance();
                    }
                    
                    // Check if we hit the end keyword
                    if matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDCASE") {
                        break;
                    }
                    
                    otherwise_body.push(self.parse_statement()?);
                    
                    // Consume trailing newline (statement terminator)
                    if matches!(self.current_token(), Token::Newline) {
                        self.advance();
                    }
                }
                otherwise = Some(otherwise_body);
                break;
            }
            
            let value = self.parse_expression()?;
            
            self.expect(Token::Colon)?;
            
            let mut body = Vec::new();
            
            while !matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDCASE" || kw == "OTHERWISE") {
                // Skip leading newlines (whitespace)
                while matches!(self.current_token(), Token::Newline) {
                    self.advance();
                }
                
                // Check if we hit the end keyword
                if matches!(self.current_token(), Token::Keyword(kw) if kw == "ENDCASE" || kw == "OTHERWISE") {
                    break;
                }
                
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
                
                // Consume trailing newline (statement terminator)
                if matches!(self.current_token(), Token::Newline) {
                    self.advance();
                }
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
        // Parse the left-hand side (lvalue) - can be variable, array access, field access, or pointer dereference
        let name = match self.current_token() {
            Token::Identifier(n) => {
                let var_name = n.clone();
                self.advance();
                var_name
            },
            _ => return Err(self.error_with_pos("Expected identifier")),
        };

        // Check for array access, field access, or pointer dereference
        let mut indices = None;
        
        // Handle array access: arr[i] or arr[i, j]
        if matches!(self.current_token(), Token::LeftBracket) {
            self.advance();
            let mut idxs = Vec::new();
            
            // Parse first index
            idxs.push(self.parse_expression()?);
            
            // Parse additional comma-separated indices
            while matches!(self.current_token(), Token::Comma) {
                self.advance();
                idxs.push(self.parse_expression()?);
            }
            
            self.expect(Token::RightBracket)?;
            indices = Some(idxs);
        }
        
        // Handle field access: obj.field
        // Note: Field access assignments like Student1.LastName <- "Smith" need special handling
        // We'll check if there's a dot after the identifier (or after array access)
        let field_name = if matches!(self.current_token(), Token::Dot) {
            self.advance();
            match self.current_token() {
                Token::Identifier(f) => {
                    let f = f.clone();
                    self.advance();
                    Some(f)
                }
                _ => return Err(self.error_with_pos("Expected field name after dot")),
            }
        } else {
            None
        };
        
        // Handle pointer dereference: ptr^
        let is_pointer_deref = if matches!(self.current_token(), Token::Caret) {
            self.advance();
            true
        } else {
            false
        };

        self.expect(Token::LeftArrow)?;

        let value = self.parse_expression()?;

        // For now, we'll store field access and pointer dereference in the name field
        // This is a simplification - in a full implementation, you might want separate AST nodes
        let final_name = if let Some(field) = field_name {
            format!("{}.{}", name, field)
        } else if is_pointer_deref {
            format!("{}^", name)
        } else {
            name
        };

        Ok(Stmt::Assign {
            name: final_name,
            indices,
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
            _ => Err(self.error_with_pos("Expected identifier")),
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
            _ => return Err(self.error_with_pos("Expected identifier")),
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
    
                // Parse array dimensions - can be [1:5][1:5] or [1:5, 1:5]
                while matches!(self.current_token(), Token::LeftBracket) {
                    self.advance();
                    
                    // Parse first dimension in this bracket
                    loop {
                        let start = self.parse_expression()?;
                        // Accept either colon (:) or comma (,) for array bounds
                        match self.current_token() {
                            Token::Colon | Token::Comma => {
                                self.advance();
                            }
                            _ => return Err(self.error_with_pos("Expected colon or comma between array bounds")),
                        }
                        let end = self.parse_expression()?;
                        dimensions.push((Box::new(start), Box::new(end)));
                        
                        // Check if there's another dimension in the same bracket (comma-separated)
                        match self.current_token() {
                            Token::Comma => {
                                self.advance();
                                continue; // Parse another dimension
                            }
                            Token::RightBracket => {
                                self.advance();
                                break; // Close this bracket
                            }
                            _ => return Err(self.error_with_pos("Expected comma or closing bracket after array dimension")),
                        }
                    }
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
                _ => Err(self.error_with_pos(&format!("Unknown type: {}", kw_str))),
            };
        }
        
        Err(self.error_with_pos("Expected type"))
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
                    _ => return Err(self.error_with_pos(&format!("Expected READ, WRITE, or RANDOM, found {}", kw_str))),
                }
            }
            _ => return Err(self.error_with_pos("Expected READ, WRITE, or RANDOM after FOR")),
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
            _ => return Err(self.error_with_pos("Expected variable name after comma in READFILE")),
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
            _ => return Err(self.error_with_pos("Expected variable name after comma in GETRECORD")),
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
            _ => return Err(self.error_with_pos("Expected variable name after comma in PUTRECORD")),
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
                        _ => return Err(self.error_with_pos("Expected field name after dot")),
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
            _ => Err(self.error_with_pos("Expected primary expression")),
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
        let mut indices = Vec::new();
        
        // Parse first index
        indices.push(self.parse_expression()?);
        
        // Parse additional comma-separated indices
        while matches!(self.current_token(), Token::Comma) {
            self.advance();
            indices.push(self.parse_expression()?);
        }
        
        self.expect(Token::RightBracket)?;
        Ok(Expr::ArrayAccess { array: name, indices })
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
        // Skip leading newlines before parsing expression
        while matches!(self.current_token(), Token::Newline) {
            self.advance();
        }
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
            Err(self.error_with_pos(&format!("Expected {:?}, found {:?}", expected, self.current_token())))
        }
    }
}