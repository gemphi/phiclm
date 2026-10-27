//! The routines that move the machine itself (`blib.b:79–81`, `1262–1349`):
//! the coroutines, `level` and `longjump`, `stop` and `abort`. A transfer
//! saves the running context in its coroutine and resumes another, handing
//! it a value as the result of the call it stopped in.

use super::context::{Co, Context, Flow};
use super::{Machine, OUTSIDE_CO};
use crate::global::Control;
use crate::ocode::Program;

impl Machine {
    fn co_index(&self, cptr: i64) -> Result<usize, String> {
        self.co.iter().position(|c| c.cptr == cptr).ok_or_else(|| format!("{cptr} is not a coroutine"))
    }

    /// `changeco`: the current context is kept (unless it has finished),
    /// `target` runs, and `value` is what it receives.
    fn transfer(&mut self, target: i64, value: i64, cx: &mut Context, keep: bool, program: &Program) -> Result<Flow, String> {
        let cur = self.co_index(self.current)?;
        let to = self.co_index(target)?;
        if keep {
            self.co[cur].saved = Some(std::mem::take(cx));
        }
        self.current = target;
        match self.co[to].saved.take() {
            Some(mut ctx) => {
                ctx.stack.push(value);
                *cx = ctx;
            }
            None => {
                // A fresh coroutine runs its body on its own stack.
                let (base, body, size) = (self.co[to].cptr, self.co[to].body, self.co[to].size);
                if body <= 0 || body as usize >= program.op.len() {
                    return Err("a coroutine's body must be a BCPL function".into());
                }
                self.set(base, OUTSIDE_CO)?;
                self.set(base + 1, OUTSIDE_CO)?;
                self.set(base + 2, value)?;
                *cx = Context { p: base, pc: body as usize, stack: Vec::new(), frames: vec![(base, 0)], end: Some(base + size) };
            }
        }
        Ok(Flow::Go)
    }

    /// A coroutine's body returned `v`: as BLIB's `c := fn(cowait(c))
    /// REPEAT`, `v` goes to its parent and the coroutine is fresh again.
    pub(super) fn finished(&mut self, v: i64, cx: &mut Context, program: &Program) -> Result<Flow, String> {
        let cur = self.co_index(self.current)?;
        let parent = std::mem::replace(&mut self.co[cur].parent, 0);
        match parent {
            0 => Err("a coroutine returned with no parent to return to".into()),
            p => self.transfer(p, v, cx, false, program),
        }
    }

    pub(super) fn control(&mut self, c: Control, arg: &[i64], cx: &mut Context, program: &Program) -> Result<Flow, String> {
        let a = |i: usize| arg.get(i).copied().unwrap_or(0);
        match c {
            Control::Createco => {
                let base = self.getvec(a(1));
                if base != 0 {
                    self.co.push(Co { cptr: base, parent: 0, body: a(0), size: a(1), saved: None });
                }
                cx.stack.push(base);
            }
            Control::Deleteco => {
                let at = self.co_index(a(0))?;
                if at == 0 || self.co[at].parent != 0 {
                    return Err(format!("cannot deleteco {}: it is running or has a parent", a(0)));
                }
                self.co.remove(at);
                self.freevec(a(0));
                cx.stack.push(-1);
            }
            Control::Callco => {
                let to = self.co_index(a(0))?;
                if self.co[to].parent != 0 {
                    return Err(format!("callco {}: it already has a parent (abort 110)", a(0)));
                }
                self.co[to].parent = self.current;
                return self.transfer(a(0), a(1), cx, true, program);
            }
            Control::Resumeco => {
                let (cur, to) = (self.co_index(self.current)?, self.co_index(a(0))?);
                if self.co[to].parent != 0 {
                    return Err(format!("resumeco {}: it already has a parent (abort 111)", a(0)));
                }
                self.co[to].parent = std::mem::replace(&mut self.co[cur].parent, 0);
                return self.transfer(a(0), a(1), cx, true, program);
            }
            Control::Cowait => {
                let cur = self.co_index(self.current)?;
                let parent = std::mem::replace(&mut self.co[cur].parent, 0);
                return match (parent, cur) {
                    // The main program waits on the CLI: it stops with `a`.
                    (0, 0) => Ok(Flow::Done(a(0))),
                    (0, _) => Err("cowait: this coroutine has no parent".into()),
                    (p, _) => self.transfer(p, a(0), cx, true, program),
                };
            }
            Control::Level => cx.stack.push(cx.p),
            Control::Longjump => {
                let (lev, lab) = (a(0), a(1));
                if lab <= 0 || lab as usize >= program.op.len() {
                    return Err(format!("longjump to {lab}: not a label"));
                }
                while let Some(&(p, depth)) = cx.frames.last() {
                    if p == lev {
                        cx.stack.truncate(depth);
                        cx.p = lev;
                        cx.pc = lab as usize;
                        return Ok(Flow::Go);
                    }
                    cx.frames.pop();
                }
                return Err(format!("longjump: no live frame at {lev}"));
            }
            Control::Stop => return Ok(Flow::Done(a(0))),
            Control::Abort => return Err(format!("abort({})", a(0))),
        }
        Ok(Flow::Go)
    }
}
