//! Calls and returns: a frame built in the store at `P + frame`, BLIB
//! dispatched by its negative entry, and a return that may end the
//! program or a coroutine's body.

use super::context::{Context, Flow};
use super::{Machine, OUTSIDE, OUTSIDE_CO, ROOM};
use crate::global::{control, native};
use crate::ocode::Program;

impl Machine {
    /// `Call { args, frame }`: the arguments, then the entry, off the stack.
    pub(super) fn call(&mut self, cx: &mut Context, args: usize, frame: usize, program: &Program) -> Result<Flow, String> {
        let mut arg = vec![0i64; args];
        for a in arg.iter_mut().rev() {
            *a = cx.pop()?;
        }
        let f = cx.pop()?;
        if f < 0 {
            return match control(f) {
                Some(c) => self.control(c, &arg, cx, program),
                None => {
                    let v = native(self, f, &arg)?;
                    cx.stack.push(v);
                    Ok(Flow::Go)
                }
            };
        }
        let np = cx.p + frame as i64;
        match cx.end {
            None if np + ROOM >= self.heap => return Err("the stack ran into the heap".into()),
            Some(end) if np + 2 + args as i64 + 8 >= end => return Err("a coroutine's stack overflowed".into()),
            _ => {}
        }
        self.set(np, cx.p)?;
        self.set(np + 1, cx.pc as i64)?;
        for (i, a) in arg.iter().enumerate() {
            self.set(np + 2 + i as i64, *a)?;
        }
        cx.frames.push((np, cx.stack.len()));
        cx.p = np;
        cx.pc = f as usize;
        Ok(Flow::Go)
    }

    /// `Fnrn` (the value is on the stack) or `Rtrn` (a 0 is pushed).
    pub(super) fn back(&mut self, cx: &mut Context, routine: bool, program: &Program) -> Result<Flow, String> {
        if routine {
            cx.stack.push(0);
        }
        let back = self.word(cx.p + 1)?;
        match back {
            OUTSIDE => Ok(Flow::Done(cx.pop()?)),
            OUTSIDE_CO => {
                let v = cx.pop()?;
                self.finished(v, cx, program)
            }
            _ => {
                cx.frames.pop();
                cx.p = self.word(cx.p)?;
                cx.pc = back as usize;
                Ok(Flow::Go)
            }
        }
    }
}
