// Parser: consumes tokens and produces an AST

#![allow(dead_code)]

use crate::ast::*;
use crate::errors::ElangError;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        tokens.push(Token { kind: TokenKind::Nothing, line: 0 });
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ElangError> {
        self.parse_program()
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len() - 1
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> Result<&Token, ElangError> {
        if self.at_end() {
            return Err(ElangError::ParseError("Unexpected end of input".into()));
        }
        let t = &self.tokens[self.pos];
        self.pos += 1;
        Ok(t)
    }

    fn check(&self, kind: &TokenKind) -> bool {
        !self.at_end() && &self.peek().kind == kind
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.check(&kind) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ElangError> {
        if self.check(&kind) {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Ok(token)
        } else {
            let token = self.peek();
            Err(ElangError::ParseError(format!(
                "expected '{:?}' but found '{:?}' at line {}",
                kind,
                token.kind,
                token.line
            )))
        }
    }

    fn check_ident(&self) -> bool {
        matches!(&self.peek().kind, TokenKind::Ident(_))
    }

    fn check_ident_str(&self, s: &str) -> bool {
        match &self.peek().kind {
            TokenKind::Ident(name) => name == s,
            _ => false,
        }
    }

    fn check_next(&self, kind: TokenKind) -> bool {
        self.pos + 1 < self.tokens.len() - 1 && self.tokens[self.pos + 1].kind == kind
    }

    fn expect_ident(&mut self) -> Result<String, ElangError> {
        match &self.peek().kind {
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.pos += 1;
                Ok(name)
            }
            _ => {
                let token = self.peek();
                Err(ElangError::ParseError(format!(
                    "Expected identifier at line {}, found {:?}",
                    token.line,
                    token.kind
                )))
            }
        }
    }

    fn skip_newlines(&mut self) {
        while !self.at_end() && self.check(&TokenKind::Newline) {
            self.pos += 1;
        }
    }

    fn parse_program(&mut self) -> Result<Program, ElangError> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.at_end() {
            stmts.push(self.parse_statement()?);
            self.skip_newlines();
        }
        Ok(stmts)
    }

    fn parse_block_stopping_at(&mut self, terminators: &[TokenKind]) -> Result<Vec<Statement>, ElangError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            if terminators.iter().any(|t| self.check(t)) {
                return Ok(stmts);
            }
            if self.at_end() {
                return Err(ElangError::ParseError(
                    "unexpected end of file — did you forget 'end'?".to_string(),
                ));
            }
            stmts.push(self.parse_statement()?);
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, ElangError> {
        self.skip_newlines();
        let stmt_line = self.peek().line;
        if (self.check_ident() || self.check(&TokenKind::Self_)) && self.check_next(TokenKind::Dot) {
            if let Some(next) = self.tokens.get(self.pos + 2) {
                if matches!(next.kind, TokenKind::Ident(_)) {
                    if let Some(eq) = self.tokens.get(self.pos + 3) {
                        if eq.kind == TokenKind::Eq {
                            let object = if self.check(&TokenKind::Self_) {
                                self.advance()?;
                                "self".to_string()
                            } else {
                                self.expect_ident()?
                            };
                            self.advance()?;
                            let field = self.expect_ident()?;
                            self.advance()?;
                            let value = self.parse_expr()?;
                            return Ok(Statement::FieldAssign { object, field, value, line: stmt_line });
                        }
                    }
                }
            }
        }

        if self.check_ident() && self.check_next(TokenKind::Eq) {
            let name = self.expect_ident()?;
            self.advance()?;
            let value = self.parse_expr()?;
            return Ok(Statement::Assign { name, value, line: stmt_line });
        }

        if self.check_ident_str("print") {
            self.advance()?;
            let value = self.parse_expr()?;
            return Ok(Statement::Print { value, line: stmt_line });
        }

        let tok = self.peek().kind.clone();
        match tok {
            TokenKind::Let => self.parse_let_decl(),
            TokenKind::Const => self.parse_const_decl(),
            TokenKind::Var => self.parse_var_decl(),
            TokenKind::Def => self.parse_fn_def(),
            TokenKind::Class => self.parse_class_def(),
            TokenKind::Return => {
                self.advance()?;
                let value = self.parse_expr()?;
                Ok(Statement::Return { value, line: stmt_line })
            }
            TokenKind::If => self.parse_if(),
            TokenKind::For => self.parse_for(),
            TokenKind::While => {
                self.advance()?;
                let kind = LoopKind::While(self.parse_expr()?);
                self.eat(TokenKind::Colon);
                let body = self.parse_block_stopping_at(&[TokenKind::End])?;
                self.expect(TokenKind::End)?;
                Ok(Statement::Loop { kind, body, line: stmt_line })
            }
            TokenKind::Loop => {
                self.advance()?;
                self.eat(TokenKind::Colon);
                let body = self.parse_block_stopping_at(&[TokenKind::End])?;
                self.expect(TokenKind::End)?;
                Ok(Statement::Loop { kind: LoopKind::Forever, body, line: stmt_line })
            }
            TokenKind::Repeat => self.parse_repeat(),
            TokenKind::Match => self.parse_match(),
            TokenKind::Try => self.parse_try(),
            TokenKind::Import => {
                self.advance()?;
                let module = self.expect_ident()?;
                Ok(Statement::Import { module, line: stmt_line })
            }
            TokenKind::Export => {
                self.advance()?;
                let stmt = self.parse_statement()?;
                Ok(Statement::Export { stmt: Box::new(stmt), line: stmt_line })
            }
            TokenKind::Async => {
                self.advance()?;
                if self.check(&TokenKind::Def) {
                    let mut stmt = self.parse_fn_def()?;
                    if let Statement::FnDef { ref mut is_async, .. } = stmt {
                        *is_async = true;
                    }
                    Ok(stmt)
                } else {
                    let expr = self.parse_expr()?;
                    Ok(Statement::ExprStmt { expr, line: stmt_line })
                }
            }
            TokenKind::Pure => {
                self.advance()?;
                if self.check(&TokenKind::Def) {
                    let mut stmt = self.parse_fn_def()?;
                    if let Statement::FnDef { ref mut is_pure, .. } = stmt {
                        *is_pure = true;
                    }
                    Ok(stmt)
                } else {
                    Err(ElangError::ParseError(format!(
                        "Expected 'def' after 'pure' at line {}",
                        self.peek().line
                    )))
                }
            }
            TokenKind::Break => {
                self.advance()?;
                Ok(Statement::Break { line: stmt_line })
            }
            TokenKind::Continue => {
                self.advance()?;
                Ok(Statement::Continue { line: stmt_line })
            }
            TokenKind::Pub | TokenKind::Pri => self.parse_class_member(),
            _ => {
                let expr = self.parse_expr()?;
                Ok(Statement::ExprStmt { expr, line: stmt_line })
            }
        }
    }

    fn parse_let_decl(&mut self) -> Result<Statement, ElangError> {
        let line = self.peek().line;
        self.advance()?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Eq)?;
        let value = self.parse_expr()?;
        Ok(Statement::LetDecl { name, value, line })
    }

    fn parse_const_decl(&mut self) -> Result<Statement, ElangError> {
        let line = self.peek().line;
        self.advance()?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Eq)?;
        let value = self.parse_expr()?;
        Ok(Statement::ConstDecl { name, value, line })
    }

    fn parse_var_decl(&mut self) -> Result<Statement, ElangError> {
        let line = self.peek().line;
        self.advance()?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Eq)?;
        let value = self.parse_expr()?;
        Ok(Statement::VarDecl { name, value, line })
    }

    fn parse_fn_def(&mut self) -> Result<Statement, ElangError> {
        let line = self.peek().line;
        self.advance()?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                params.push(self.expect_ident()?);
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen)?;
        self.eat(TokenKind::Colon);
        let body = self.parse_block_stopping_at(&[TokenKind::End])?;
        self.expect(TokenKind::End)?;
        Ok(Statement::FnDef {
            name,
            params,
            body,
            is_async: false,
            is_pure: false,
            visibility: Visibility::Default,
            line,
        })
    }

    fn parse_class_def(&mut self) -> Result<Statement, ElangError> {
        let line = self.peek().line;
        self.advance()?;
        let name = self.expect_ident()?;
        let parent = if self.eat(TokenKind::Extends) {
            Some(self.expect_ident()?)
        } else {
            None
        };
        self.eat(TokenKind::Colon);
        let body = self.parse_class_body()?;
        self.expect(TokenKind::End)?;
        Ok(Statement::ClassDef { name, parent, body, line })
    }

    fn parse_class_body(&mut self) -> Result<Vec<Statement>, ElangError> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.at_end() && !self.check(&TokenKind::End) {
            stmts.push(self.parse_class_member()?);
            self.skip_newlines();
        }
        if self.at_end() {
            return Err(ElangError::ParseError(
                "unexpected end of file — did you forget 'end'?".to_string(),
            ));
        }
        Ok(stmts)
    }

    fn parse_class_member(&mut self) -> Result<Statement, ElangError> {
        self.skip_newlines();
        let line = self.peek().line;
        let vis = if self.eat(TokenKind::Pub) {
            Visibility::Pub
        } else if self.eat(TokenKind::Pri) {
            Visibility::Pri
        } else {
            Visibility::Default
        };

        if self.check(&TokenKind::Def) {
            let mut stmt = self.parse_fn_def()?;
            if let Statement::FnDef { ref mut visibility, .. } = stmt {
                *visibility = vis;
            }
            return Ok(stmt);
        }

        if self.check_ident() && self.check_next(TokenKind::Eq) {
            let name = self.expect_ident()?;
            self.advance()?;
            let value = self.parse_expr()?;
            return Ok(Statement::Field { name, value, visibility: vis, line });
        }

        Err(ElangError::ParseError(format!(
            "Expected class member (field or method) at line {}",
            self.peek().line
        )))
    }

    fn parse_if(&mut self) -> Result<Statement, ElangError> {
        let line = self.peek().line;
        self.advance()?;
        let condition = self.parse_expr()?;
        self.eat(TokenKind::Colon);
        let then_block = self.parse_block_stopping_at(&[TokenKind::End, TokenKind::Else])?;
        let else_block = if self.eat(TokenKind::Else) {
            self.eat(TokenKind::Colon);
            Some(self.parse_block_stopping_at(&[TokenKind::End])?)
        } else {
            None
        };
        self.expect(TokenKind::End)?;
        Ok(Statement::If { condition, then_block, else_block, line })
    }

    fn parse_for(&mut self) -> Result<Statement, ElangError> {
        let line = self.peek().line;
        self.advance()?;
        let var = self.expect_ident()?;
        self.expect(TokenKind::In)?;
        let iterable = self.parse_expr()?;
        self.eat(TokenKind::Colon);
        let body = self.parse_block_stopping_at(&[TokenKind::End])?;
        self.expect(TokenKind::End)?;
        Ok(Statement::ForIn { var, iterable, body, line })
    }

    fn parse_repeat(&mut self) -> Result<Statement, ElangError> {
        let line = self.peek().line;
        self.advance()?;
        if self.check_ident() && self.check_next(TokenKind::From) {
            let var = self.expect_ident()?;
            self.advance()?;
            let from = self.parse_expr()?;
            self.expect(TokenKind::To)?;
            let to = self.parse_expr()?;
            self.eat(TokenKind::Colon);
            let body = self.parse_block_stopping_at(&[TokenKind::End])?;
            self.expect(TokenKind::End)?;
            Ok(Statement::Loop {
                kind: LoopKind::RepeatRange { var, from, to },
                body,
                line,
            })
        } else {
            let count = self.parse_expr()?;
            self.expect(TokenKind::Times)?;
            self.eat(TokenKind::Colon);
            let body = self.parse_block_stopping_at(&[TokenKind::End])?;
            self.expect(TokenKind::End)?;
            Ok(Statement::Loop {
                kind: LoopKind::RepeatN(count),
                body,
                line,
            })
        }
    }

    fn is_match_pattern_start(&self) -> bool {
        let kind = &self.peek().kind;
        matches!(kind, TokenKind::Int(_) | TokenKind::Float(_) | TokenKind::Str(_)
            | TokenKind::True | TokenKind::False | TokenKind::Nothing | TokenKind::Not)
            || self.check_ident_str("_") || self.check_ident_str("is")
    }

    fn parse_match(&mut self) -> Result<Statement, ElangError> {
        let line = self.peek().line;
        self.advance()?;
        let value = self.parse_expr()?;
        self.eat(TokenKind::Colon);
        self.skip_newlines();
        let mut arms = Vec::new();
        while !self.at_end() && !self.check(&TokenKind::End) {
            let pattern = self.parse_match_pattern()?;
            self.expect(TokenKind::Colon)?;
            // Parse arm body: statements until next pattern or End
            let mut body = Vec::new();
            loop {
                self.skip_newlines();
                if self.check(&TokenKind::End) {
                    break;
                }
                if self.is_match_pattern_start() {
                    break;
                }
                body.push(self.parse_statement()?);
            }
            self.skip_newlines();
            arms.push(MatchArm { pattern, body });
        }
        if self.at_end() {
            return Err(ElangError::ParseError(
                "unexpected end of file — did you forget 'end'?".to_string(),
            ));
        }
        self.expect(TokenKind::End)?;
        Ok(Statement::Match { value, arms, line })
    }

    fn parse_match_pattern(&mut self) -> Result<MatchPattern, ElangError> {
        // _ wildcard
        if self.check_ident_str("_") {
            self.advance()?;
            return Ok(MatchPattern::Wildcard);
        }
        if self.eat(TokenKind::Not) {
            return Ok(MatchPattern::Wildcard);
        }
        match &self.peek().kind {
            TokenKind::True | TokenKind::False | TokenKind::Nothing
            | TokenKind::Int(_) | TokenKind::Float(_) | TokenKind::Str(_) => {
                let expr = self.parse_expr()?;
                Ok(MatchPattern::Literal(expr))
            }
            TokenKind::Ident(name) if name == "is" => {
                self.advance()?;
                let type_name = self.expect_ident()?;
                Ok(MatchPattern::IsType(type_name))
            }
            _ => {
                let expr = self.parse_expr()?;
                Ok(MatchPattern::Literal(expr))
            }
        }
    }

    fn parse_try(&mut self) -> Result<Statement, ElangError> {
        let line = self.peek().line;
        self.advance()?;
        self.eat(TokenKind::Colon);
        let body = self.parse_block_stopping_at(&[TokenKind::End, TokenKind::Catch])?;
        let mut catches = Vec::new();
        while self.eat(TokenKind::Catch) {
            let error_type = if !self.check_ident() && self.check_ident() {
                None
            } else if self.check_ident() && self.tokens.get(self.pos + 1).map_or(false, |t| t.kind != TokenKind::Colon) {
                let type_name = self.expect_ident()?;
                Some(type_name)
            } else {
                None
            };
            let var = self.expect_ident()?;
            self.eat(TokenKind::Colon);
            let catch_body = self.parse_block_stopping_at(&[TokenKind::End, TokenKind::Catch])?;
            catches.push(CatchClause { error_type, var, body: catch_body });
        }
        self.expect(TokenKind::End)?;
        Ok(Statement::Try { body, catches, line })
    }

    fn parse_expr(&mut self) -> Result<Expr, ElangError> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr, ElangError> {
        let mut lhs = self.parse_prefix_expr()?;

        loop {
            if self.at_end() {
                break;
            }
            let kind = self.peek().kind.clone();
            let line = self.peek().line;

            match kind {
                TokenKind::LParen => {
                    self.advance()?;
                    let args = self.parse_call_args()?;
                    lhs = Expr::Call { callee: Box::new(lhs), args, line };
                    continue;
                }
                TokenKind::LBracket => {
                    self.advance()?;
                    let index = self.parse_expr()?;
                    self.expect(TokenKind::RBracket)?;
                    lhs = Expr::Index { object: Box::new(lhs), index: Box::new(index), line };
                    continue;
                }
                TokenKind::Dot => {
                    self.advance()?;
                    let field = self.expect_ident()?;
                    lhs = Expr::Field { object: Box::new(lhs), field, line };
                    continue;
                }
                _ => {}
            }

            if kind == TokenKind::FatArrow {
                if let Expr::Call { callee, args, .. } = &lhs {
                    if let Expr::Ident { name, .. } = callee.as_ref() {
                        if name == "fn" {
                            let params: Vec<String> = args.iter().filter_map(|a| {
                                if let Expr::Ident { name: p, .. } = a { Some(p.clone()) } else { None }
                            }).collect();
                            let line = self.peek().line;
                            self.advance()?;
                            let body = self.parse_expr()?;
                            lhs = Expr::Lambda { params, body: Box::new(body), line };
                            continue;
                        }
                    }
                }
                break;
            }

            if kind == TokenKind::PipeGt {
                let bp = 0;
                if bp < min_bp {
                    break;
                }
                self.advance()?;
                let rhs = self.parse_expr_bp(bp)?;
                lhs = Expr::Pipe { left: Box::new(lhs), right: Box::new(rhs), line };
                continue;
            }

            if let Some((bp, op)) = Self::infix_bp(&kind) {
                if bp < min_bp {
                    break;
                }
                self.advance()?;
                let rhs = self.parse_expr_bp(bp + 1)?;
                lhs = Expr::BinOp { left: Box::new(lhs), op, right: Box::new(rhs), line };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    fn parse_prefix_expr(&mut self) -> Result<Expr, ElangError> {
        let tok = self.advance()?;
        let line = tok.line;
        match tok.kind.clone() {
            TokenKind::Int(n) => Ok(Expr::Int { value: n, line }),
            TokenKind::Float(n) => Ok(Expr::Float { value: n, line }),
            TokenKind::Str(s) => Ok(Expr::Str { value: s, line }),
            TokenKind::StrInterp(s) => Ok(Expr::StrInterp { value: s, line }),
            TokenKind::True => Ok(Expr::Bool { value: true, line }),
            TokenKind::False => Ok(Expr::Bool { value: false, line }),
            TokenKind::Nothing => Ok(Expr::Nothing { line }),
            TokenKind::Ident(name) => Ok(Expr::Ident { name, line }),
            TokenKind::Self_ => Ok(Expr::Ident { name: "self".to_string(), line }),
            TokenKind::Minus => {
                let expr = self.parse_expr_bp(7)?;
                Ok(Expr::UnaryOp { op: UnaryOpKind::Neg, expr: Box::new(expr), line })
            }
            TokenKind::Not => {
                let expr = self.parse_expr_bp(7)?;
                Ok(Expr::UnaryOp { op: UnaryOpKind::Not, expr: Box::new(expr), line })
            }
            TokenKind::Await => {
                let expr = self.parse_expr_bp(7)?;
                Ok(Expr::Await { expr: Box::new(expr), line })
            }
            TokenKind::LParen => {
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBracket => {
                let mut items = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        if self.at_end() {
                            return Err(ElangError::ParseError(
                                "unexpected end of file in list literal".to_string(),
                            ));
                        }
                        items.push(self.parse_expr()?);
                        if !self.eat(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Expr::List { items, line })
            }
            TokenKind::LBrace => {
                let mut pairs = Vec::new();
                if !self.check(&TokenKind::RBrace) {
                    loop {
                        if self.at_end() {
                            return Err(ElangError::ParseError(
                                "unexpected end of file in map literal".to_string(),
                            ));
                        }
                        let key = self.expect_ident()?;
                        self.expect(TokenKind::Colon)?;
                        let value = self.parse_expr()?;
                        pairs.push((key, value));
                        if !self.eat(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Expr::Map { pairs, line })
            }
            _ => Err(ElangError::ParseError(format!(
                "Expected expression at line {}",
                self.peek().line
            ))),
        }
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, ElangError> {
        let mut args = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                if self.at_end() {
                    return Err(ElangError::ParseError(
                        "unexpected end of file in argument list".to_string(),
                    ));
                }
                args.push(self.parse_expr()?);
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(args)
    }

    fn infix_bp(kind: &TokenKind) -> Option<(u8, BinOpKind)> {
        match kind {
            TokenKind::Or => Some((1, BinOpKind::Or)),
            TokenKind::And => Some((2, BinOpKind::And)),
            TokenKind::EqEq => Some((3, BinOpKind::Eq)),
            TokenKind::Neq => Some((3, BinOpKind::NotEq)),
            TokenKind::Lt => Some((4, BinOpKind::Lt)),
            TokenKind::Gt => Some((4, BinOpKind::Gt)),
            TokenKind::Le => Some((4, BinOpKind::LtEq)),
            TokenKind::Ge => Some((4, BinOpKind::GtEq)),
            TokenKind::Plus => Some((5, BinOpKind::Add)),
            TokenKind::Minus => Some((5, BinOpKind::Sub)),
            TokenKind::Star => Some((6, BinOpKind::Mul)),
            TokenKind::Slash => Some((6, BinOpKind::Div)),
            TokenKind::Percent => Some((6, BinOpKind::Mod)),
            _ => None,
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, ElangError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn parse_source(source: &str) -> Result<Program, ElangError> {
        let tokens = tokenize(source)?;
        parse(tokens)
    }

    #[test]
    fn test_let_with_binop() -> Result<(), ElangError> {
        let program = parse_source("let x = 10 + 5")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::LetDecl { name, value, .. } => {
                assert_eq!(name, "x");
                match value {
                    Expr::BinOp { left, op, right, .. } => {
                        assert!(matches!(op, BinOpKind::Add));
                        assert!(matches!(**left, Expr::Int { value: 10, .. }));
                        assert!(matches!(**right, Expr::Int { value: 5, .. }));
                    }
                    _ => panic!("Expected BinOp"),
                }
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_if_statement() -> Result<(), ElangError> {
        let program = parse_source("if x > 0\n  print x\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::If { condition, then_block, else_block, .. } => {
                assert!(else_block.is_none());
                assert_eq!(then_block.len(), 1);
                assert!(matches!(&then_block[0], Statement::Print { .. }));
                match condition {
                    Expr::BinOp { left, op, right, .. } => {
                        assert!(matches!(op, BinOpKind::Gt));
                        assert!(matches!(**left, Expr::Ident { .. }));
                        assert!(matches!(**right, Expr::Int { value: 0, .. }));
                    }
                    _ => panic!("Expected BinOp"),
                }
            }
            _ => panic!("Expected If"),
        }
        Ok(())
    }

    #[test]
    fn test_fn_def() -> Result<(), ElangError> {
        let program = parse_source("def greet(name):\n  print name\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::FnDef { name, params, body, is_async, is_pure, .. } => {
                assert_eq!(name, "greet");
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], "name");
                assert!(!is_async);
                assert!(!is_pure);
                assert_eq!(body.len(), 1);
                assert!(matches!(&body[0], Statement::Print { .. }));
            }
            _ => panic!("Expected FnDef"),
        }
        Ok(())
    }

    #[test]
    fn test_parse_error_missing_if_end() {
        let result = parse_source("if x > 0\n  print x");
        assert!(result.is_err());
        if let Err(ref err) = result {
            assert!(matches!(err, ElangError::ParseError(_)));
            let msg = format!("{}", err);
            assert!(msg.contains("end") || msg.contains("end of file"));
        }
    }

    #[test]
    fn test_parse_error_missing_fn_end() {
        let result = parse_source("def foo():\n  print 1");
        assert!(result.is_err());
        assert!(matches!(result, Err(ElangError::ParseError(_))));
    }

    #[test]
    fn test_parse_error_missing_match_end() {
        let result = parse_source("match x:\n  1: print \"one\"");
        assert!(result.is_err());
        assert!(matches!(result, Err(ElangError::ParseError(_))));
    }

    #[test]
    fn test_valid_complete_program_no_regression() -> Result<(), ElangError> {
        let program = parse_source("let x = 42\nprint x\nif x > 0:\n  print \"positive\"\nend")?;
        assert_eq!(program.len(), 3);
        Ok(())
    }

    #[test]
    fn test_match_newline_separated_arms() -> Result<(), ElangError> {
        let source = "match status:\n  200: print \"OK\"\n  404: print \"Not found\"\n  _: print \"Unknown\"\nend";
        let program = parse_source(source)?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::Match { arms, .. } => {
                assert_eq!(arms.len(), 3);
            }
            _ => panic!("Expected Match"),
        }
        Ok(())
    }

    #[test]
    fn test_multiline_expression_in_parens() -> Result<(), ElangError> {
        let program = parse_source("let x = (1 +\n 2)")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::LetDecl { name, value, .. } => {
                assert_eq!(name, "x");
                match value {
                    Expr::BinOp { left, op, right, .. } => {
                        assert!(matches!(op, BinOpKind::Add));
                        assert!(matches!(**left, Expr::Int { value: 1, .. }));
                        assert!(matches!(**right, Expr::Int { value: 2, .. }));
                    }
                    _ => panic!("Expected BinOp"),
                }
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_if_block_with_multiple_statements() -> Result<(), ElangError> {
        let program = parse_source("if x > 0:\n  print 1\n  print 2\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::If { then_block, .. } => {
                assert_eq!(then_block.len(), 2);
            }
            _ => panic!("Expected If"),
        }
        Ok(())
    }
}
