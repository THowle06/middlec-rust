use crate::ast::*;
use crate::iloc::{IlocOp, IlocProgram, Reg};
use crate::symbol::SymbolTable;
use std::collections::HashMap;

pub struct CodeGen {
    program: IlocProgram,
    reg_counter: usize,
    label_counter: usize,
    symbols: SymbolTable,
    pub string_literals: Vec<(String, String)>,
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            program: IlocProgram::default(),
            reg_counter: 0,
            label_counter: 0,
            symbols: SymbolTable::new(),
            string_literals: Vec::new(),
        }
    }

    fn new_reg(&mut self) -> Reg {
        let r = Reg::Virtual(self.reg_counter);
        self.reg_counter += 1;
        r
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let lbl = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        lbl
    }

    pub fn generate(mut self, ast: &Program) -> (IlocProgram, usize, Vec<(String, String)>) {
        for unit in &ast.units {
            if let ProgramUnit::Func(func) = unit {
                self.gen_func(func);
            }
        }
        let stack_bytes = self.symbols.total_stack_bytes();
        (self.program, stack_bytes, self.string_literals)
    }

    fn gen_func(&mut self, func: &FuncDef) {
        self.program.emit(IlocOp::Label(func.name.clone()));
        self.symbols.enter_scope();

        for stmt in &func.body {
            self.gen_stmt(stmt);
        }

        self.symbols.exit_scope();
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Var(decl) => {
                let sym = self.symbols.declare(&decl.name, decl.ty.clone()).unwrap();
                if let Some(init) = &decl.init {
                    let val_reg = self.gen_expr(init);
                    self.program.emit(IlocOp::StoreAI {
                        src: val_reg,
                        offset: sym.stack_offset,
                        base: Reg::FP,
                    });
                }
            }
            Stmt::Expr(expr) => {
                self.gen_expr(expr);
            }
            Stmt::Block(stmts) => {
                self.symbols.enter_scope();
                for s in stmts {
                    self.gen_stmt(s);
                }
                self.symbols.exit_scope();
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_reg = self.gen_expr(cond);
                let true_lbl = self.new_label("if_true");
                let false_lbl = self.new_label("if_false");
                let end_lbl = self.new_label("if_end");

                if let Some(else_stmt) = else_branch {
                    self.program.emit(IlocOp::Cbr {
                        cond: cond_reg,
                        true_lbl: true_lbl.clone(),
                        false_lbl: false_lbl.clone(),
                    });

                    self.program.emit(IlocOp::Label(true_lbl));
                    self.gen_stmt(then_branch);
                    self.program.emit(IlocOp::Jump(end_lbl.clone()));

                    self.program.emit(IlocOp::Label(false_lbl));
                    self.gen_stmt(else_stmt);
                    self.program.emit(IlocOp::Jump(end_lbl.clone()));
                } else {
                    self.program.emit(IlocOp::Cbr {
                        cond: cond_reg,
                        true_lbl: true_lbl.clone(),
                        false_lbl: false_lbl.clone(),
                    });

                    self.program.emit(IlocOp::Label(true_lbl));
                    self.gen_stmt(then_branch);
                    self.program.emit(IlocOp::Jump(end_lbl.clone()));
                }

                self.program.emit(IlocOp::Label(end_lbl));
            }
            Stmt::While { cond, body } => {
                let start_lbl = self.new_label("while_start");
                let body_lbl = self.new_label("while_body");
                let end_lbl = self.new_label("while_end");

                self.program.emit(IlocOp::Label(start_lbl.clone()));
                let cond_reg = self.gen_expr(cond);
                self.program.emit(IlocOp::Cbr {
                    cond: cond_reg,
                    true_lbl: body_lbl.clone(),
                    false_lbl: end_lbl.clone(),
                });

                self.program.emit(IlocOp::Label(body_lbl));
                self.gen_stmt(body);
                self.program.emit(IlocOp::Jump(start_lbl));

                self.program.emit(IlocOp::Label(end_lbl));
            }
            _ => {}
        }
    }

    fn gen_expr(&mut self, expr: &Expr) -> Reg {
        match expr {
            Expr::IntLit(n) => {
                let dst = self.new_reg();
                self.program.emit(IlocOp::LoadI(*n, dst.clone()));
                dst
            }
            Expr::BoolLit(b) => {
                let dst = self.new_reg();
                self.program
                    .emit(IlocOp::LoadI(if *b { 1 } else { 0 }, dst.clone()));
                dst
            }
            Expr::StringLit(s) => {
                let lbl = self.new_label("str");
                self.string_literals.push((lbl.clone(), s.clone()));
                let dst = self.new_reg();
                self.program.emit(IlocOp::LoadLabel(lbl, dst.clone()));
                dst
            }
            Expr::Var(name) => {
                let sym = self.symbols.lookup(name).unwrap().clone();
                let dst = self.new_reg();
                self.program.emit(IlocOp::LoadAI {
                    base: Reg::FP,
                    offset: sym.stack_offset,
                    dst: dst.clone(),
                });
                dst
            }
            Expr::Binary { op, lhs, rhs } => {
                if *op == BinaryOp::Assign {
                    if let Expr::Var(name) = lhs.as_ref() {
                        let sym = self.symbols.lookup(name).unwrap().clone();
                        let rhs_reg = self.gen_expr(rhs);
                        self.program.emit(IlocOp::StoreAI {
                            src: rhs_reg.clone(),
                            offset: sym.stack_offset,
                            base: Reg::FP,
                        });
                        return rhs_reg;
                    }
                }

                let l_reg = self.gen_expr(lhs);
                let r_reg = self.gen_expr(rhs);
                let dst = self.new_reg();

                let op_code = match op {
                    BinaryOp::Add => IlocOp::Add(l_reg, r_reg, dst.clone()),
                    BinaryOp::Sub => IlocOp::Sub(l_reg, r_reg, dst.clone()),
                    BinaryOp::Mul => IlocOp::Mul(l_reg, r_reg, dst.clone()),
                    BinaryOp::Div => IlocOp::Div(l_reg, r_reg, dst.clone()),
                    BinaryOp::Mod => IlocOp::Mod(l_reg, r_reg, dst.clone()),
                    BinaryOp::Eq => IlocOp::CmpEq(l_reg, r_reg, dst.clone()),
                    BinaryOp::Ne => IlocOp::CmpNe(l_reg, r_reg, dst.clone()),
                    BinaryOp::Lt => IlocOp::CmpLt(l_reg, r_reg, dst.clone()),
                    BinaryOp::Le => IlocOp::CmpLe(l_reg, r_reg, dst.clone()),
                    BinaryOp::Gt => IlocOp::CmpGt(l_reg, r_reg, dst.clone()),
                    BinaryOp::Ge => IlocOp::CmpGe(l_reg, r_reg, dst.clone()),
                    _ => unimplemented!("Op {:?}", op),
                };
                self.program.emit(op_code);
                dst
            }
            Expr::Call { callee, args } => {
                if callee == "putint" || callee == "putstr" || callee == "put" {
                    let arg_reg = self.gen_expr(&args[0]);
                    self.program
                        .emit(IlocOp::Add(arg_reg, Reg::Virtual(9999), Reg::R0));
                    self.program.emit(IlocOp::Call(callee.clone()));
                    Reg::R0
                } else {
                    unimplemented!("User function calls")
                }
            }
            _ => unimplemented!("Expr: {:?}", expr),
        }
    }
}

