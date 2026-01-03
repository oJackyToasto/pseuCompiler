use log::{debug, trace, error};

#[derive(Debug, Clone, PartialEq)]  // Add Debug if not already there
pub enum Token {
    Number(String),
    Identifier(String),
    String(String),
    Keyword(String),
    Char(String),

    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,

    And,
    Or,
    Not,

    LeftArrow,
    RightArrow,

    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Caret,
    Dot,
    
    Newline,
    EOF,
}

impl Token {
    pub fn is_right_paren(&self) -> bool {
        matches!(self, Token::RightParen)
    }
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
pub struct TokenWithPos {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            tokens.push(token.clone());  // Need Clone on Token enum
            
            if token == Token::EOF {
                break;
            }
        }
        tokens
    }
    
    pub fn tokenize_with_pos(&mut self) -> Vec<TokenWithPos> {
        debug!("Starting tokenization");
        let mut tokens = Vec::new();
        loop {
            // Capture position BEFORE calling next_token (which may advance line/column)
            let line = self.line;
            let column = self.column;
            let token = self.next_token();
            
            trace!("Tokenized: {:?} at {}:{}", token, line, column);
            
            tokens.push(TokenWithPos {
                token: token.clone(),
                line,
                column,
            });
            
            if token == Token::EOF {
                debug!("Tokenization complete. Total tokens: {}", tokens.len());
                break;
            }
        }
        tokens
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                ' ' | '\t' => {
                    self.pos += 1;
                    self.column += 1;
                }
                '/' => {
                    // Check if it's a comment: //
                    if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '/' {
                        // Skip until newline
                        while self.pos < self.input.len() {
                            if self.input[self.pos] == '\n' || self.input[self.pos] == '\r' {
                                break;
                            }
                            self.pos += 1;
                        }
                    } else {
                        break; // It's a division operator, not a comment
                    }
                }
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos >= self.input.len() {
            return None;
        }
        Some(self.input[self.pos])
    }

    fn peek_next(&self) -> Option<char> {
        if self.pos + 1 >= self.input.len() {
            return None;
        }
        Some(self.input[self.pos + 1])
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            return None;
        }
        
        let ch = self.input[self.pos];
        self.pos += 1;
        
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else if ch == '\r' {
            if self.peek() == Some('\n') {
                self.pos += 1;
            }
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        
        Some(ch)
    }

    fn read_number(&mut self) -> Token {
        let mut number = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                number.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        Token::Number(number)
    }

    fn read_id_or_kwd(&mut self) -> Token {
        let mut id = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                id.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        match id.as_str() {
            "AND" => Token::And,
            "OR" => Token::Or,
            "NOT" => Token::Not,
            "DECLARE" | "FUNCTION" | "RETURNS" | "FOR" | "WHILE" | "IF" | "TYPE" | "PROCEDURE"
            | "ELSE" | "DO" | "END" | "ENDFUNCTION" | "NEXT" | "ENDIF" | "ENDTYPE" | "ENDPROCEDURE"
            | "ENDWHILE" | "BREAK" | "RETURN" | "INPUT" | "OUTPUT" | "THEN" | "CALL" | "REPEAT"
            | "OPENFILE" | "CLOSEFILE" | "WRITEFILE" | "SEEK" | "GETRECORD" | "PUTRECORD" | "TRUE" | "FALSE"
            | "READFILE" | "MOD" | "LENGTH" | "SET" | "OF" | "TO" | "STEP" | "UNTIL" | "ROUND" | "RAND"
            | "STRING" | "INTEGER" | "REAL" | "CHAR" | "BOOLEAN" | "DATE" | "ARRAY" | "ENDCASE"
            | "UCASE" | "LCASE" | "READ" | "WRITE" | "RANDOM" | "CASE" | "OTHERWISE" | "DIV" | "INT"
            | "SUBSTRING" | "MID" | "RIGHT" | "EOF" => Token::Keyword(id),
            _ => Token::Identifier(id),
        }
    }

    fn read_string(&mut self) -> Token {
        let mut string = String::new();
        self.advance();

        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance();
                    return Token::String(string);
                }
                '\\' => {
                    self.advance();
                    if let Some(escaped) = self.peek() {
                        self.advance();
                        match escaped {
                            'n' => string.push('\n'),
                            'r' => string.push('\r'),
                            't' => string.push('\t'),
                            '"' => string.push('"'),
                            '\\' => string.push('\\'),
                            _ => string.push(escaped),
                        }
                    }
                }
                _ => string.push(self.advance().unwrap()),
            }
        }
        Token::String(string)
    }   

    fn read_char(&mut self) -> Token {
        let mut char = String::new();
        self.advance(); // Skip opening quote
        char.push(self.advance().unwrap()); // Read the character
        if let Some('\'') = self.peek() {
            self.advance(); // Skip closing quote
        }
        Token::Char(char)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        if self.pos >= self.input.len() {
            return Token::EOF;
        }

        let ch = self.peek().unwrap();

        if ch == '\n' {
            self.advance();
            return Token::Newline;
        }

        if ch == '\r' {
            self.advance();
            return Token::Newline;
        }

        if ch == '"' {
            return self.read_string();
        }

        if ch == '\'' {
            return self.read_char();
        }

        if ch.is_ascii_digit() {
            return self.read_number();
        }
        
        if ch.is_ascii_alphabetic() {
            return self.read_id_or_kwd();
        }

        if ch == '-' && self.peek_next() == Some('>') {    
            self.advance();
            self.advance();
            return Token::RightArrow;
        }

        if ch == '<' && self.peek_next() == Some('-') {
            self.advance();
            self.advance();
            return Token::LeftArrow;
        }
        
        if ch == '<' && self.peek_next() == Some('>') {
            self.advance();
            self.advance();
            return Token::NotEquals;
        }

        if ch == '<' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Token::LessThanOrEqual;
        }

        if ch == '>' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Token::GreaterThanOrEqual;
        }

        match ch {
            '+' => { self.advance(); Token::Plus }
            '-' => { self.advance(); Token::Minus }
            '*' => { self.advance(); Token::Multiply }
            '/' => { self.advance(); Token::Divide }
            '=' => { self.advance(); Token::Equals }
            '<' => { self.advance(); Token::LessThan }
            '>' => { self.advance(); Token::GreaterThan }
            '(' => { self.advance(); Token::LeftParen }
            ')' => { self.advance(); Token::RightParen }
            '[' => { self.advance(); Token::LeftBracket }
            ']' => { self.advance(); Token::RightBracket }
            ',' => { self.advance(); Token::Comma }
            ':' => { self.advance(); Token::Colon }
            '^' => { self.advance(); Token::Caret }
            '.' => { self.advance(); Token::Dot }
            _ => {
                let ch = self.advance().unwrap();
                let msg = format!("Unexpected character: '{}' at line {}:{}", ch, self.line, self.column);
                error!("{}", msg);
                panic!("{}", msg);
            }
        }
    }
}