use crate::{
    ast::{BinaryOp, Expr, FuncDef, Program, ProgramUnit, Stmt, Type, UnaryOp, VarDecl},
    token::Token,
};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // --- Helper Lookahead & Consumption Methods ---

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_offset(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    fn check(&self, expected: &Token) -> bool {
        self.peek() == Some(expected)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        match self.advance() {
            Some(tok) if tok == expected => Ok(()),
            Some(actual) => Err(format!("Expected {:?}, but found {:?}", expected, actual)),
            None => Err(format!("Expected {:?}, but reached EOF", expected)),
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Some(Token::Ident(name)) => Ok(name),
            Some(actual) => Err(format!("Expected identifier, found {:?}", actual)),
            None => Err("Expected identifier, reached EOF".to_string()),
        }
    }

    // --- Program & Top-Level Declarations ---

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut units = Vec::new();
        while self.peek().is_some() {
            units.push(self.parse_program_unit()?);
        }
        Ok(Program { units })
    }

    fn parse_program_unit(&mut self) -> Result<ProgramUnit, String> {
        let ty = self.parse_type()?;
        let name = self.expect_ident()?;

        // Lookahead to differentiate function definition from global variable
        if self.check(&Token::LParen) {
            self.advance();
            let params = self.parse_param_list()?;
            self.expect(Token::RParen)?;
            let body = self.parse_block()?;
            Ok(ProgramUnit::Func(FuncDef {
                return_ty: ty,
                name,
                params,
                body,
            }))
        } else {
            let init = if self.check(&Token::Assign) {
                self.advance();
                Some(self.parse_expr(0)?)
            } else {
                None
            };
            self.expect(Token::Semi)?;
            Ok(ProgramUnit::Var(VarDecl { ty, name, init }))
        }
    }

    fn parse_param_list(&mut self) -> Result<Vec<(Type, String)>, String> {
        let mut params = Vec::new();
        if self.check(&Token::RParen) {
            return Ok(params);
        }
        loop {
            let ty = self.parse_type()?;
            let name = self.expect_ident()?;
            params.push((ty, name));
            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let base = match self.advance() {
            Some(Token::KwInt) => Type::Int,
            Some(Token::KwChar) => Type::Char,
            Some(Token::KwBool) => Type::Bool,
            Some(Token::KwVoid) => Type::Void,
            Some(Token::KwStruct) => {
                let name = self.expect_ident()?;
                Type::Struct(name)
            }
            Some(tok) => return Err(format!("Expected type specifier, found {:?}", tok)),
            None => return Err("Expected type specifier, reached EOF".to_string()),
        };

        let mut ty = base;
        while self.check(&Token::Star) {
            self.advance();
            ty = Type::Pointer(Box::new(ty));
        }
        Ok(ty)
    }

    // --- Statements ---

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(Token::LBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&Token::RBrace) && self.peek().is_some() {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(Token::RBrace)?;
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Some(Token::LBrace) => {
                let block = self.parse_block()?;
                Ok(Stmt::Block(block))
            }
            Some(Token::KwIf) => {
                self.advance();
                self.expect(Token::LParen)?;
                let cond = self.parse_expr(0)?;
                self.expect(Token::RParen)?;
                let then_branch = Box::new(self.parse_stmt()?);
                let else_branch = if self.check(&Token::KwElse) {
                    self.advance();
                    Some(Box::new(self.parse_stmt()?))
                } else {
                    None
                };
                Ok(Stmt::If {
                    cond,
                    then_branch,
                    else_branch,
                })
            }
            Some(Token::KwWhile) => {
                self.advance();
                self.expect(Token::LParen)?;
                let cond = self.parse_expr(0)?;
                self.expect(Token::RParen)?;
                let body = Box::new(self.parse_stmt()?);
                Ok(Stmt::While { cond, body })
            }
            Some(Token::KwReturn) => {
                self.advance();
                let expr = if !self.check(&Token::Semi) {
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };
                self.expect(Token::Semi)?;
                Ok(Stmt::Return(expr))
            }
            Some(Token::KwBreak) => {
                self.advance();
                self.expect(Token::Semi)?;
                Ok(Stmt::Break)
            }
            Some(Token::KwContinue) => {
                self.advance();
                self.expect(Token::Semi)?;
                Ok(Stmt::Continue)
            }
            Some(
                Token::KwInt | Token::KwChar | Token::KwBool | Token::KwVoid | Token::KwStruct,
            ) => {
                // Local Variable Declaration
                let ty = self.parse_type()?;
                let name = self.expect_ident()?;
                let init = if self.check(&Token::Assign) {
                    self.advance();
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };
                self.expect(Token::Semi)?;
                Ok(Stmt::Var(VarDecl { ty, name, init }))
            }
            _ => {
                // Expression Statement
                let expr = self.parse_expr(0)?;
                self.expect(Token::Semi)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    // --- Pratt Expression Parsing ---

    fn get_infix_precedence(tok: &Token) -> (u8, u8) {
        match tok {
            // Assignment operators: lowest precedence, right-associative
            Token::Assign
            | Token::AddAssign
            | Token::SubAssign
            | Token::MulAssign
            | Token::DivAssign
            | Token::ModAssign => (2, 1),
            // Logical OR
            Token::OrOr => (3, 4),
            // Logical AND
            Token::AndAnd => (5, 6),
            // Equality
            Token::Eq | Token::Ne => (7, 8),
            // Relational
            Token::Lt | Token::Le | Token::Gt | Token::Ge => (9, 10),
            // Additive
            Token::Plus | Token::Minus => (11, 12),
            // Multiplicative
            Token::Star | Token::Slash | Token::Percent => (13, 14),
            _ => (0, 0),
        }
    }

    pub fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, String> {
        let mut lhs = self.parse_prefix()?;

        // Postfix operators: function call '()', array index '[]'
        loop {
            match self.peek() {
                Some(Token::LParen) => {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    self.expect(Token::RParen)?;
                    if let Expr::Var(callee) = lhs {
                        lhs = Expr::Call { callee, args };
                    } else {
                        return Err("Callee must be a function identifier".to_string());
                    }
                }
                Some(Token::LBracket) => {
                    self.advance();
                    let index = self.parse_expr(0)?;
                    self.expect(Token::RBracket)?;
                    lhs = Expr::Index {
                        base: Box::new(lhs),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }

        // Infix operators
        while let Some(tok) = self.peek() {
            let (l_bp, r_bp) = Self::get_infix_precedence(tok);
            if l_bp < min_bp || l_bp == 0 {
                break;
            }

            let op_tok = self.advance().unwrap();
            let op = match op_tok {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                Token::Eq => BinaryOp::Eq,
                Token::Ne => BinaryOp::Ne,
                Token::Lt => BinaryOp::Lt,
                Token::Le => BinaryOp::Le,
                Token::Gt => BinaryOp::Gt,
                Token::Ge => BinaryOp::Ge,
                Token::AndAnd => BinaryOp::And,
                Token::OrOr => BinaryOp::Or,
                Token::Assign => BinaryOp::Assign,
                Token::AddAssign => BinaryOp::AddAssign,
                Token::SubAssign => BinaryOp::SubAssign,
                Token::MulAssign => BinaryOp::MulAssign,
                Token::DivAssign => BinaryOp::DivAssign,
                Token::ModAssign => BinaryOp::ModAssign,
                _ => unreachable!(),
            };

            let rhs = self.parse_expr(r_bp)?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Some(Token::IntLit(n)) => Ok(Expr::IntLit(n)),
            Some(Token::CharLit(c)) => Ok(Expr::CharLit(c)),
            Some(Token::StringLit(s)) => Ok(Expr::StringLit(s)),
            Some(Token::KwTrue) => Ok(Expr::BoolLit(true)),
            Some(Token::KwFalse) => Ok(Expr::BoolLit(false)),
            Some(Token::Ident(name)) => Ok(Expr::Var(name)),
            Some(Token::LParen) => {
                let inner = self.parse_expr(0)?;
                self.expect(Token::RParen)?;
                Ok(inner)
            }
            Some(Token::Minus) => {
                let operand = self.parse_expr(15)?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(operand),
                })
            }
            Some(Token::Bang) => {
                let operand = self.parse_expr(15)?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(operand),
                })
            }
            Some(Token::Star) => {
                let operand = self.parse_expr(15)?;
                Ok(Expr::Unary {
                    op: UnaryOp::Deref,
                    expr: Box::new(operand),
                })
            }
            Some(Token::Ampersand) => {
                let operand = self.parse_expr(15)?;
                Ok(Expr::Unary {
                    op: UnaryOp::AddressOf,
                    expr: Box::new(operand),
                })
            }
            Some(tok) => Err(format!("Expected expression, found {:?}", tok)),
            None => Err("Expected expression, reached EOF".to_string()),
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.check(&Token::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expr(0)?);
            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(args)
    }
}
