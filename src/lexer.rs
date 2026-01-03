enum Token {
    Number(String),
    Identifier(String),
    String(String),
    Keyword(String),

    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,

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

    Newline,
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
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

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                ' ' | '\t' => {
                    self.pos += 1;
                    self.column += 1;
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
            "DECLARE" | "FUNCTION" | "RETURNS" | "FOR" | "WHILE" | "IF" 
            | "ELSE" | "DO" | "END" | "ENDFUNCTION" | "NEXT" | "ENDIF" 
            | "ENDWHILE" | "BREAK" | "RETURN" | "INPUT" | "OUTPUT" 
            | "OPENFILE" | "CLOSEFILE" | "READFILE" | "MOD" | "LENGTH" 
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

        if ch.is_ascii_digit() {
            return self.read_number();
        }
        
        if ch.is_ascii_alphabetic() {
            self.read_id_or_kwd();
        }
        if ch == '-' {
            if self.peek_next() == Some('<') {
                self.advance(); // consume '-'
                self.advance(); // consume '<'
                return Token::LeftArrow;
            }
            if self.peek_next() == Some('>') {
                self.advance();
                self.advance();
                return Token::RightArrow;
            }
        }
    
        if ch == '=' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Token::Equals;
        }
    
        if ch == '!' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Token::NotEquals;
        }
    
        if ch == '<' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Token::LessThan;
        }
    
        if ch == '>' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Token::GreaterThan;
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
            _ => {
                let ch = self.advance().unwrap();
                panic!("Unexpected character: '{}' at line {}:{}", ch, self.line, self.column);
            }
        }
    }
}