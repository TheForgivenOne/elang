use crate::errors::ElangError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Let,
    Const,
    Var,
    Def,
    Class,
    Extends,
    Implements,
    Interface,
    Return,
    If,
    Else,
    For,
    While,
    Loop,
    Repeat,
    Times,
    In,
    From,
    To,
    Match,
    Try,
    Catch,
    Import,
    Export,
    Async,
    Await,
    Self_,
    Pub,
    Pri,
    Pure,
    Alloc,
    Free,
    Ref,
    New,
    End,
    Break,
    Continue,
    And,
    Or,
    Not,
    True,
    False,
    Nothing,
    Int(i64),
    Float(f64),
    Str(String),
    StrInterp(String),
    Ident(String),
    Eq,
    EqEq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    PipeGt,
    FatArrow,
    Arrow,
    Newline,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Colon,
    DotDot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    open_delimiters: i32,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let chars: Vec<char> = source.chars().collect();
        Lexer { chars, pos: 0, line: 1, open_delimiters: 0 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, ElangError> {
        let mut tokens = Vec::new();
        let len = self.chars.len();

        while self.pos < len {
            let c = self.chars[self.pos];

            // Handle \r\n as one newline
            if c == '\r' && self.pos + 1 < len && self.chars[self.pos + 1] == '\n' {
                self.pos += 1;
                if self.open_delimiters == 0 {
                    if tokens.last().map_or(true, |t: &Token| t.kind != TokenKind::Newline) {
                        tokens.push(Token { kind: TokenKind::Newline, line: self.line });
                    }
                }
                self.line += 1;
                self.pos += 1;
                continue;
            }
            if c == '\n' || c == '\r' {
                if self.open_delimiters == 0 {
                    if tokens.last().map_or(true, |t: &Token| t.kind != TokenKind::Newline) {
                        tokens.push(Token { kind: TokenKind::Newline, line: self.line });
                    }
                }
                self.line += 1;
                self.pos += 1;
                continue;
            }

            // Track open/close delimiters (skip newlines inside brackets)
            match c {
                '(' => self.open_delimiters += 1,
                ')' => self.open_delimiters -= 1,
                '[' => self.open_delimiters += 1,
                ']' => self.open_delimiters -= 1,
                '{' => self.open_delimiters += 1,
                '}' => self.open_delimiters -= 1,
                _ => {}
            }

            if c.is_ascii_whitespace() {
                self.pos += 1;
                continue;
            }

            if c == '-' && self.pos + 1 < len && self.chars[self.pos + 1] == '-' {
                self.pos += 2;
                while self.pos < len && self.chars[self.pos] != '\n' {
                    self.pos += 1;
                }
                continue;
            }

            if c.is_ascii_digit() {
                let start = self.pos;
                self.pos += 1;
                while self.pos < len && self.chars[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
                let mut is_float = false;
                if self.pos < len
                    && self.chars[self.pos] == '.'
                    && self.pos + 1 < len
                    && self.chars[self.pos + 1].is_ascii_digit()
                {
                    is_float = true;
                    self.pos += 1;
                    while self.pos < len && self.chars[self.pos].is_ascii_digit() {
                        self.pos += 1;
                    }
                }
                let num_str: String = self.chars[start..self.pos].iter().collect();
                if is_float {
                    let val: f64 = num_str.parse().unwrap();
                    tokens.push(Token { kind: TokenKind::Float(val), line: self.line });
                } else {
                    let val: i64 = num_str.parse().unwrap();
                    tokens.push(Token { kind: TokenKind::Int(val), line: self.line });
                }
                continue;
            }

            if c == '"' {
                self.pos += 1;
                let mut s = String::new();
                let mut has_interp = false;
                while self.pos < len && self.chars[self.pos] != '"' {
                    if self.chars[self.pos] == '\\' && self.pos + 1 < len {
                        self.pos += 1;
                        match self.chars[self.pos] {
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            'r' => s.push('\r'),
                            '\\' => s.push('\\'),
                            '"' => s.push('"'),
                            '{' => s.push('{'),
                            _ => {
                                return Err(ElangError::LexError(format!(
                                    "Invalid escape sequence at line {}",
                                    self.line
                                )))
                            }
                        }
                        self.pos += 1;
                        continue;
                    }
                    if self.chars[self.pos] == '{' {
                        has_interp = true;
                    }
                    if self.chars[self.pos] == '\n' {
                        self.line += 1;
                    }
                    s.push(self.chars[self.pos]);
                    self.pos += 1;
                }
                if self.pos >= len {
                    return Err(ElangError::LexError(format!(
                        "Unterminated string literal at line {}",
                        self.line
                    )));
                }
                self.pos += 1;
                let kind = if has_interp {
                    TokenKind::StrInterp(s)
                } else {
                    TokenKind::Str(s)
                };
                tokens.push(Token { kind, line: self.line });
                continue;
            }

            if c.is_alphabetic() || c == '_' {
                let start = self.pos;
                self.pos += 1;
                while self.pos < len
                    && (self.chars[self.pos].is_alphanumeric() || self.chars[self.pos] == '_')
                {
                    self.pos += 1;
                }
                let word: String = self.chars[start..self.pos].iter().collect();
                let kind = match word.as_str() {
                    "let" => TokenKind::Let,
                    "const" => TokenKind::Const,
                    "var" => TokenKind::Var,
                    "def" => TokenKind::Def,
                    "class" => TokenKind::Class,
                    "extends" => TokenKind::Extends,
                    "implements" => TokenKind::Implements,
                    "interface" => TokenKind::Interface,
                    "return" => TokenKind::Return,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "for" => TokenKind::For,
                    "while" => TokenKind::While,
                    "loop" => TokenKind::Loop,
                    "repeat" => TokenKind::Repeat,
                    "times" => TokenKind::Times,
                    "in" => TokenKind::In,
                    "from" => TokenKind::From,
                    "to" => TokenKind::To,
                    "match" => TokenKind::Match,
                    "try" => TokenKind::Try,
                    "catch" => TokenKind::Catch,
                    "import" => TokenKind::Import,
                    "export" => TokenKind::Export,
                    "async" => TokenKind::Async,
                    "await" => TokenKind::Await,
                    "self" => TokenKind::Self_,
                    "pub" => TokenKind::Pub,
                    "pri" => TokenKind::Pri,
                    "pure" => TokenKind::Pure,
                    "alloc" => TokenKind::Alloc,
                    "free" => TokenKind::Free,
                    "ref" => TokenKind::Ref,
                    "new" => TokenKind::New,
                    "end" => TokenKind::End,
                    "break" => TokenKind::Break,
                    "continue" => TokenKind::Continue,
                    "and" => TokenKind::And,
                    "or" => TokenKind::Or,
                    "not" => TokenKind::Not,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    "nothing" => TokenKind::Nothing,
                    _ => TokenKind::Ident(word),
                };
                tokens.push(Token { kind, line: self.line });
                continue;
            }

            if c == '=' && self.pos + 1 < len && self.chars[self.pos + 1] == '=' {
                tokens.push(Token { kind: TokenKind::EqEq, line: self.line });
                self.pos += 2;
                continue;
            }
            if c == '=' && self.pos + 1 < len && self.chars[self.pos + 1] == '>' {
                tokens.push(Token { kind: TokenKind::FatArrow, line: self.line });
                self.pos += 2;
                continue;
            }
            if c == '=' {
                tokens.push(Token { kind: TokenKind::Eq, line: self.line });
                self.pos += 1;
                continue;
            }

            if c == '!' && self.pos + 1 < len && self.chars[self.pos + 1] == '=' {
                tokens.push(Token { kind: TokenKind::Neq, line: self.line });
                self.pos += 2;
                continue;
            }
            if c == '!' {
                return Err(ElangError::LexError(format!(
                    "Unexpected character '!' at line {}",
                    self.line
                )));
            }

            if c == '<' && self.pos + 1 < len && self.chars[self.pos + 1] == '=' {
                tokens.push(Token { kind: TokenKind::Le, line: self.line });
                self.pos += 2;
                continue;
            }
            if c == '<' {
                tokens.push(Token { kind: TokenKind::Lt, line: self.line });
                self.pos += 1;
                continue;
            }

            if c == '>' && self.pos + 1 < len && self.chars[self.pos + 1] == '=' {
                tokens.push(Token { kind: TokenKind::Ge, line: self.line });
                self.pos += 2;
                continue;
            }
            if c == '>' {
                tokens.push(Token { kind: TokenKind::Gt, line: self.line });
                self.pos += 1;
                continue;
            }

            if c == '|' && self.pos + 1 < len && self.chars[self.pos + 1] == '>' {
                tokens.push(Token { kind: TokenKind::PipeGt, line: self.line });
                self.pos += 2;
                continue;
            }
            if c == '|' {
                return Err(ElangError::LexError(format!(
                    "Unexpected character '|' at line {}",
                    self.line
                )));
            }

            if c == '-' && self.pos + 1 < len && self.chars[self.pos + 1] == '>' {
                tokens.push(Token { kind: TokenKind::Arrow, line: self.line });
                self.pos += 2;
                continue;
            }
            if c == '-' {
                tokens.push(Token { kind: TokenKind::Minus, line: self.line });
                self.pos += 1;
                continue;
            }

            if c == '.' && self.pos + 1 < len && self.chars[self.pos + 1] == '.' {
                tokens.push(Token { kind: TokenKind::DotDot, line: self.line });
                self.pos += 2;
                continue;
            }

            let kind = match c {
                '+' => TokenKind::Plus,
                '*' => TokenKind::Star,
                '/' => TokenKind::Slash,
                '%' => TokenKind::Percent,
                '&' => TokenKind::Amp,
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '[' => TokenKind::LBracket,
                ']' => TokenKind::RBracket,
                '{' => TokenKind::LBrace,
                '}' => TokenKind::RBrace,
                ',' => TokenKind::Comma,
                '.' => TokenKind::Dot,
                ':' => TokenKind::Colon,
                _ => {
                    return Err(ElangError::LexError(format!(
                        "Unexpected character '{}' at line {}",
                        c, self.line
                    )))
                }
            };
            tokens.push(Token { kind, line: self.line });
            self.pos += 1;
        }

        Ok(tokens)
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, ElangError> {
    let mut lexer = Lexer::new(source.to_string());
    lexer.tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tk(kind: TokenKind, line: usize) -> Token {
        Token { kind, line }
    }

    #[test]
    fn test_let_assignment() {
        let mut lexer = Lexer::new("let x = 10".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], tk(TokenKind::Let, 1));
        assert_eq!(tokens[1], tk(TokenKind::Ident("x".to_string()), 1));
        assert_eq!(tokens[2], tk(TokenKind::Eq, 1));
        assert_eq!(tokens[3], tk(TokenKind::Int(10), 1));
    }

    #[test]
    fn test_const_float() {
        let mut lexer = Lexer::new("const PI = 3.14".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], tk(TokenKind::Const, 1));
        assert_eq!(tokens[1], tk(TokenKind::Ident("PI".to_string()), 1));
        assert_eq!(tokens[2], tk(TokenKind::Eq, 1));
        assert_eq!(tokens[3], tk(TokenKind::Float(3.14), 1));
    }

    #[test]
    fn test_comment() {
        let mut lexer = Lexer::new("-- this is a comment".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 0);
    }
}
