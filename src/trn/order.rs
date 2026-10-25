//! Commands to O-code: everything leaves the stack as it found it.

use super::Trn;
use crate::ocode::Op;
use crate::tree::{Command, Declaration, Element, Expression};

impl Trn {
    pub(super) fn command(&mut self, c: &Command) -> Result<(), String> {
        match c {
            Command::Block(body) => {
                let keep = self.scope.len();
                for e in body {
                    match e {
                        Element::Do(c) => self.command(c)?,
                        Element::Let(run) => {
                            for d in run {
                                match d {
                                    Declaration::Variable(name, e) => {
                                        self.expression(e)?;
                                        let at = self.free;
                                        self.free += 1;
                                        self.scope.push((name.clone(), at));
                                        self.emit(Op::Sp(at));
                                    }
                                    Declaration::Manifest(rows) => self.constant.extend(rows.iter().cloned()),
                                    other => return Err(format!("not a v1 local declaration: {other:?}")),
                                }
                            }
                        }
                    }
                }
                self.scope.truncate(keep);
            }
            Command::Assign(lhs, e) => {
                self.expression(e)?;
                match lhs {
                    Expression::Name(n) => match (self.local(n), self.program.global.get(n).copied()) {
                        (Some(at), _) => {
                            self.emit(Op::Sp(at));
                        }
                        (_, Some(slot)) => {
                            self.emit(Op::Sg(slot));
                        }
                        _ => return Err(format!("assignment to unknown name {n}")),
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
        }
        Ok(())
    }

    /// A head-tested (or untested) loop: LOOP returns to the head.
    fn headed(&mut self, head: Option<(&Expression, bool)>, body: &Command) -> Result<(), String> {
        let top = self.here();
        let mut out = None;
        if let Some((q, flip)) = head {
            self.expression(q)?;
            out = Some(self.emit(if flip { Op::Jt(0) } else { Op::Jf(0) }));
        }
        self.looping.push((Vec::new(), Vec::new()));
        self.command(body)?;
        self.emit(Op::Jump(top));
        let (breaks, loops) = self.looping.pop().expect("pushed");
        if let Some(at) = out {
            self.patch(at);
        }
        for at in breaks {
            self.patch(at);
        }
        for at in loops {
            self.program.op[at] = Op::Jump(top);
        }
        Ok(())
    }

    /// A tail-tested loop (`c REPEATWHILE e`): LOOP returns to the test.
    fn tailed(&mut self, body: &Command, q: &Expression, flip: bool) -> Result<(), String> {
        let top = self.here();
        self.looping.push((Vec::new(), Vec::new()));
        self.command(body)?;
        let test = self.here();
        self.expression(q)?;
        self.emit(if flip { Op::Jf(top) } else { Op::Jt(top) });
        let (breaks, loops) = self.looping.pop().expect("pushed");
        for at in breaks {
            self.patch(at);
        }
        for at in loops {
            self.program.op[at] = Op::Jump(test);
        }
        Ok(())
    }

    /// `FOR v = a TO b BY k`: the limit held in a hidden slot, LOOP returns
    /// to the increment.
    fn counted(&mut self, var: &str, from: &Expression, to: &Expression, by: Option<&Expression>, body: &Command) -> Result<(), String> {
        let step = match by {
            None => 1,
            Some(e) => self.fold(e).ok_or("FOR … BY needs a constant in v1")?,
        };
        self.expression(from)?;
        let v = self.free;
        self.free += 1;
        self.scope.push((var.to_string(), v));
        self.emit(Op::Sp(v));
        self.expression(to)?;
        let lim = self.free;
        self.free += 1;
        self.emit(Op::Sp(lim));
        let top = self.here();
        self.emit(Op::Lp(v));
        self.emit(Op::Lp(lim));
        self.emit(if step >= 0 { Op::Le } else { Op::Ge });
        let out = self.emit(Op::Jf(0));
        self.looping.push((Vec::new(), Vec::new()));
        self.command(body)?;
        let (breaks, loops) = self.looping.pop().expect("pushed");
        let incr = self.here();
        self.emit(Op::Lp(v));
        self.emit(Op::Num(step));
        self.emit(Op::Add);
        self.emit(Op::Sp(v));
        self.emit(Op::Jump(top));
        self.patch(out);
        for at in breaks {
            self.patch(at);
        }
        for at in loops {
            self.program.op[at] = Op::Jump(incr);
        }
        self.scope.pop();
        Ok(())
    }
}
