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
        let mut is_else_if = false;
        let else_block = if self.eat(TokenKind::Else) {
            if self.check(&TokenKind::If) {
                is_else_if = true;
                Some(vec![self.parse_if()?])
            } else {
                self.eat(TokenKind::Colon);
                Some(self.parse_block_stopping_at(&[TokenKind::End])?)
            }
        } else {
            None
        };
        if !is_else_if {
            self.expect(TokenKind::End)?;
        }
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
            let var = self.expect_ident()?;
            let error_type = if self.check_ident_str("is") {
                self.advance()?;
                Some(self.expect_ident()?)
            } else {
                None
            };
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
                let rhs = self.parse_expr_bp(bp + 1)?;
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

    #[test]
    fn test_else_if_chain_three_branches() -> Result<(), ElangError> {
        let program = parse_source(
            "if score >= 90:\n  print \"A\"\nelse if score >= 80:\n  print \"B\"\nelse if score >= 70:\n  print \"C\"\nend"
        )?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::If { else_block, .. } => {
                assert!(else_block.is_some());
                let block = else_block.as_ref().unwrap();
                assert_eq!(block.len(), 1);
                match &block[0] {
                    Statement::If { else_block: inner_else, .. } => {
                        assert!(inner_else.is_some());
                        let inner = inner_else.as_ref().unwrap();
                        assert_eq!(inner.len(), 1);
                        assert!(matches!(&inner[0], Statement::If { .. }));
                    }
                    _ => panic!("Expected nested If"),
                }
            }
            _ => panic!("Expected If"),
        }
        Ok(())
    }

    #[test]
    fn test_else_if_with_final_else() -> Result<(), ElangError> {
        let program = parse_source(
            "if score >= 90:\n  print \"A\"\nelse if score >= 80:\n  print \"B\"\nelse:\n  print \"F\"\nend"
        )?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::If { else_block, .. } => {
                assert!(else_block.is_some());
                let block = else_block.as_ref().unwrap();
                assert_eq!(block.len(), 1);
                match &block[0] {
                    Statement::If { else_block: inner_else, .. } => {
                        assert!(inner_else.is_some());
                        let inner = inner_else.as_ref().unwrap();
                        assert_eq!(inner.len(), 1);
                        assert!(matches!(&inner[0], Statement::Print { .. }));
                    }
                    _ => panic!("Expected nested If"),
                }
            }
            _ => panic!("Expected If"),
        }
        Ok(())
    }

    #[test]
    fn test_plain_if_no_else() -> Result<(), ElangError> {
        let program = parse_source("if x > 0:\n  print \"positive\"\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::If { else_block, .. } => {
                assert!(else_block.is_none());
            }
            _ => panic!("Expected If"),
        }
        Ok(())
    }

    #[test]
    fn test_not_false_parses() -> Result<(), ElangError> {
        let program = parse_source("if not false:\n  print \"yes\"\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::If { condition, .. } => {
                assert!(matches!(condition, Expr::UnaryOp { op: UnaryOpKind::Not, .. }));
            }
            _ => panic!("Expected If"),
        }
        Ok(())
    }

    #[test]
    fn test_not_true_parses() -> Result<(), ElangError> {
        let program = parse_source("if not true:\n  print \"no\"\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::If { condition, .. } => {
                assert!(matches!(condition, Expr::UnaryOp { op: UnaryOpKind::Not, .. }));
            }
            _ => panic!("Expected If"),
        }
        Ok(())
    }

    #[test]
    fn test_not_method_call_parses() -> Result<(), ElangError> {
        let program = parse_source("if not is_done():\n  print \"still running\"\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::If { condition, .. } => {
                match condition {
                    Expr::UnaryOp { op, expr, .. } => {
                        assert!(matches!(op, UnaryOpKind::Not));
                        assert!(matches!(expr.as_ref(), Expr::Call { .. }));
                    }
                    _ => panic!("Expected UnaryOp"),
                }
            }
            _ => panic!("Expected If"),
        }
        Ok(())
    }

    // === Declaration variants ===

    #[test]
    fn test_var_decl() -> Result<(), ElangError> {
        let program = parse_source("var x = 10")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::VarDecl { name, value, .. } => {
                assert_eq!(name, "x");
                assert!(matches!(value, Expr::Int { value: 10, .. }));
            }
            _ => panic!("Expected VarDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_const_decl() -> Result<(), ElangError> {
        let program = parse_source("const MAX = 100")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::ConstDecl { name, value, .. } => {
                assert_eq!(name, "MAX");
                assert!(matches!(value, Expr::Int { value: 100, .. }));
            }
            _ => panic!("Expected ConstDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_var_no_init() -> Result<(), ElangError> {
        let program = parse_source("var x = nothing")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::VarDecl { name, value, .. } => {
                assert_eq!(name, "x");
                assert!(matches!(value, Expr::Nothing { .. }));
            }
            _ => panic!("Expected VarDecl"),
        }
        Ok(())
    }

    // === Assignment ===

    #[test]
    fn test_assign_statement() -> Result<(), ElangError> {
        let program = parse_source("x = 42")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::Assign { name, value, .. } => {
                assert_eq!(name, "x");
                assert!(matches!(value, Expr::Int { value: 42, .. }));
            }
            _ => panic!("Expected Assign"),
        }
        Ok(())
    }

    // === While loop ===

    #[test]
    fn test_while_loop() -> Result<(), ElangError> {
        let program = parse_source("while i < 10:\n  i = i + 1\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::Loop { kind, body, .. } => {
                assert!(matches!(kind, LoopKind::While(_)));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected Loop"),
        }
        Ok(())
    }

    // === Loop forever ===

    #[test]
    fn test_loop_forever() -> Result<(), ElangError> {
        let program = parse_source("loop:\n  break\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::Loop { kind, body, .. } => {
                assert!(matches!(kind, LoopKind::Forever));
                assert_eq!(body.len(), 1);
                assert!(matches!(&body[0], Statement::Break { .. }));
            }
            _ => panic!("Expected Loop"),
        }
        Ok(())
    }

    // === For in ===

    #[test]
    fn test_for_in_loop() -> Result<(), ElangError> {
        let program = parse_source("for item in items:\n  print item\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::ForIn { var, iterable, body, .. } => {
                assert_eq!(var, "item");
                assert!(matches!(iterable, Expr::Ident { name, .. } if name == "items"));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected ForIn"),
        }
        Ok(())
    }

    // === Repeat N times ===

    #[test]
    fn test_repeat_n_times() -> Result<(), ElangError> {
        let program = parse_source("repeat 5 times:\n  print \"hi\"\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::Loop { kind, body, .. } => {
                match kind {
                    LoopKind::RepeatN(expr) => assert!(matches!(expr, Expr::Int { value: 5, .. })),
                    _ => panic!("Expected RepeatN"),
                }
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected Loop"),
        }
        Ok(())
    }

    // === Repeat with range ===

    #[test]
    fn test_repeat_range() -> Result<(), ElangError> {
        let program = parse_source("repeat i from 1 to 5:\n  print i\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::Loop { kind, body, .. } => {
                match kind {
                    LoopKind::RepeatRange { var, from, to } => {
                        assert_eq!(var, "i");
                        assert!(matches!(from, Expr::Int { value: 1, .. }));
                        assert!(matches!(to, Expr::Int { value: 5, .. }));
                    }
                    _ => panic!("Expected RepeatRange"),
                }
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected Loop"),
        }
        Ok(())
    }

    // === Class ===

    #[test]
    fn test_empty_class() -> Result<(), ElangError> {
        let program = parse_source("class Foo:\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::ClassDef { name, parent, .. } => {
                assert_eq!(name, "Foo");
                assert!(parent.is_none());
            }
            _ => panic!("Expected ClassDef"),
        }
        Ok(())
    }

    #[test]
    fn test_class_with_extends() -> Result<(), ElangError> {
        let program = parse_source("class Dog extends Animal:\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::ClassDef { name, parent, .. } => {
                assert_eq!(name, "Dog");
                assert_eq!(parent.as_deref(), Some("Animal"));
            }
            _ => panic!("Expected ClassDef"),
        }
        Ok(())
    }

    #[test]
    fn test_class_with_fields_and_methods() -> Result<(), ElangError> {
        let program = parse_source("class Counter:\n  pub count = 0\n  pub def inc():\n    self.count = self.count + 1\n  end\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::ClassDef { name, body, .. } => {
                assert_eq!(name, "Counter");
                assert_eq!(body.len(), 2);
                assert!(matches!(&body[0], Statement::Field { name: f, .. } if f == "count"));
                assert!(matches!(&body[1], Statement::FnDef { name: fn_name, .. } if fn_name == "inc"));
            }
            _ => panic!("Expected ClassDef"),
        }
        Ok(())
    }

    // === Return, Break, Continue ===

    #[test]
    fn test_return_value() -> Result<(), ElangError> {
        let program = parse_source("def foo():\n  return 42\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::FnDef { body, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0] {
                    Statement::Return { value, .. } => {
                        assert!(matches!(value, Expr::Int { value: 42, .. }));
                    }
                    _ => panic!("Expected Return"),
                }
            }
            _ => panic!("Expected FnDef"),
        }
        Ok(())
    }

    #[test]
    fn test_break_statement() -> Result<(), ElangError> {
        let program = parse_source("loop:\n  break\nend")?;
        assert!(matches!(&program[0], Statement::Loop { .. }));
        Ok(())
    }

    #[test]
    fn test_continue_statement() -> Result<(), ElangError> {
        let program = parse_source("loop:\n  continue\nend")?;
        assert!(matches!(&program[0], Statement::Loop { .. }));
        Ok(())
    }

    // === Try/Catch ===

    #[test]
    fn test_try_catch() -> Result<(), ElangError> {
        let program = parse_source("try:\n  risky()\ncatch err:\n  print err\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::Try { body, catches, .. } => {
                assert_eq!(body.len(), 1);
                assert_eq!(catches.len(), 1);
                assert_eq!(catches[0].var, "err");
                assert!(catches[0].error_type.is_none());
            }
            _ => panic!("Expected Try"),
        }
        Ok(())
    }

    #[test]
    fn test_try_catch_with_error_type() -> Result<(), ElangError> {
        let program = parse_source("try:\n  risky()\ncatch err is RuntimeError:\n  print err\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::Try { body, catches, .. } => {
                assert_eq!(catches.len(), 1);
                assert_eq!(catches[0].var, "err");
                assert_eq!(catches[0].error_type.as_deref(), Some("RuntimeError"));
            }
            _ => panic!("Expected Try"),
        }
        Ok(())
    }

    // === Import / Export ===

    #[test]
    fn test_import() -> Result<(), ElangError> {
        let program = parse_source("import math")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::Import { module, .. } => assert_eq!(module, "math"),
            _ => panic!("Expected Import"),
        }
        Ok(())
    }

    #[test]
    fn test_export_function() -> Result<(), ElangError> {
        let program = parse_source("export def foo():\n  print 1\nend")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::Export { stmt, .. } => {
                assert!(matches!(stmt.as_ref(), Statement::FnDef { name, .. } if name == "foo"));
            }
            _ => panic!("Expected Export"),
        }
        Ok(())
    }

    // === Expression types ===

    #[test]
    fn test_float_literal_expr() -> Result<(), ElangError> {
        let program = parse_source("let x = 3.14")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => assert!(matches!(value, Expr::Float { .. })),
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_bool_literal_expr() -> Result<(), ElangError> {
        let program = parse_source("let a = true\nlet b = false")?;
        assert_eq!(program.len(), 2);
        match (&program[0], &program[1]) {
            (Statement::LetDecl { value: a_val, .. }, Statement::LetDecl { value: b_val, .. }) => {
                assert!(matches!(a_val, Expr::Bool { value: true, .. }));
                assert!(matches!(b_val, Expr::Bool { value: false, .. }));
            }
            _ => panic!("Expected LetDecls"),
        }
        Ok(())
    }

    #[test]
    fn test_str_literal_expr() -> Result<(), ElangError> {
        let program = parse_source(r#"let s = "hello""#)?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                assert!(matches!(value, Expr::Str { value: s, .. } if s == "hello"));
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_nothing_literal_expr() -> Result<(), ElangError> {
        let program = parse_source("let n = nothing")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => assert!(matches!(value, Expr::Nothing { .. })),
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_list_literal() -> Result<(), ElangError> {
        let program = parse_source("let items = [1, 2, 3]")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                match value {
                    Expr::List { items, .. } => {
                        assert_eq!(items.len(), 3);
                    }
                    _ => panic!("Expected List"),
                }
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_empty_list() -> Result<(), ElangError> {
        let program = parse_source("let e = []")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                assert!(matches!(value, Expr::List { items, .. } if items.is_empty()));
            }
            _ => panic!("Expected empty List"),
        }
        Ok(())
    }

    #[test]
    fn test_map_literal() -> Result<(), ElangError> {
        let program = parse_source("{a: 1, b: 2}")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::ExprStmt { expr, .. } => {
                match expr {
                    Expr::Map { pairs, .. } => {
                        assert_eq!(pairs.len(), 2);
                    }
                    _ => panic!("Expected Map"),
                }
            }
            _ => panic!("Expected ExprStmt"),
        }
        Ok(())
    }

    #[test]
    fn test_index_expression() -> Result<(), ElangError> {
        let program = parse_source("let x = items[0]")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                assert!(matches!(value, Expr::Index { .. }));
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_field_access() -> Result<(), ElangError> {
        let program = parse_source("let x = obj.field")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                assert!(matches!(value, Expr::Field { .. }));
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_lambda_simple() -> Result<(), ElangError> {
        let program = parse_source("let double = fn(x) => x * 2")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                assert!(matches!(value, Expr::Lambda { params, .. } if params.len() == 1));
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_pipe_expression() -> Result<(), ElangError> {
        let program = parse_source("let r = 5 |> double")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                assert!(matches!(value, Expr::Pipe { .. }));
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_function_call() -> Result<(), ElangError> {
        let program = parse_source("let r = add(1, 2, 3)")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                match value {
                    Expr::Call { callee, args, .. } => {
                        assert!(matches!(callee.as_ref(), Expr::Ident { name, .. } if name == "add"));
                        assert_eq!(args.len(), 3);
                    }
                    _ => panic!("Expected Call"),
                }
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    // === Operator precedence ===

    #[test]
    fn test_mul_over_add_precedence() -> Result<(), ElangError> {
        let program = parse_source("let x = 1 + 2 * 3")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                match value {
                    Expr::BinOp { left, op, right, .. } => {
                        assert!(matches!(op, BinOpKind::Add));
                        assert!(matches!(**left, Expr::Int { value: 1, .. }));
                        assert!(matches!(**right, Expr::BinOp { op: inner_op, .. } if matches!(inner_op, BinOpKind::Mul)));
                    }
                    _ => panic!("Expected BinOp"),
                }
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    #[test]
    fn test_parens_override_precedence() -> Result<(), ElangError> {
        let program = parse_source("let x = (1 + 2) * 3")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                match value {
                    Expr::BinOp { left, op, right, .. } => {
                        assert!(matches!(op, BinOpKind::Mul));
                        assert!(matches!(**left, Expr::BinOp { op: inner_op, .. } if matches!(inner_op, BinOpKind::Add)));
                        assert!(matches!(**right, Expr::Int { value: 3, .. }));
                    }
                    _ => panic!("Expected BinOp"),
                }
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    // === Unary negation ===

    #[test]
    fn test_unary_negation() -> Result<(), ElangError> {
        let program = parse_source("let x = -5")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                assert!(matches!(value, Expr::UnaryOp { op: UnaryOpKind::Neg, .. }));
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    // === Comparison operators ===

    #[test]
    fn test_comparison_operators() -> Result<(), ElangError> {
        let ops = [("==", BinOpKind::Eq), ("!=", BinOpKind::NotEq), ("<", BinOpKind::Lt),
                    (">", BinOpKind::Gt), ("<=", BinOpKind::LtEq), (">=", BinOpKind::GtEq)];
        for (op_str, expected_op) in ops {
            let source = format!("let x = 1 {} 2", op_str);
            let program = parse_source(&source)?;
            match &program[0] {
                Statement::LetDecl { value, .. } => {
                    assert!(matches!(value, Expr::BinOp { op, .. } if op == &expected_op),
                        "expected BinOp with {:?} for '{}'", expected_op, op_str);
                }
                _ => panic!("Expected LetDecl"),
            }
        }
        Ok(())
    }

    #[test]
    fn test_logical_and_or() -> Result<(), ElangError> {
        let program = parse_source("let x = true and false or true")?;
        match &program[0] {
            Statement::LetDecl { value, .. } => {
                match value {
                    Expr::BinOp { op, .. } => {
                        assert!(matches!(op, BinOpKind::Or));
                    }
                    _ => panic!("Expected BinOp"),
                }
            }
            _ => panic!("Expected LetDecl"),
        }
        Ok(())
    }

    // === Empty program ===

    #[test]
    fn test_empty_program() -> Result<(), ElangError> {
        let program = parse_source("")?;
        assert_eq!(program.len(), 0);
        Ok(())
    }

    // === Statement as expression ===

    #[test]
    fn test_bare_expression() -> Result<(), ElangError> {
        let program = parse_source("42")?;
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::ExprStmt { expr, .. } => {
                assert!(matches!(expr, Expr::Int { value: 42, .. }));
            }
            _ => panic!("Expected ExprStmt"),
        }
        Ok(())
    }

    // === Error cases ===

    #[test]
    fn test_error_missing_try_end() {
        let result = parse_source("try:\n  risky()");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_while_end() {
        let result = parse_source("while true:\n  print 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_loop_end() {
        let result = parse_source("loop:\n  break");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_parse_invalid_expression() {
        let result = parse_source("let x = +");
        assert!(result.is_err());
    }
}
