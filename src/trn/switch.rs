//! `SWITCHON`: the value kept, the body laid down, then the dispatch.

use super::{Switch, Trn};
use crate::ocode::Op;
use crate::tree::{Command, Expression};

impl Trn {
    /// `SWITCHON e INTO body`: the value kept in a hidden slot, the body
    /// laid down with its CASE labels, then the dispatch — one test per
    /// CASE, the DEFAULT (or the end) when none matches. ENDCASE and
    /// falling off the body both reach the end.
    pub(super) fn switch(&mut self, e: &Expression, body: &Command) -> Result<(), String> {
        self.expression(e)?;
        let v = self.free;
        self.free += 1;
        self.emit(Op::Sp(v));
        let dispatch = self.emit(Op::Jump(0));
        self.switching.push(Switch::default());
        self.command(body)?;
        let off_end = self.emit(Op::Jump(0));
        let sw = self.switching.pop().expect("pushed");
        self.patch(dispatch);
        for (k, at) in &sw.case {
            self.emit(Op::Lp(v));
            self.emit(Op::Num(*k));
            self.emit(Op::Eq);
            self.emit(Op::Jt(*at));
        }
        let none = self.emit(Op::Jump(sw.default.unwrap_or(0)));
        self.patch(off_end);
        for at in sw.out {
            self.patch(at);
        }
        if sw.default.is_none() {
            self.patch(none);
        }
        Ok(())
    }
}
