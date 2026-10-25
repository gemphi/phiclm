//! Expressions to O-code: everything leaves exactly one value on the stack.

use super::Trn;
use crate::ocode::Op;
use crate::tree::Expression;

impl Trn {
    pub(super) fn expression(&mut self, e: &Expression) -> Result<(), String> {
        match e {
            Expression::Num(n) => {
                self.emit(Op::Num(*n));
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
            Expression::Name(n) => match (self.constant.get(n), self.local(n), self.program.global.get(n)) {
                (Some(k), _, _) => {
                    let k = *k;
                    self.emit(Op::Num(k));
                }
                (_, Some(at), _) => {
                    self.emit(Op::Lp(at));
                }
                (_, _, Some(slot)) => {
                    let slot = *slot;
                    self.emit(Op::Lg(slot));
                }
                _ => return Err(format!("unknown name {n}")),
            },
            Expression::Monad(op, a) => {
                if *op == "@" {
                    return match a.as_ref() {
                        Expression::Name(n) => match self.local(n) {
                            Some(at) => {
                                self.emit(Op::Llp(at));
                                Ok(())
                            }
                            None => Err(format!("@{n}: only a local has an address in v1")),
                        },
                        other => Err(format!("@ of {other:?} not in v1")),
                    };
                }
                self.expression(a)?;
                self.emit(match *op {
                    "-" => Op::Neg,
                    "!" => Op::Rv,
                    "~" => Op::Not,
                    "abs" => Op::Abs,
                    other => return Err(format!("monad {other}?")),
                });
            }
            Expression::Dyad(op, a, b) => {
                self.expression(a)?;
                self.expression(b)?;
                match *op {
                    "!" => {
                        self.emit(Op::Add);
                        self.emit(Op::Rv);
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
                let at = self.free;
                self.free += *k as usize + 1;
                self.emit(Op::Llp(at));
            }
        }
        Ok(())
    }
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
        "=" => Op::Eq,
        "~=" => Op::Ne,
        "<" => Op::Ls,
        ">" => Op::Gr,
        "<=" => Op::Le,
        ">=" => Op::Ge,
        other => return Err(format!("dyad {other}?")),
    })
}
