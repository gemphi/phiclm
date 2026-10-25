//! O-code: BCPL's tiny stack intermediate, kept tiny. ~35 operations carry
//! the whole language — the fifty-year-old proof (docs/169 §1) that a small
//! intermediate beats a big target, and the same bet `Row` makes.

/// One operation. `Lp`/`Sp` work frame slots (the P pointer), `Lg`/`Sg`
/// the global vector, `Rv`/`Stind` the one linear store (`!`), `Llp` an
/// address in the frame (`@`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Num(i64),
    /// Push the address of string `i` of the program's string table.
    Str(usize),
    Lp(usize),
    Sp(usize),
    Llp(usize),
    Lg(usize),
    Sg(usize),
    /// `A := mem[A]` — monadic `!`.
    Rv,
    /// Pop an address, pop a value, `mem[address] := value`.
    Stind,
    Pop,
    Neg,
    Not,
    Abs,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Lsh,
    Rsh,
    Eq,
    Ne,
    Ls,
    Gr,
    Le,
    Ge,
    Jump(usize),
    Jf(usize),
    Jt(usize),
    /// Pop `args` arguments then the entry; new frame `frame` words up.
    Call { args: usize, frame: usize },
    /// Return from a function: the value is on the stack.
    Fnrn,
    /// Return from a routine: a 0 is pushed for uniformity.
    Rtrn,
    Finish,
}

impl Op {
    /// The mnemonic, for telling the compile as rows.
    pub fn name(&self) -> &'static str {
        match self {
            Op::Num(_) => "num", Op::Str(_) => "str", Op::Lp(_) => "lp", Op::Sp(_) => "sp", Op::Llp(_) => "llp",
            Op::Lg(_) => "lg", Op::Sg(_) => "sg", Op::Rv => "rv", Op::Stind => "stind", Op::Pop => "pop",
            Op::Neg => "neg", Op::Not => "not", Op::Abs => "abs", Op::Add => "add", Op::Sub => "sub",
            Op::Mul => "mul", Op::Div => "div", Op::Rem => "rem", Op::And => "and", Op::Or => "or",
            Op::Lsh => "lsh", Op::Rsh => "rsh", Op::Eq => "eq", Op::Ne => "ne", Op::Ls => "ls", Op::Gr => "gr",
            Op::Le => "le", Op::Ge => "ge", Op::Jump(_) => "jump", Op::Jf(_) => "jf", Op::Jt(_) => "jt",
            Op::Call { .. } => "call", Op::Fnrn => "fnrn", Op::Rtrn => "rtrn", Op::Finish => "finish",
        }
    }
}

/// A compiled program: the ops, the string table, the global vector's
/// names, and each compiled function's `(slot, entry)`.
#[derive(Debug, Default)]
pub struct Program {
    pub op: Vec<Op>,
    pub string: Vec<String>,
    pub global: std::collections::HashMap<String, usize>,
    pub entry: Vec<(usize, usize)>,
}
