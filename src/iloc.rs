#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reg {
    Virtual(usize),
    R0,
    FP,
    SP,
}

impl std::fmt::Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reg::Virtual(n) => write!(f, "%r{}", n),
            Reg::R0 => write!(f, "%r0"),
            Reg::FP => write!(f, "%fp"),
            Reg::SP => write!(f, "%sp"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IlocOp {
    LoadI(i64, Reg),
    LoadLabel(String, Reg),
    LoadAI {
        base: Reg,
        offset: i32,
        dst: Reg,
    },
    StoreAI {
        src: Reg,
        offset: i32,
        base: Reg,
    },
    Add(Reg, Reg, Reg),
    Sub(Reg, Reg, Reg),
    Mul(Reg, Reg, Reg),
    Div(Reg, Reg, Reg),
    Mod(Reg, Reg, Reg),
    CmpEq(Reg, Reg, Reg),
    CmpNe(Reg, Reg, Reg),
    CmpLt(Reg, Reg, Reg),
    CmpLe(Reg, Reg, Reg),
    CmpGt(Reg, Reg, Reg),
    CmpGe(Reg, Reg, Reg),
    Cbr {
        cond: Reg,
        true_lbl: String,
        false_lbl: String,
    },
    Jump(String),
    Call(String),
    Label(String),
}

#[derive(Debug, Default)]
pub struct IlocProgram {
    pub ops: Vec<IlocOp>,
}

impl IlocProgram {
    pub fn emit(&mut self, op: IlocOp) {
        self.ops.push(op);
    }
}
