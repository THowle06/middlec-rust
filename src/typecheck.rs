use crate::{
    ast::{
        BinaryOp::{self},
        Expr, FuncDef, Program, ProgramUnit, Stmt,
        Type::{self},
        UnaryOp, VarDecl,
    },
    symbol::SymbolTable,
};

pub struct TypeChecker<'a> {
    symbols: &'a mut SymbolTable,
}

impl<'a> TypeChecker<'a> {
    pub fn new(symbols: &'a mut SymbolTable) -> Self {
        Self { symbols }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), String> {
        for unit in &program.units {
            match unit {
                ProgramUnit::Var(decl) => {
                    self.check_var_decl(decl)?;
                }
                ProgramUnit::Func(func) => {
                    self.check_func_def(func)?;
                }
            }
        }
        Ok(())
    }

    fn check_var_decl(&mut self, decl: &VarDecl) -> Result<(), String> {
        if let Some(init) = &decl.init {
            let expr_ty = self.check_expr(init)?;
            if expr_ty != decl.ty {
                return Err(format!(
                    "Type mismatch in declaration of '{}': declared {:?} but initialised with {:?}",
                    decl.name, decl.ty, expr_ty
                ));
            }
        }
        self.symbols.declare(&decl.name, decl.ty.clone())?;
        Ok(())
    }

    fn check_func_def(&mut self, func: &FuncDef) -> Result<(), String> {
        self.symbols.enter_scope();
        for (ty, name) in &func.params {
            self.symbols.declare(name, ty.clone())?;
        }

        for stmt in &func.body {
            self.check_stmt(stmt, &func.return_ty)?;
        }

        self.symbols.exit_scope();
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt, ret_ty: &Type) -> Result<(), String> {
        match stmt {
            Stmt::Var(decl) => self.check_var_decl(decl),
            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
            Stmt::Block(stmts) => {
                self.symbols.enter_scope();
                for s in stmts {
                    self.check_stmt(s, ret_ty)?;
                }
                self.symbols.exit_scope();
                Ok(())
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_ty = self.check_expr(cond)?;
                if cond_ty != Type::Bool {
                    return Err(format!("If condition must be bool, found {:?}", cond_ty));
                }
                self.check_stmt(then_branch, ret_ty)?;
                if let Some(el) = else_branch {
                    self.check_stmt(el, ret_ty)?;
                }
                Ok(())
            }
            Stmt::While { cond, body } => {
                let cond_ty = self.check_expr(cond)?;
                if cond_ty != Type::Bool {
                    return Err(format!("While condition must be bool, found {:?}", cond_ty));
                }
                self.check_stmt(body, ret_ty)?;
                Ok(())
            }
            Stmt::Return(Some(expr)) => {
                let ty = self.check_expr(expr)?;
                if &ty != ret_ty {
                    return Err(format!(
                        "Function return type mismatch: expected {:?}, got {:?}",
                        ret_ty, ty
                    ));
                }
                Ok(())
            }
            Stmt::Return(None) => {
                if *ret_ty != Type::Void {
                    return Err(format!(
                        "Void return in non-void function returning {:?}",
                        ret_ty
                    ));
                }
                Ok(())
            }
            Stmt::Break | Stmt::Continue => Ok(()),
            Stmt::For {
                init,
                cond,
                step,
                body,
            } => {
                self.symbols.enter_scope();
                if let Some(i) = init {
                    self.check_stmt(i, ret_ty)?;
                }
                if let Some(c) = cond {
                    let cond_ty = self.check_expr(c)?;
                    if cond_ty != Type::Bool {
                        return Err(format!(
                            "For loop condition must be bool, found {:?}",
                            cond_ty
                        ));
                    }
                }
                if let Some(s) = step {
                    self.check_expr(s)?;
                }
                self.check_stmt(body, ret_ty)?;
                self.symbols.exit_scope();
                Ok(())
            }
        }
    }

