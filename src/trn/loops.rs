//! The loops: head-tested, tail-tested and counted, each patching its
//! BREAKs and LOOPs when it closes.

use super::{Binding, Trn};
use crate::ocode::Op;
use crate::tree::{Command, Expression};

impl Trn {
    /// A head-tested (or untested) loop: LOOP returns to the head.
    pub(super) fn headed(&mut self, head: Option<(&Expression, bool)>, body: &Command) -> Result<(), String> {
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
    pub(super) fn tailed(&mut self, body: &Command, q: &Expression, flip: bool) -> Result<(), String> {
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
    pub(super) fn counted(&mut self, var: &str, from: &Expression, to: &Expression, by: Option<&Expression>, body: &Command) -> Result<(), String> {
        let step = match by {
            None => 1,
            Some(e) => self.fold(e).ok_or("FOR … BY needs a constant in v1")?,
        };
        self.expression(from)?;
        let v = self.free;
        self.free += 1;
        self.scope.push((var.to_string(), Binding::Local(v)));
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
