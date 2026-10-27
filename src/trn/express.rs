//! Expressions to O-code: everything leaves exactly one value on the stack.

use super::{Binding, Trn};
use crate::ocode::Op;
use crate::tree::Expression;

impl Trn {
    pub(super) fn expression(&mut self, e: &Expression) -> Result<(), String> {
        match e {
            Expression::Num(n) => {
                self.emit(Op::Num(*n));
            }
            Expression::Fnum(f) => {
                self.emit(Op::Num(f.to_bits() as i64));
            }
            Expression::True => {
                self.emit(Op::Num(-1));
            }
            Expression::False => {
                self.emit(Op::Num(0));
            }
            Expression::Str(s) => {
                let i = self.program.string.iter().position(|x| x == s).unwrap_or_else(|| {
                    self.program.string.push(s.clone());
                    self.program.string.len() - 1
                });
                self.emit(Op::Str(i));
            }
            Expression::Name(n) => match self.lookup(n) {
                Some(Binding::Local(at)) => {
                    self.emit(Op::Lp(at));
                }
                Some(Binding::Const(k)) => {
                    self.emit(Op::Num(k));
                }
                Some(Binding::Cell(slot)) => {
                    self.emit(Op::Lg(slot));
                }
                Some(Binding::Label) => {
                    let at = self.emit(Op::Num(0));
                    self.label_value.push((at, n.clone()));
                }
                None => return Err(format!("unknown name {n}")),
            },
            Expression::Monad("@", a) => self.address(a)?,
            Expression::Monad(op, a) => {
                self.expression(a)?;
                self.emit(monad(op)?);
            }
            Expression::Dyad("of", f, p) => self.field(f, p)?,
            Expression::Dyad(op, a, b) => {
                self.expression(a)?;
                self.expression(b)?;
                match *op {
                    "!" => {
                        self.emit(Op::Add);
                        self.emit(Op::Rv);
                    }
                    "%" => {
                        self.emit(Op::Getbyte);
                    }
                    other => {
                        self.emit(dyad(other)?);
                    }
                }
            }
            Expression::Cond(q, yes, no) => {
                self.expression(q)?;
                let to_no = self.emit(Op::Jf(0));
                self.expression(yes)?;
                let to_end = self.emit(Op::Jump(0));
                self.patch(to_no);
                self.expression(no)?;
                self.patch(to_end);
            }
            Expression::Call(f, args) => {
                self.expression(f)?;
                for a in args {
                    self.expression(a)?;
                }
                let frame = self.free;
                self.emit(Op::Call { args: args.len(), frame });
            }
            Expression::Valof(c) => {
                self.valof.push(Vec::new());
                self.command(c)?;
                self.emit(Op::Num(0)); // fell off the end without RESULTIS
                for at in self.valof.pop().unwrap_or_default() {
                    self.patch(at);
                }
            }
            Expression::Vec(k) => {
                let k = self.fold(k).filter(|k| *k >= 0).ok_or_else(|| format!("VEC needs a constant size, found {k:?}"))?;
                let at = self.free;
                self.free += k as usize + 1;
                self.emit(Op::Llp(at));
            }
            Expression::Table(items) => {
                let words = items.iter().map(|e| self.fold(e).ok_or_else(|| format!("TABLE needs constants, found {e:?}"))).collect::<Result<Vec<_>, _>>()?;
                self.program.table.push(words);
                self.emit(Op::Table(self.program.table.len() - 1));
            }
            Expression::Slct(..) => {
                let k = self.fold(e).ok_or("A field too large, or not constant, in a SLCT expression")?;
                self.emit(Op::Num(k));
            }
        }
        Ok(())
    }

    /// `@e`: where a local, a global or static cell, or a word reached by
    /// `!` lives (`@(v!i)` is `v + i`, `@!p` is `p`).
    fn address(&mut self, e: &Expression) -> Result<(), String> {
        match (e, e.name().and_then(|n| self.lookup(n))) {
            (_, Some(Binding::Local(at))) => {
                self.emit(Op::Llp(at));
            }
            (_, Some(Binding::Cell(slot))) => {
                self.emit(Op::Num(slot as i64));
            }
            (Expression::Name(n), _) => return Err(format!("@{n}: not a variable")),
            (Expression::Dyad("!", a, b), _) => {
                self.expression(a)?;
                self.expression(b)?;
                self.emit(Op::Add);
            }
            (Expression::Monad("!", a), _) => self.expression(a)?,
            (other, _) => return Err(format!("@ of {other:?}: it has no address")),
        }
        Ok(())
    }
}

impl Expression {
    fn name(&self) -> Option<&str> {
        match self {
            Expression::Name(n) => Some(n),
            _ => None,
        }
    }
}

/// The monadic operator's op.
pub(super) fn monad(op: &str) -> Result<Op, String> {
    Ok(match op {
        "-" => Op::Neg,
        "!" => Op::Rv,
        "~" => Op::Not,
        "abs" => Op::Abs,
        "float" => Op::Float,
        "fix" => Op::Fix,
        "#-" => Op::Fneg,
        "#abs" => Op::Fabs,
        other => return Err(format!("monad {other}?")),
    })
}

/// The dyadic operator's op.
pub(super) fn dyad(op: &str) -> Result<Op, String> {
    Ok(match op {
        "+" => Op::Add,
        "-" => Op::Sub,
        "*" => Op::Mul,
        "/" => Op::Div,
        "rem" => Op::Rem,
        "&" => Op::And,
        "|" => Op::Or,
        "<<" => Op::Lsh,
        ">>" => Op::Rsh,
        "eqv" => Op::Eqv,
        "neqv" => Op::Neqv,
        "=" => Op::Eq,
        "~=" => Op::Ne,
        "<" => Op::Ls,
        ">" => Op::Gr,
        "<=" => Op::Le,
        ">=" => Op::Ge,
        "#+" => Op::Fadd,
        "#-" => Op::Fsub,
        "#*" => Op::Fmul,
        "#/" => Op::Fdiv,
        "#=" => Op::Feq,
        "#~=" => Op::Fne,
        "#<" => Op::Fls,
        "#>" => Op::Fgr,
        "#<=" => Op::Fle,
        "#>=" => Op::Fge,
        other => return Err(format!("dyad {other}?")),
    })
}
