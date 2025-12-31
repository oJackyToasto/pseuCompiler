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
    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.input[self.position] {
                ' ' | '\t' => {
                    self.pos += 1;
                    self.column += 1;
                }
                _ => break,
            }
        }
    }

    pub fn next_token(&mut self) {
        self.skip_whitespace();
        
        if self.pos >= self.input.len() {
            return Token::EOF;
        }

        let ch = self.input[self.pos];

        if ch == '\n' {
            self.pos += 1;
            self.line += 1;
            self.column += 1;
            return Token::NewLine;
        }

        else if ch == '\r' {
            self.pos += 1;
        }
    }
}