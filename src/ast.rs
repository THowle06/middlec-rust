#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Char,
    Bool,
    Void,
    Pointer(Box<Type>),
    Array(Box<Type>, usize),
    Struct(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
    Deref,
    AddressOf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLit(i64),
    CharLit(char),
    BoolLit(bool),
    StringLit(String),
    Var(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Ternary {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub ty: Type,
    pub name: String,
    pub init: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Var(VarDecl),
    Expr(Expr),
    Block(Vec<Stmt>),
    If {
        cond: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        step: Option<Expr>,
        body: Box<Stmt>,
    },
    Return(Option<Expr>),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncDef {
    pub return_ty: Type,
    pub name: String,
    pub params: Vec<(Type, String)>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProgramUnit {
    Var(VarDecl),
    Func(FuncDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub units: Vec<ProgramUnit>,
}
