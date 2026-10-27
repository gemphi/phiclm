//! The ONE ladder — BCPL's `g/bcplfecg.h` as ours. The lexer's tokens, the
//! syntax tree's nodes and the OCODE operators share a single numbering
//! (`s_number=1; s_name; s_string …`), so a kind IS its number at every
//! level, a GROUP is a contiguous range of it, and a kind that must drill
//! down carries its own sub-enum (`sf_*` under FLTOP and the assignment
//! ops). Read from the source, not invented; the order below is the
//! header's order, because the order is the hierarchy.
//!
//! phiclm grew up with six Rust enums (Token, Expression, Command, Element,
//! Declaration, Op). Each now names its rung: `kind()` (and `S::from` for
//! an op) places every variant on this ladder, so a tree is read by group
//! the way the header reads it — and it is the same ladder the language
//! model's kinds will climb: a band selects a group, a group is a range.

/// The manifest, in the header's order; the discriminant is the code.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum S {
    Number = 1, Name, String, True, False,
    Valof, Lv, Rv, Vecap, Fnap,
    Query,
    Neg, Abs,
    Mul, Div, Rem, Add, Sub,
    Eq, Ne, Ls, Gr, Le, Ge,
    Slct, Of,
    Byteap, Mthap,
    Not, Lshift, Rshift, Logand, Logor,
    Eqv, Neqv, Cond, Comma, Table,
    Needs, Section,
    Ass,
    Rtap, Goto, Resultis, Colon,
    Test, For, If, Unless,
    While, Until, Repeat, Repeatwhile, Repeatuntil,
    Skip,
    Loop, Break, Return, Finish,
    Endcase, Switchon, Case, Default,
    Seq, Let, And, Manifest, Global, Static,
    Valdef, Vecdef, Constdef, Const,
    Fndef, Rtdef, Local, Label,
    // other lexical tokens
    Be, Lsect, Rsect, Get,
    Semicolon, Into, To, By, Do, Else,
    Vec, Lparen, Rparen, Sbra, Sket, Dot, Eof,
    Bitsperbcplword,
    None,
    // OCODE operators
    Lf, Lp, Lg, Ln, Lstr, Ll, Llp, Llg, Lll,
    Sp, Sg, Sl, Stind, Jump, Jt, Jf, Endfor,
    Lab, Stack, Store, Rstack, Entry,
    Save, Fnrn, Rtrn, Res, Datalab, Itemn,
    Endproc, Getbyte, Putbyte,
    // the extended compiler's: floating point and the assignment operators
    // (added 15/07/10), the field ops (19/07/10), FLTOP
    Fnum,
    Float, Fix, Fabs,
    Fmul, Fdiv, Fadd, Fsub, Fpos, Fneg,
    Feq, Fne, Fls, Fgr, Fle, Fge,
    Assvecap,
    Assmul, Assdiv, Assrem, Assadd, Asssub,
    Assfmul, Assfdiv, Assfadd, Assfsub,
    Asslshift, Assrshift,
    Asslogand, Asslogor, Asseqv, Assneqv,
    Selld, Selst,
    Fltop,
}

/// A GROUP is a contiguous range of the ladder: the hierarchy is the order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Group {
    pub name: &'static str,
    pub first: S,
    pub last: S,
}

/// The groups the header's layout draws, outermost first.
pub const GROUPS: &[Group] = &[
    Group { name: "constant", first: S::Number, last: S::False },
    Group { name: "access", first: S::Valof, last: S::Query },
    Group { name: "monadic", first: S::Neg, last: S::Abs },
    Group { name: "arithmetic", first: S::Mul, last: S::Sub },
    Group { name: "relation", first: S::Eq, last: S::Ge },
    Group { name: "selection", first: S::Slct, last: S::Mthap },
    Group { name: "logic", first: S::Not, last: S::Table },
    Group { name: "linkage", first: S::Needs, last: S::Section },
    Group { name: "command", first: S::Ass, last: S::Default },
    Group { name: "declaration", first: S::Seq, last: S::Label },
    Group { name: "lexical", first: S::Be, last: S::None },
    Group { name: "ocode", first: S::Lf, last: S::Putbyte },
    Group { name: "floating", first: S::Fnum, last: S::Fge },
    Group { name: "assignment", first: S::Assvecap, last: S::Assneqv },
    Group { name: "extended", first: S::Selld, last: S::Fltop },
];

impl S {
    pub fn code(self) -> u16 {
        self as u16
    }

    /// The group a code belongs to — a kind is a member of a range.
    pub fn group(self) -> &'static str {
        GROUPS.iter().find(|g| g.first <= self && self <= g.last).map_or("?", |g| g.name)
    }

    /// `opname` — the readout the compiler keeps per level.
    pub fn name(self) -> String {
        format!("s_{:?}", self).to_lowercase()
    }
}

/// The sub-enum a kind carries to drill down: `sf_*`, the assignment
/// operators under FLTOP and the `ass*` kinds. A kind's own enum, inside it.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Sf {
    None = 0, Vecap, Fmul, Fdiv, Fadd, Fsub, Mul, Div, Rem, Add, Sub, Lshift, Rshift, Logand, Logor, Eqv, Neqv,
}

mod nodes;
mod ops;
mod tokens;

#[cfg(test)]
mod tests;