pub struct ArmEmitter {
    reg_map: HashMap<Reg, String>,
    available_regs: Vec<&'static str>,
    next_reg_idx: usize,
}

impl ArmEmitter {
    pub fn new() -> Self {
        Self {
            reg_map: HashMap::new(),
            available_regs: vec!["r4", "r5", "r6", "r7", "r8", "r9", "r10"],
            next_reg_idx: 0,
        }
    }

    fn get_arm_reg(&mut self, reg: &Reg) -> String {
        match reg {
            Reg::FP => "r11".to_string(),
            Reg::SP => "sp".to_string(),
            Reg::R0 => "r0".to_string(),
            Reg::Virtual(9999) => "r0".to_string(),
            Reg::Virtual(_) => self
                .reg_map
                .entry(reg.clone())
                .or_insert_with(|| {
                    let r = self.available_regs[self.next_reg_idx % self.available_regs.len()];
                    self.next_reg_idx += 1;
                    r.to_string()
                })
                .clone(),
        }
    }

    pub fn emit_arm(
        &mut self,
        iloc: &IlocProgram,
        stack_bytes: usize,
        strings: &[(String, String)],
    ) -> String {
        let mut out = String::new();

        // 1. Program Entry point
        out.push_str("    mov sp, #8192\n");
        out.push_str("    bl main\n");
        out.push_str("    swi 2\n\n");

        // 2. Instructions
        for op in &iloc.ops {
            match op {
                IlocOp::Label(name) => {
                    out.push_str(&format!("{}:\n", name));
                    if name == "main" {
                        out.push_str("    stmfd sp!, {r4-r11, lr}\n");
                        out.push_str("    mov r11, sp\n");
                        if stack_bytes > 0 {
                            out.push_str(&format!("    sub sp, sp, #{}\n", stack_bytes));
                        }
                    }
                }
                IlocOp::LoadI(val, dst) => {
                    let d = self.get_arm_reg(dst);
                    out.push_str(&format!("    mov {}, #{}\n", d, val));
                }
                IlocOp::LoadLabel(lbl, dst) => {
                    let d = self.get_arm_reg(dst);
                    out.push_str(&format!("    adr {}, {}\n", d, lbl));
                }
                IlocOp::LoadAI { base, offset, dst } => {
                    let b = self.get_arm_reg(base);
                    let d = self.get_arm_reg(dst);
                    out.push_str(&format!("    ldr {}, [{}, #{}]\n", d, b, offset));
                }
                IlocOp::StoreAI { src, offset, base } => {
                    let s = self.get_arm_reg(src);
                    let b = self.get_arm_reg(base);
                    out.push_str(&format!("    str {}, [{}, #{}]\n", s, b, offset));
                }
                IlocOp::Add(lhs, rhs, dst) => {
                    if let Reg::Virtual(9999) = rhs {
                        // Copy assignment to r0
                        let s = self.get_arm_reg(lhs);
                        let d = self.get_arm_reg(dst);
                        out.push_str(&format!("    mov {}, {}\n", d, s));
                    } else {
                        let l = self.get_arm_reg(lhs);
                        let r = self.get_arm_reg(rhs);
                        let d = self.get_arm_reg(dst);
                        out.push_str(&format!("    add {}, {}, {}\n", d, l, r));
                    }
                }
                IlocOp::Sub(lhs, rhs, dst) => {
                    let l = self.get_arm_reg(lhs);
                    let r = self.get_arm_reg(rhs);
                    let d = self.get_arm_reg(dst);
                    out.push_str(&format!("    sub {}, {}, {}\n", d, l, r));
                }
                IlocOp::Mul(lhs, rhs, dst) => {
                    let l = self.get_arm_reg(lhs);
                    let r = self.get_arm_reg(rhs);
                    let d = self.get_arm_reg(dst);
                    out.push_str(&format!("    mul {}, {}, {}\n", d, l, r));
                }
                IlocOp::CmpEq(lhs, rhs, dst) => self.emit_cmp(lhs, rhs, dst, "eq", &mut out),
                IlocOp::CmpNe(lhs, rhs, dst) => self.emit_cmp(lhs, rhs, dst, "ne", &mut out),
                IlocOp::CmpLt(lhs, rhs, dst) => self.emit_cmp(lhs, rhs, dst, "lt", &mut out),
                IlocOp::CmpLe(lhs, rhs, dst) => self.emit_cmp(lhs, rhs, dst, "le", &mut out),
                IlocOp::CmpGt(lhs, rhs, dst) => self.emit_cmp(lhs, rhs, dst, "gt", &mut out),
                IlocOp::CmpGe(lhs, rhs, dst) => self.emit_cmp(lhs, rhs, dst, "ge", &mut out),
                IlocOp::Cbr {
                    cond,
                    true_lbl,
                    false_lbl,
                } => {
                    let c = self.get_arm_reg(cond);
                    out.push_str(&format!("    cmp {}, #0\n", c));
                    out.push_str(&format!("    bne {}\n", true_lbl));
                    out.push_str(&format!("    b {}\n", false_lbl));
                }
                IlocOp::Jump(lbl) => {
                    out.push_str(&format!("    b {}\n", lbl));
                }
                IlocOp::Call(callee) => {
                    out.push_str(&format!("    bl {}\n", callee));
                }
                _ => {}
            }
        }

        // Epilogue for main
        out.push_str("    mov sp, r11\n");
        out.push_str("    ldmfd sp, {r4-r11, pc}\n\n");

        // Built-in I/O Subroutines
        out.push_str("putint:\n    swi 4\n    mov pc, lr\n\n");
        out.push_str("putstr:\n    swi 3\n    mov pc, lr\n\n");
        out.push_str("put:\n    swi 0\n    mov pc, lr\n\n");

        // String literals section
        for (lbl, text) in strings {
            out.push_str("    align\n");
            out.push_str(&format!("{}:\n", lbl));
            let bytes: Vec<String> = text
                .bytes()
                .chain(std::iter::once(0))
                .map(|b| format!("{}", b))
                .collect();
            out.push_str(&format!("    defb {}\n", bytes.join(",")));
        }

        out
    }

    fn emit_cmp(&mut self, lhs: &Reg, rhs: &Reg, dst: &Reg, cond: &str, out: &mut String) {
        let l = self.get_arm_reg(lhs);
        let r = self.get_arm_reg(rhs);
        let d = self.get_arm_reg(dst);
        out.push_str(&format!("    cmp {}, {}\n", l, r));
        out.push_str(&format!("    mov {}, #0\n", d));
        out.push_str(&format!("    mov{} {}, #1\n", cond, d));
    }
}
