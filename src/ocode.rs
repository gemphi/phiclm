//! O-code: BCPL's tiny stack intermediate, kept tiny. 55 operations carry
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
    /// Push the address of `TABLE` `i` of the program's tables.
    Table(usize),
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
    /// The floating monads: the word holds an `f64`'s bits. `FIX` rounds
    /// to the nearest integer, halves away from zero (`cintsys.c:2018`).
    Float,
    Fix,
    Fneg,
    Fabs,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Lsh,
    Rsh,
    /// `a EQV b` — every bit where the two agree.
    Eqv,
    /// `a NEQV b` (and `XOR`) — every bit where they differ.
    Neqv,
    /// Pop `i`, pop `s`, push byte `i` of `s`. A byte is a word here: our
    /// strings are one character per word, the length at word 0, so `s%0`
    /// is the length and `bytesperword` is 1.
    Getbyte,
    /// Pop `i`, pop `s`, pop a value; byte `i` of `s` := the value.
    Putbyte,
    Eq,
    Ne,
    Ls,
    Gr,
    Le,
    Ge,
    Fadd,
    Fsub,
    Fmul,
    Fdiv,
    Feq,
    Fne,
    Fls,
    Fgr,
    Fle,
    Fge,
    Jump(usize),
    /// Pop a label's address and go there: the computed GOTO.
    Goto,
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
            Op::Num(_) => "num", Op::Str(_) => "str", Op::Table(_) => "table", Op::Lp(_) => "lp", Op::Sp(_) => "sp", Op::Llp(_) => "llp",
            Op::Lg(_) => "lg", Op::Sg(_) => "sg", Op::Rv => "rv", Op::Stind => "stind", Op::Pop => "pop",
            Op::Neg => "neg", Op::Not => "not", Op::Abs => "abs", Op::Add => "add", Op::Sub => "sub",
            Op::Mul => "mul", Op::Div => "div", Op::Rem => "rem", Op::And => "and", Op::Or => "or",
            Op::Lsh => "lsh", Op::Rsh => "rsh", Op::Eqv => "eqv", Op::Neqv => "neqv",
            Op::Getbyte => "getbyte", Op::Putbyte => "putbyte", Op::Eq => "eq", Op::Ne => "ne", Op::Ls => "ls", Op::Gr => "gr",
            Op::Le => "le", Op::Ge => "ge", Op::Jump(_) => "jump", Op::Jf(_) => "jf", Op::Jt(_) => "jt",
            Op::Float => "float", Op::Fix => "fix", Op::Fneg => "fneg", Op::Fabs => "fabs", Op::Fadd => "fadd",
            Op::Fsub => "fsub", Op::Fmul => "fmul", Op::Fdiv => "fdiv", Op::Feq => "feq", Op::Fne => "fne",
            Op::Fls => "fls", Op::Fgr => "fgr", Op::Fle => "fle", Op::Fge => "fge", Op::Goto => "goto",
            Op::Call { .. } => "call", Op::Fnrn => "fnrn", Op::Rtrn => "rtrn", Op::Finish => "finish",
        }
    }
}

/// A compiled program: the ops, the string table, the `TABLE`s, the global
/// vector's names, each compiled function's `(slot, entry)`, and the
/// `STATIC` cells' initial `(slot, value)`.
#[derive(Debug, Default)]
pub struct Program {
    pub op: Vec<Op>,
    pub string: Vec<String>,
    pub table: Vec<Vec<i64>>,
    pub global: std::collections::HashMap<String, usize>,
    pub entry: Vec<(usize, usize)>,
    pub init: Vec<(usize, i64)>,
}
