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

    fn assert_tokens(source: &str, expected: Vec<TokenKind>) {
        let tokens = tokenize(source).unwrap();
        let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(kinds.len(), expected.len(), "token count mismatch for '{:?}'.\ngot:      {:?}\nexpected: {:?}", source, kinds, expected);
        for (i, (got, exp)) in kinds.iter().zip(expected.iter()).enumerate() {
            if *got != exp {
                panic!("token {} mismatch for '{:?}':\n  got:      {:?}\n  expected: {:?}", i, source, got, exp);
            }
        }
    }

    fn assert_lex_error(source: &str) {
        assert!(tokenize(source).is_err(), "expected lex error for '{}'", source);
    }

    // === Keywords ===

    #[test]
    fn test_all_keywords() {
        let keywords = [
            ("let", TokenKind::Let), ("const", TokenKind::Const), ("var", TokenKind::Var),
            ("def", TokenKind::Def), ("class", TokenKind::Class), ("extends", TokenKind::Extends),
            ("implements", TokenKind::Implements), ("interface", TokenKind::Interface),
            ("return", TokenKind::Return), ("if", TokenKind::If), ("else", TokenKind::Else),
            ("for", TokenKind::For), ("while", TokenKind::While), ("loop", TokenKind::Loop),
            ("repeat", TokenKind::Repeat), ("times", TokenKind::Times), ("in", TokenKind::In),
            ("from", TokenKind::From), ("to", TokenKind::To), ("match", TokenKind::Match),
            ("try", TokenKind::Try), ("catch", TokenKind::Catch), ("import", TokenKind::Import),
            ("export", TokenKind::Export), ("async", TokenKind::Async), ("await", TokenKind::Await),
            ("self", TokenKind::Self_), ("pub", TokenKind::Pub), ("pri", TokenKind::Pri),
            ("pure", TokenKind::Pure), ("alloc", TokenKind::Alloc), ("free", TokenKind::Free),
            ("ref", TokenKind::Ref), ("new", TokenKind::New), ("end", TokenKind::End),
            ("break", TokenKind::Break), ("continue", TokenKind::Continue),
            ("and", TokenKind::And), ("or", TokenKind::Or), ("not", TokenKind::Not),
            ("true", TokenKind::True), ("false", TokenKind::False), ("nothing", TokenKind::Nothing),
        ];
        for (source, expected) in keywords {
            let tokens = tokenize(source).unwrap();
            assert_eq!(tokens.len(), 1, "keyword '{}' should produce 1 token", source);
            assert_eq!(tokens[0].kind, expected, "keyword '{}' mismatch", source);
        }
    }

    #[test]
    fn test_identifiers() {
        let cases = [("x", "x"), ("myVar", "myVar"), ("_private", "_private"), ("a_b_c", "a_b_c"), ("x1", "x1")];
        for (source, expected) in cases {
            let tokens = tokenize(source).unwrap();
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].kind, TokenKind::Ident(expected.to_string()));
        }
    }

    // === Literals ===

    #[test]
    fn test_int_literals() {
        let cases = [("0", 0), ("1", 1), ("42", 42), ("999999", 999999)];
        for (source, expected) in cases {
            let tokens = tokenize(source).unwrap();
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].kind, TokenKind::Int(expected));
        }
    }

    #[test]
    fn test_float_literals() {
        let cases = [("0.0", 0.0), ("3.14", 3.14), ("0.5", 0.5), ("100.0", 100.0)];
        for (source, expected) in cases {
            let tokens = tokenize(source).unwrap();
            assert_eq!(tokens.len(), 1);
            match &tokens[0].kind {
                TokenKind::Float(val) => assert!((val - expected).abs() < 1e-10),
                _ => panic!("Expected Float for '{}'", source),
            }
        }
    }

    #[test]
    fn test_string_literals() {
        let tokens = tokenize(r#""hello""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Str("hello".to_string()));
    }

    #[test]
    fn test_string_literal_empty() {
        let tokens = tokenize(r#""""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Str("".to_string()));
    }

    #[test]
    fn test_string_escape_sequences() {
        let tokens = tokenize(r#""line1\nline2\tindented\\backslash\"quote""#).unwrap();
        assert_eq!(tokens.len(), 1);
        match &tokens[0].kind {
            TokenKind::Str(s) => {
                assert_eq!(s, "line1\nline2\tindented\\backslash\"quote");
            }
            _ => panic!("Expected Str"),
        }
    }

    #[test]
    fn test_string_interpolation_detected() {
        let tokens = tokenize(r#""hello {name} world""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind, TokenKind::StrInterp(_)));
    }

    #[test]
    fn test_string_interpolation_multi_brace() {
        let tokens = tokenize(r#""{a} + {b} = {c}""#).unwrap();
        assert_eq!(tokens.len(), 1);
        match &tokens[0].kind {
            TokenKind::StrInterp(s) => assert_eq!(s, "{a} + {b} = {c}"),
            _ => panic!("Expected StrInterp, got {:?}", tokens[0].kind),
        }
    }

    #[test]
    fn test_string_interpolation_empty_braces() {
        let tokens = tokenize(r#""hello {} world""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind, TokenKind::StrInterp(_)));
    }

    // === Operators ===

    #[test]
    fn test_single_char_operators() {
        let cases = [
            ("+", TokenKind::Plus), ("-", TokenKind::Minus), ("*", TokenKind::Star),
            ("/", TokenKind::Slash), ("%", TokenKind::Percent), ("<", TokenKind::Lt),
            (">", TokenKind::Gt), ("=", TokenKind::Eq), (".", TokenKind::Dot),
            (",", TokenKind::Comma), (":", TokenKind::Colon),
            ("(", TokenKind::LParen), (")", TokenKind::RParen),
            ("[", TokenKind::LBracket), ("]", TokenKind::RBracket),
            ("{", TokenKind::LBrace), ("}", TokenKind::RBrace),
            ("&", TokenKind::Amp),
        ];
        for (source, expected) in cases {
            let tokens = tokenize(source).unwrap();
            assert_eq!(tokens.len(), 1, "operator '{}' should produce 1 token", source);
            assert_eq!(tokens[0].kind, expected, "operator '{}' mismatch", source);
        }
    }

    #[test]
    fn test_multi_char_operators() {
        let cases = [
            ("==", TokenKind::EqEq), ("!=", TokenKind::Neq), ("<=", TokenKind::Le),
            (">=", TokenKind::Ge), ("|>", TokenKind::PipeGt), ("=>", TokenKind::FatArrow),
            ("->", TokenKind::Arrow), ("..", TokenKind::DotDot),
        ];
        for (source, expected) in cases {
            let tokens = tokenize(source).unwrap();
            assert_eq!(tokens.len(), 1, "operator '{}' should produce 1 token", source);
            assert_eq!(tokens[0].kind, expected, "operator '{}' mismatch", source);
        }
    }

    // === Comments ===

    #[test]
    fn test_comment_line_start() {
        let tokens = tokenize("-- this is a comment").unwrap();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_comment_after_code() {
        let tokens = tokenize("let x = 1 -- inline comment").unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[3].kind, TokenKind::Int(1));
    }

    #[test]
    fn test_multiple_comments() {
        let tokens = tokenize("-- first\n-- second\nlet x = 1\n-- third").unwrap();
        assert_eq!(tokens.len(), 6); // newline, let, x, =, 1, newline
        // The key is the let x = 1 is lexed correctly
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Let));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Int(1)));
    }

    // === Statement / Punctuation ===

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

    // === Newlines ===

    #[test]
    fn test_newline_simple() {
        let tokens = tokenize("let x = 1\nprint x").unwrap();
        let newline_count = tokens.iter().filter(|t| t.kind == TokenKind::Newline).count();
        assert_eq!(newline_count, 1);
    }

    #[test]
    fn test_newline_suppressed_in_parens() {
        let tokens = tokenize("(1 +\n 2)").unwrap();
        let newline_count = tokens.iter().filter(|t| t.kind == TokenKind::Newline).count();
        assert_eq!(newline_count, 0);
    }

    #[test]
    fn test_newline_suppressed_in_brackets() {
        let tokens = tokenize("[1,\n 2]").unwrap();
        let newline_count = tokens.iter().filter(|t| t.kind == TokenKind::Newline).count();
        assert_eq!(newline_count, 0);
    }

    #[test]
    fn test_newline_suppressed_in_braces() {
        let tokens = tokenize("{a:\n 1}").unwrap();
        let newline_count = tokens.iter().filter(|t| t.kind == TokenKind::Newline).count();
        assert_eq!(newline_count, 0);
    }

    // === Line tracking ===

    #[test]
    fn test_line_numbers() {
        let tokens = tokenize("let a = 1\nlet b = 2\nlet c = 3").unwrap();
        let let_tokens: Vec<&Token> = tokens.iter().filter(|t| t.kind == TokenKind::Let).collect();
        assert_eq!(let_tokens.len(), 3);
        assert_eq!(let_tokens[0].line, 1);
        assert_eq!(let_tokens[1].line, 2);
        assert_eq!(let_tokens[2].line, 3);
    }

    // === Empty and whitespace ===

    #[test]
    fn test_empty_source() {
        let tokens = tokenize("").unwrap();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_whitespace_only() {
        let tokens = tokenize("   \t  \n  ").unwrap();
        // Whitespace-only produces no meaningful tokens
        // The newline will produce a Newline token though
        assert!(tokens.iter().all(|t| t.kind == TokenKind::Newline));
    }

    // === Multi-element programs ===

    #[test]
    fn test_full_expression() {
        let tokens = tokenize("let result = 10 + 20 * 3").unwrap();
        assert_eq!(tokens.len(), 8);
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Ident("result".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Eq);
        assert_eq!(tokens[3].kind, TokenKind::Int(10));
        assert_eq!(tokens[4].kind, TokenKind::Plus);
        assert_eq!(tokens[5].kind, TokenKind::Int(20));
        assert_eq!(tokens[6].kind, TokenKind::Star);
        assert_eq!(tokens[7].kind, TokenKind::Int(3));
    }

    #[test]
    fn test_list_literal_tokens() {
        let tokens = tokenize("[1, 2, 3]").unwrap();
        let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(kinds, vec![
            &TokenKind::LBracket,
            &TokenKind::Int(1),
            &TokenKind::Comma,
            &TokenKind::Int(2),
            &TokenKind::Comma,
            &TokenKind::Int(3),
            &TokenKind::RBracket,
        ]);
    }

    // === Error cases ===

    #[test]
    fn test_error_unexpected_char_at() {
        assert_lex_error("@");
    }

    #[test]
    fn test_error_unexpected_char_dollar() {
        assert_lex_error("$");
    }

    #[test]
    fn test_error_unterminated_string() {
        assert_lex_error(r#""unterminated"#);
    }

    #[test]
    fn test_error_invalid_escape() {
        assert_lex_error(r#""\z""#);
    }

    #[test]
    fn test_error_bang_operator() {
        assert_lex_error("!x");
    }

    #[test]
    fn test_error_pipe_alone() {
        assert_lex_error("a | b");
    }

    #[test]
    fn test_error_question_mark() {
        assert_lex_error("a ? b");
    }
}
