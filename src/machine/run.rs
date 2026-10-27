//! The run: fetch an op, do it, until `start()` returns or the steps run
//! out. The running thread's state is one [`Context`]; calls and returns
//! are in `call.rs`, the transfers between threads in `co.rs`.

use super::arith::{binary, unary};
use super::context::{Co, Context, Flow};
use super::{Machine, GLOBAL, OUTSIDE};
use crate::ocode::{Op, Program};

impl Machine {
    /// Runs `start()` (the cintsys convention) for at most `limit` steps.
    pub fn run(&mut self, program: &Program, limit: usize) -> Result<i64, String> {
        let slot = *program.global.get("start").ok_or("no start()")?;
        let entry = self.mem[slot];
        if entry <= 0 {
            return Err("start() has no body".into());
        }
        let p = self.mem[GLOBAL - 1];
        self.set(p, OUTSIDE)?;
        self.set(p + 1, OUTSIDE)?;
        // The main program is coroutine 0; its pointer is its first frame.
        self.co = vec![Co { cptr: p, parent: 0, body: entry, size: 0, saved: None }];
        self.current = p;
        let mut cx = Context { p, pc: entry as usize, frames: vec![(p, 0)], ..Context::default() };
        while self.steps < limit {
            self.steps += 1;
            let op = *program.op.get(cx.pc).ok_or("ran off the code")?;
            cx.pc += 1;
            match self.step(op, &mut cx, program)? {
                Flow::Go => {}
                Flow::Done(v) => return Ok(v),
            }
        }
        Err(format!("out of steps after {limit}"))
    }

    fn step(&mut self, op: Op, cx: &mut Context, program: &Program) -> Result<Flow, String> {
        match op {
            Op::Num(n) => cx.stack.push(n),
            Op::Str(i) => cx.stack.push(self.string[i]),
            Op::Table(i) => cx.stack.push(self.table[i]),
            Op::Lp(k) => {
                let v = self.word(cx.p + k as i64)?;
                cx.stack.push(v);
            }
            Op::Sp(k) => {
                let v = cx.pop()?;
                self.set(cx.p + k as i64, v)?;
            }
            Op::Llp(k) => cx.stack.push(cx.p + k as i64),
            Op::Lg(k) => cx.stack.push(self.mem[k]),
            Op::Sg(k) => {
                let v = cx.pop()?;
                self.mem[k] = v;
            }
            Op::Rv => {
                let a = cx.pop()?;
                let v = self.word(a)?;
                cx.stack.push(v);
            }
            Op::Stind => {
                let a = cx.pop()?;
                let v = cx.pop()?;
                self.set(a, v)?;
            }
            Op::Getbyte => {
                let i = cx.pop()?;
                let s = cx.pop()?;
                let v = self.word(s + i)?;
                cx.stack.push(v);
            }
            Op::Putbyte => {
                let i = cx.pop()?;
                let s = cx.pop()?;
                let v = cx.pop()?;
                self.set(s + i, v)?;
            }
            Op::Pop => {
                cx.pop()?;
            }
            one @ (Op::Neg | Op::Not | Op::Abs | Op::Float | Op::Fix | Op::Fneg | Op::Fabs) => {
                let a = cx.pop()?;
                cx.stack.push(unary(one, a));
            }
            Op::Jump(t) => cx.pc = t,
            Op::Goto => {
                let t = cx.pop()?;
                match t > 0 && (t as usize) < program.op.len() {
                    true => cx.pc = t as usize,
                    false => return Err(format!("GOTO {t}: not a label")),
                }
            }
            Op::Jf(t) => {
                if cx.pop()? == 0 {
                    cx.pc = t;
                }
            }
            Op::Jt(t) => {
                if cx.pop()? != 0 {
                    cx.pc = t;
                }
            }
            Op::Call { args, frame } => return self.call(cx, args, frame, program),
            Op::Fnrn => return self.back(cx, false, program),
            Op::Rtrn => return self.back(cx, true, program),
            Op::Finish => return Ok(Flow::Done(cx.stack.pop().unwrap_or(0))),
            two @ (Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Rem | Op::And | Op::Or | Op::Lsh | Op::Rsh
            | Op::Eqv | Op::Neqv | Op::Eq | Op::Ne | Op::Ls | Op::Gr | Op::Le | Op::Ge | Op::Fadd | Op::Fsub
            | Op::Fmul | Op::Fdiv | Op::Feq | Op::Fne | Op::Fls | Op::Fgr | Op::Fle | Op::Fge) => {
                let b = cx.pop()?;
                let a = cx.pop()?;
                cx.stack.push(binary(two, a, b)?);
            }
        }
        Ok(Flow::Go)
    }
}
