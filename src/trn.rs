//! `trn` — the tree translated to O-code, keeping BCPL's own name for the
//! pass. One walk, labels fixed up in place, the frame laid out as the
//! machine expects: slot 0 the saved P, slot 1 the saved PC, parameters
//! from slot 2, locals and vectors after.

use crate::ocode::{Op, Program};
use crate::tree::{Declaration, Expression};
use std::collections::HashMap;

struct Trn {
    program: Program,
    constant: HashMap<String, i64>,
    scope: Vec<(String, usize)>,
    /// Next free frame slot — also the distance a new frame sits up.
    free: usize,
    /// Enclosing loops: the BREAK jumps and the LOOP jumps, each patched
    /// when the construct closes (BREAK to after it, LOOP to its test or
    /// its increment — the construct knows which).
    looping: Vec<(Vec<usize>, Vec<usize>)>,
    /// Enclosing VALOFs: the jumps to each one's end, patched when it closes.
    valof: Vec<Vec<usize>>,
}

/// Translates a parsed program. Natives are declared first, so `writef`
/// and its kin are global names like any other (the global-vector trick).
pub fn translate(tree: &[Declaration]) -> Result<Program, String> {
    let mut t = Trn { program: Program::default(), constant: HashMap::new(), scope: Vec::new(), free: 2, looping: Vec::new(), valof: Vec::new() };
    // A guard at pc 0, so no function's entry is 0 and an unset global
    // vector slot (0) can never be entered.
    t.program.op.push(Op::Finish);
    for (i, name) in crate::global::NATIVE.iter().enumerate() {
        t.program.global.insert(name.to_string(), i);
    }
    let mut next = crate::global::NATIVE.len();
    // Pass 1: every top-level name gets its slot, so forward calls resolve.
    for d in tree {
        match d {
            Declaration::Function(name, ..) | Declaration::Routine(name, ..) | Declaration::Variable(name, _) => {
                t.program.global.entry(name.clone()).or_insert_with(|| {
                    next += 1;
                    next - 1
                });
            }
            Declaration::Manifest(rows) => t.constant.extend(rows.iter().cloned()),
            Declaration::Global(rows) => {
                for (name, slot) in rows {
                    t.program.global.insert(name.clone(), crate::global::NATIVE.len() + *slot);
                    next = next.max(crate::global::NATIVE.len() + slot + 1);
                }
            }
        }
    }
    for d in tree {
        t.top(d)?;
    }
    Ok(t.program)
}

impl Trn {
    fn emit(&mut self, op: Op) -> usize {
        self.program.op.push(op);
        self.program.op.len() - 1
    }

    fn here(&self) -> usize {
        self.program.op.len()
    }

    fn patch(&mut self, at: usize) {
        let to = self.here();
        self.program.op[at] = match self.program.op[at] {
            Op::Jump(_) => Op::Jump(to),
            Op::Jf(_) => Op::Jf(to),
            Op::Jt(_) => Op::Jt(to),
            other => other,
        };
    }

    fn top(&mut self, d: &Declaration) -> Result<(), String> {
        match d {
            Declaration::Function(name, params, body) => self.enter(name, params, |t| {
                t.expression(body)?;
                t.emit(Op::Fnrn);
                Ok(())
            }),
            Declaration::Routine(name, params, body) => self.enter(name, params, |t| {
                t.command(body)?;
                t.emit(Op::Rtrn);
                Ok(())
            }),
            Declaration::Variable(name, e) => {
                // A top-level value must be constant in v1.
                match self.fold(e) {
                    Some(k) => {
                        self.constant.insert(name.clone(), k);
                        Ok(())
                    }
                    None => Err(format!("top-level LET {name} must be constant in v1")),
                }
            }
            Declaration::Manifest(_) | Declaration::Global(_) => Ok(()),
        }
    }

    fn enter(&mut self, name: &str, params: &[String], body: impl FnOnce(&mut Self) -> Result<(), String>) -> Result<(), String> {
        let slot = self.program.global[name];
        let entry = self.here();
        self.program.entry.push((slot, entry));
        self.scope.clear();
        self.free = 2;
        for p in params {
            let at = self.free;
            self.scope.push((p.clone(), at));
            self.free += 1;
        }
        body(self)
    }

    fn fold(&self, e: &Expression) -> Option<i64> {
        match e {
            Expression::Num(n) => Some(*n),
            Expression::True => Some(-1),
            Expression::False => Some(0),
            Expression::Name(n) => self.constant.get(n).copied(),
            _ => None,
        }
    }

    fn local(&self, name: &str) -> Option<usize> {
        self.scope.iter().rev().find(|(n, _)| n == name).map(|(_, at)| *at)
    }
}

mod express;
mod order;
