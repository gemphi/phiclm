//! The machine: BCPL's model kept whole — ONE word type, a linear store,
//! frames IN the store at P (so `@` of a local is a real address and `!`
//! reaches anything), a global vector at the bottom, strings loaded after
//! it, and the stack region above. Expression evaluation uses a Rust-side
//! stack, as INTCODE's interpreter kept registers outside the store.

use crate::global::{native, NATIVE};
use crate::ocode::{Op, Program};

/// Words of store, and where its regions sit.
pub const MEMORY: usize = 1 << 16;
pub const GLOBAL: usize = 400;

/// A return address meaning "the machine was entered here".
const OUTSIDE: i64 = -9_000_000;

pub struct Machine {
    pub mem: Vec<i64>,
    pub out: String,
    pub steps: usize,
    /// Each string's address in the store.
    string: Vec<i64>,
}

impl Machine {
    /// Loads a program: entries into the global vector, natives as negative
    /// entries, strings as `[len, chars…]` after the globals.
    pub fn load(program: &Program) -> Self {
        let mut mem = vec![0i64; MEMORY];
        for (i, _) in NATIVE.iter().enumerate() {
            mem[i] = -1 - i as i64;
        }
        for (slot, entry) in &program.entry {
            mem[*slot] = *entry as i64;
        }
        let mut at = GLOBAL;
        let mut string = Vec::new();
        for s in &program.string {
            string.push(at as i64);
            mem[at] = s.chars().count() as i64;
            for (i, c) in s.chars().enumerate() {
                mem[at + 1 + i] = c as i64;
            }
            at += s.chars().count() + 1;
        }
        let mut m = Machine { mem, out: String::new(), steps: 0, string };
        m.mem[GLOBAL - 1] = (at + 8) as i64; // where the stack region starts
        m
    }

    fn word(&self, at: i64) -> Result<i64, String> {
        self.mem.get(at as usize).copied().ok_or_else(|| format!("read outside the store: {at}"))
    }

    fn set(&mut self, at: i64, v: i64) -> Result<(), String> {
        match self.mem.get_mut(at as usize) {
            Some(w) => {
                *w = v;
                Ok(())
            }
            None => Err(format!("write outside the store: {at}")),
        }
    }

    /// Runs `start()` (the cintsys convention) for at most `limit` steps.
    pub fn run(&mut self, program: &Program, limit: usize) -> Result<i64, String> {
        let slot = *program.global.get("start").ok_or("no start()")?;
        let entry = self.mem[slot];
        if entry <= 0 {
            return Err("start() has no body".into());
        }
        let mut p = self.mem[GLOBAL - 1];
        self.set(p, OUTSIDE)?;
        self.set(p + 1, OUTSIDE)?;
        let mut pc = entry as usize;
        let mut stack: Vec<i64> = Vec::new();
        let pop = |stack: &mut Vec<i64>| stack.pop().ok_or_else(|| "stack underflow".to_string());
        while self.steps < limit {
            self.steps += 1;
            let op = *program.op.get(pc).ok_or("ran off the code")?;
            pc += 1;
            match op {
                Op::Num(n) => stack.push(n),
                Op::Str(i) => stack.push(self.string[i]),
                Op::Lp(k) => stack.push(self.word(p + k as i64)?),
                Op::Sp(k) => {
                    let v = pop(&mut stack)?;
                    self.set(p + k as i64, v)?;
                }
                Op::Llp(k) => stack.push(p + k as i64),
                Op::Lg(k) => stack.push(self.mem[k]),
                Op::Sg(k) => {
                    let v = pop(&mut stack)?;
                    self.mem[k] = v;
                }
                Op::Rv => {
                    let a = pop(&mut stack)?;
                    stack.push(self.word(a)?);
                }
                Op::Stind => {
                    let a = pop(&mut stack)?;
                    let v = pop(&mut stack)?;
                    self.set(a, v)?;
                }
                Op::Pop => {
                    pop(&mut stack)?;
                }
                Op::Neg => {
                    let a = pop(&mut stack)?;
                    stack.push(-a);
                }
                Op::Not => {
                    let a = pop(&mut stack)?;
                    stack.push(!a);
                }
                Op::Abs => {
                    let a = pop(&mut stack)?;
                    stack.push(a.abs());
                }
                Op::Jump(t) => pc = t,
                Op::Jf(t) => {
                    if pop(&mut stack)? == 0 {
                        pc = t;
                    }
                }
                Op::Jt(t) => {
                    if pop(&mut stack)? != 0 {
                        pc = t;
                    }
                }
                Op::Call { args, frame } => {
                    let mut arg = vec![0i64; args];
                    for a in arg.iter_mut().rev() {
                        *a = pop(&mut stack)?;
                    }
                    let f = pop(&mut stack)?;
                    if f < 0 {
                        stack.push(native(self, f, &arg)?);
                    } else {
                        let np = p + frame as i64;
                        self.set(np, p)?;
                        self.set(np + 1, pc as i64)?;
                        for (i, a) in arg.iter().enumerate() {
                            self.set(np + 2 + i as i64, *a)?;
                        }
                        p = np;
                        pc = f as usize;
                    }
                }
                Op::Fnrn | Op::Rtrn => {
                    if matches!(op, Op::Rtrn) {
                        stack.push(0);
                    }
                    let back = self.word(p + 1)?;
                    if back == OUTSIDE {
                        return pop(&mut stack);
                    }
                    p = self.word(p)?;
                    pc = back as usize;
                }
                Op::Finish => return Ok(stack.pop().unwrap_or(0)),
                two @ (Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Rem | Op::And | Op::Or | Op::Lsh | Op::Rsh
                | Op::Eq | Op::Ne | Op::Ls | Op::Gr | Op::Le | Op::Ge) => {
                    let b = pop(&mut stack)?;
                    let a = pop(&mut stack)?;
                    let truth = |t: bool| if t { -1 } else { 0 };
                    stack.push(match two {
                        Op::Add => a.wrapping_add(b),
                        Op::Sub => a.wrapping_sub(b),
                        Op::Mul => a.wrapping_mul(b),
                        Op::Div => a.checked_div(b).ok_or("division by zero")?,
                        Op::Rem => a.checked_rem(b).ok_or("REM by zero")?,
                        Op::And => a & b,
                        Op::Or => a | b,
                        Op::Lsh => a.wrapping_shl(b as u32),
                        Op::Rsh => ((a as u64).wrapping_shr(b as u32)) as i64,
                        Op::Eq => truth(a == b),
                        Op::Ne => truth(a != b),
                        Op::Ls => truth(a < b),
                        Op::Gr => truth(a > b),
                        Op::Le => truth(a <= b),
                        Op::Ge => truth(a >= b),
                        _ => unreachable!(),
                    });
                }
            }
        }
        Err(format!("out of steps after {limit}"))
    }
}
