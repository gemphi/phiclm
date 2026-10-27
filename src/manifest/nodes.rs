//! Every tree node's rung: the node `rbexp`, `rdcom` and `rdef` build for
//! it, so a parsed program is read by group the way the header reads it.

use super::ops::dyad;
use super::S;

impl crate::tree::Expression {
    /// The expression's rung: the node `rbexp`/`rnexp` builds for it.
    pub fn kind(&self) -> S {
        use crate::tree::Expression;
        match self {
            Expression::Num(_) => S::Number,
            Expression::Fnum(_) => S::Fnum,
            Expression::Name(_) => S::Name,
            Expression::Str(_) => S::String,
            Expression::True => S::True,
            Expression::False => S::False,
            Expression::Monad(op, _) => match *op {
                "-" => S::Neg,
                "!" => S::Rv,
                "@" => S::Lv,
                "~" => S::Not,
                "abs" => S::Abs,
                "float" => S::Float,
                "fix" => S::Fix,
                "#-" => S::Fneg,
                "#abs" => S::Fabs,
                _ => S::None,
            },
            Expression::Dyad(op, ..) => dyad(op),
            Expression::Cond(..) => S::Cond,
            Expression::Call(..) => S::Fnap,
            Expression::Valof(_) => S::Valof,
            // `LET v = VEC k` is BCPL's `s_vecdef`: a vector is declared, not computed.
            Expression::Vec(_) => S::Vecdef,
            Expression::Table(_) => S::Table,
            Expression::Slct(..) => S::Slct,
        }
    }
}

impl crate::tree::Command {
    /// The command's rung: the node `rdcom`/`rbcom` builds for it. A block is
    /// `s_seq`, a routine call `s_rtap`, a label `s_colon`.
    pub fn kind(&self) -> S {
        use crate::tree::Command;
        match self {
            Command::Block(_) => S::Seq,
            Command::Assign(..) => S::Ass,
            Command::Do(_) => S::Rtap,
            Command::If(..) => S::If,
            Command::Unless(..) => S::Unless,
            Command::Test(..) => S::Test,
            Command::While(..) => S::While,
            Command::Until(..) => S::Until,
            Command::Repeat(_) => S::Repeat,
            Command::RepeatWhile(..) => S::Repeatwhile,
            Command::RepeatUntil(..) => S::Repeatuntil,
            Command::For(..) => S::For,
            Command::Resultis(_) => S::Resultis,
            Command::Return => S::Return,
            Command::Finish => S::Finish,
            Command::Break => S::Break,
            Command::Loop => S::Loop,
            Command::Switchon(..) => S::Switchon,
            Command::Case(..) => S::Case,
            Command::Default(_) => S::Default,
            Command::Endcase => S::Endcase,
            Command::Goto(_) => S::Goto,
            Command::Label(..) => S::Colon,
        }
    }
}

impl crate::tree::Element {
    pub fn kind(&self) -> S {
        use crate::tree::Element;
        match self {
            Element::Let(_) => S::Let,
            Element::Do(c) => c.kind(),
        }
    }
}

impl crate::tree::Declaration {
    /// The declaration's rung: `s_fndef`, `s_rtdef`, `s_valdef`, and the
    /// three declaration blocks.
    pub fn kind(&self) -> S {
        use crate::tree::Declaration;
        match self {
            Declaration::Function(..) => S::Fndef,
            Declaration::Routine(..) => S::Rtdef,
            Declaration::Variable(..) => S::Valdef,
            Declaration::Manifest(_) => S::Manifest,
            Declaration::Global(_) => S::Global,
            Declaration::Static(_) => S::Static,
        }
    }
}
