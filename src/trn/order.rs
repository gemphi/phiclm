//! Commands to O-code: everything leaves the stack as it found it.

use super::{Binding, Trn};
use crate::ocode::Op;
use crate::tree::{Command, Element, Expression};

impl Trn {
    pub(super) fn command(&mut self, c: &Command) -> Result<(), String> {
        match c {
            Command::Block(body) => {
                let keep = self.scope.len();
                for e in body {
                    match e {
                        Element::Do(c) => self.command(c)?,
                        Element::Let(run) => self.declare(run)?,
                    }
                }
                self.scope.truncate(keep);
            }
            Command::Assign(Expression::Dyad("of", f, p), e) => self.store_field(f, p, e)?,
            Command::Assign(lhs, e) => {
                self.expression(e)?;
                match lhs {
                    Expression::Name(n) => match self.lookup(n) {
                        Some(Binding::Local(at)) => {
                            self.emit(Op::Sp(at));
                        }
                        Some(Binding::Cell(slot)) => {
                            self.emit(Op::Sg(slot));
                        }
                        Some(Binding::Const(_) | Binding::Label) => return Err(format!("{n} is a constant: it cannot be assigned")),
                        None => return Err(format!("assignment to unknown name {n}")),
                    },
                    Expression::Dyad("!", a, b) => {
                        self.expression(a)?;
                        self.expression(b)?;
                        self.emit(Op::Add);
                        self.emit(Op::Stind);
                    }
                    Expression::Monad("!", a) => {
                        self.expression(a)?;
                        self.emit(Op::Stind);
                    }
                    Expression::Dyad("%", a, b) => {
                        self.expression(a)?;
                        self.expression(b)?;
                        self.emit(Op::Putbyte);
                    }
                    other => return Err(format!("not an lvalue: {other:?}")),
                }
            }
            Command::Do(e) => {
                self.expression(e)?;
                self.emit(Op::Pop);
            }
            Command::If(q, c) => {
                self.expression(q)?;
                let end = self.emit(Op::Jf(0));
                self.command(c)?;
                self.patch(end);
            }
            Command::Unless(q, c) => {
                self.expression(q)?;
                let end = self.emit(Op::Jt(0));
                self.command(c)?;
                self.patch(end);
            }
            Command::Test(q, yes, no) => {
                self.expression(q)?;
                let to_no = self.emit(Op::Jf(0));
                self.command(yes)?;
                let to_end = self.emit(Op::Jump(0));
                self.patch(to_no);
                self.command(no)?;
                self.patch(to_end);
            }
            Command::While(q, body) => self.headed(Some((q, false)), body)?,
            Command::Until(q, body) => self.headed(Some((q, true)), body)?,
            Command::Repeat(body) => self.headed(None, body)?,
            Command::RepeatWhile(body, q) => self.tailed(body, q, false)?,
            Command::RepeatUntil(body, q) => self.tailed(body, q, true)?,
            Command::For(var, from, to, by, body) => self.counted(var, from, to, by.as_ref(), body)?,
            Command::Resultis(e) => {
                self.expression(e)?;
                if self.valof.is_empty() {
                    self.emit(Op::Fnrn);
                } else {
                    let at = self.emit(Op::Jump(0));
                    self.valof.last_mut().expect("checked").push(at);
                }
            }
            Command::Return => {
                self.emit(Op::Rtrn);
            }
            Command::Finish => {
                self.emit(Op::Finish);
            }
            Command::Break | Command::Loop => {
                let at = self.here();
                let (breaks, loops) = self.looping.last_mut().ok_or("BREAK/LOOP outside a loop")?;
                if matches!(c, Command::Break) { breaks.push(at) } else { loops.push(at) }
                self.emit(Op::Jump(0));
            }
            Command::Switchon(e, body) => self.switch(e, body)?,
            Command::Case(k, body) => {
                let k = self.fold(k).ok_or_else(|| format!("CASE {k:?}: not a constant"))?;
                let at = self.here();
                let sw = self.switching.last_mut().ok_or("CASE label out of context")?;
                if sw.case.iter().any(|(c, _)| *c == k) {
                    return Err(format!("'CASE {k}:' occurs twice"));
                }
                sw.case.push((k, at));
                self.command(body)?;
            }
            Command::Default(body) => {
                let at = self.here();
                let sw = self.switching.last_mut().ok_or("DEFAULT label out of context")?;
                if sw.default.replace(at).is_some() {
                    return Err("Bad DEFAULT label: it occurs twice".into());
                }
                self.command(body)?;
            }
            Command::Endcase => {
                let at = self.here();
                self.switching.last_mut().ok_or("Illegal use of ENDCASE")?.out.push(at);
                self.emit(Op::Jump(0));
            }
            Command::Goto(e) => match (e, self.lookup_name(e)) {
                (Expression::Name(label), Some(Binding::Label)) => {
                    let at = self.emit(Op::Jump(0));
                    self.goto.push((at, label.clone()));
                }
                (Expression::Name(label), None) => return Err(format!("GOTO {label}: no such label in this routine")),
                (e, _) => {
                    // A computed GOTO: the value is a label's address.
                    self.expression(e)?;
                    self.emit(Op::Goto);
                }
            },
            Command::Label(label, body) => {
                let at = self.here();
                if self.label.insert(label.clone(), at).is_some() {
                    return Err(format!("label {label} is set twice"));
                }
                self.command(body)?;
            }
        }
        Ok(())
    }

    pub(super) fn lookup_name(&self, e: &Expression) -> Option<Binding> {
        match e {
            Expression::Name(n) => self.lookup(n),
            _ => None,
        }
    }
}