    pub fn check_expr(&self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::IntLit(_) => Ok(Type::Int),
            Expr::CharLit(_) => Ok(Type::Char),
            Expr::BoolLit(_) => Ok(Type::Bool),
            Expr::StringLit(_) => Ok(Type::Pointer(Box::new(Type::Char))),
            Expr::Var(name) => match self.symbols.lookup(name) {
                Some(sym) => Ok(sym.ty.clone()),
                None => Err(format!("Undefined variable: '{}'", name)),
            },
            Expr::Binary { op, lhs, rhs } => {
                let l_ty = self.check_expr(lhs)?;
                let r_ty = self.check_expr(rhs)?;

                match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod => {
                        if l_ty == Type::Int && r_ty == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(format!(
                                "Arithmetic operands must be int, found {:?} and {:?}",
                                l_ty, r_ty
                            ))
                        }
                    }
                    BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                        if l_ty == Type::Int && r_ty == Type::Int {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Comparison operands must be int, found {:?} and {:?}",
                                l_ty, r_ty
                            ))
                        }
                    }
                    BinaryOp::Eq | BinaryOp::Ne => {
                        if l_ty == r_ty {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Equality operands must match, found {:?} and {:?}",
                                l_ty, r_ty
                            ))
                        }
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if l_ty == Type::Bool && r_ty == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Logical operands must be bool, found {:?} and {:?}",
                                l_ty, r_ty
                            ))
                        }
                    }
                    BinaryOp::Assign => {
                        if l_ty == r_ty {
                            Ok(l_ty)
                        } else {
                            Err(format!(
                                "Assignment type mismatch: {:?} and {:?}",
                                l_ty, r_ty
                            ))
                        }
                    }
                    BinaryOp::AddAssign
                    | BinaryOp::SubAssign
                    | BinaryOp::MulAssign
                    | BinaryOp::DivAssign
                    | BinaryOp::ModAssign => {
                        if l_ty == Type::Int && r_ty == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(format!(
                                "Compound assignment requires int, found {:?} and {:?}",
                                l_ty, r_ty
                            ))
                        }
                    }
                }
            }
            Expr::Unary { op, expr } => {
                let inner = self.check_expr(expr)?;
                match op {
                    UnaryOp::Neg => {
                        if inner == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(format!("Unary '-' requires int, found {:?}", inner))
                        }
                    }
                    UnaryOp::Not => {
                        if inner == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(format!("Unary '!' requires bool, found {:?}", inner))
                        }
                    }
                    UnaryOp::AddressOf => Ok(Type::Pointer(Box::new(inner))),
                    UnaryOp::Deref => match inner {
                        Type::Pointer(target) => Ok(*target),
                        other => Err(format!("Cannot deference non-pointer type {:?}", other)),
                    },
                }
            }
            Expr::Call { callee, args } => {
                if callee == "putint" {
                    if args.len() != 1 {
                        return Err("putint expects exactly 1 argument".to_string());
                    }
                    let arg_ty = self.check_expr(&args[0])?;
                    if arg_ty != Type::Int {
                        return Err(format!("putint expects int argument, found {:?}", arg_ty));
                    }
                    return Ok(Type::Void);
                }

                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(Type::Void)
            }
            Expr::Index { base, index } => {
                let idx_ty = self.check_expr(index)?;
                if idx_ty != Type::Int {
                    return Err(format!("Array index must be int, found {:?}", idx_ty));
                }
                let base_ty = self.check_expr(base)?;
                match base_ty {
                    Type::Array(elem_ty, _) | Type::Pointer(elem_ty) => Ok(*elem_ty),
                    other => Err(format!(
                        "Cannot index into non-array/pointer type {:?}",
                        other
                    )),
                }
            }
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => {
                let cond_ty = self.check_expr(cond)?;
                if cond_ty != Type::Bool {
                    return Err(format!(
                        "Ternary condition must be bool, found {:?}",
                        cond_ty
                    ));
                }
                let then_ty = self.check_expr(then_expr)?;
                let else_ty = self.check_expr(else_expr)?;
                if then_ty == else_ty {
                    Ok(then_ty)
                } else {
                    Err(format!(
                        "Ternary branches must match: {:?} vs {:?}",
                        then_ty, else_ty
                    ))
                }
            }
        }
    }
}
