//! A routine's translation: its slot, its frame, its labels resolved at its
//! end; and the declaration blocks' rows folded into names.

use super::labels::{command_labels, expression_labels};
use super::{Binding, Trn};
use crate::ocode::Op;
use crate::tree::{Declaration, Rows};
use std::collections::HashSet;

impl Trn {
    pub(super) fn top(&mut self, d: &Declaration) -> Result<(), String> {
        match d {
            Declaration::Function(name, ..) | Declaration::Routine(name, ..) => {
                let slot = self.program.global[name];
                self.routine(slot, Vec::new(), d)?;
                // Then the functions its blocks declared, and theirs.
                while let Some((slot, base, d)) = self.pending.pop() {
                    self.routine(slot, base, &d)?;
                }
                Ok(())
            }
            Declaration::Variable(..) | Declaration::Manifest(_) | Declaration::Global(_) | Declaration::Static(_) => Ok(()),
        }
    }

    /// One function or routine, its code at `slot`, `base` the names it
    /// sees besides its own.
    pub(super) fn routine(&mut self, slot: usize, base: Vec<(String, Binding)>, d: &Declaration) -> Result<(), String> {
        let mut set = HashSet::new();
        match d {
            Declaration::Function(name, params, body) => {
                expression_labels(body, &mut set);
                self.enter(name, slot, base, params, set, |t| {
                    t.expression(body)?;
                    t.emit(Op::Fnrn);
                    Ok(())
                })
            }
            Declaration::Routine(name, params, body) => {
                command_labels(body, &mut set);
                self.enter(name, slot, base, params, set, |t| {
                    t.command(body)?;
                    t.emit(Op::Rtrn);
                    Ok(())
                })
            }
            other => Err(format!("not a routine: {other:?}")),
        }
    }

    /// A block's rows folded in order, a bare name one past the last; `then`
    /// takes each before the next is folded, so later rows may name it.
    pub(super) fn rows(&mut self, rows: &Rows, what: &str, mut then: impl FnMut(&mut Self, &str, i64) -> Result<(), String>) -> Result<(), String> {
        let mut prev = -1i64;
        for (name, e) in rows {
            let k = match e {
                Some(e) => self.fold(e).ok_or_else(|| format!("{what} {name}: not a constant"))?,
                None => prev.wrapping_add(1),
            };
            prev = k;
            then(self, name, k)?;
        }
        Ok(())
    }

    /// A fresh global-vector slot, below the word that holds the stack's start.
    pub(super) fn slot(&mut self) -> Result<usize, String> {
        match self.next < crate::machine::GLOBAL - 1 {
            true => {
                self.next += 1;
                Ok(self.next - 1)
            }
            false => Err("the global vector is full".into()),
        }
    }

    /// A STATIC cell: a fresh slot holding `k` when the program loads.
    pub(super) fn cell(&mut self, k: i64) -> Result<usize, String> {
        let slot = self.slot()?;
        self.program.init.push((slot, k));
        Ok(slot)
    }

    /// `GLOBAL $( name: k $)` — user slot `k`, counted past the natives and
    /// `result2` (so `libhdr`'s `ug` is a user slot too).
    pub(super) fn global(&mut self, name: &str, k: i64) -> Result<usize, String> {
        let slot = crate::global::USER as i64 + k;
        match k >= 0 && slot < crate::machine::GLOBAL as i64 - 1 {
            true => {
                self.next = self.next.max(slot as usize + 1);
                Ok(slot as usize)
            }
            false => Err(format!("GLOBAL {name}: {k} is outside the global vector")),
        }
    }

    pub(super) fn enter(
        &mut self,
        name: &str,
        slot: usize,
        base: Vec<(String, Binding)>,
        params: &[String],
        labels: HashSet<String>,
        body: impl FnOnce(&mut Self) -> Result<(), String>,
    ) -> Result<(), String> {
        let entry = self.here();
        self.program.entry.push((slot, entry));
        self.scope = base;
        self.free = 2;
        self.labels = labels;
        self.label.clear();
        for p in params {
            let at = self.free;
            self.scope.push((p.clone(), Binding::Local(at)));
            self.free += 1;
        }
        body(self)?;
        // A label is reached, and read, before or after it is set.
        for (at, label) in std::mem::take(&mut self.goto) {
            let to = *self.label.get(&label).ok_or_else(|| format!("GOTO {label}: no such label in {name}"))?;
            self.program.op[at] = Op::Jump(to);
        }
        for (at, label) in std::mem::take(&mut self.label_value) {
            let to = *self.label.get(&label).ok_or_else(|| format!("label {label} is never set in {name}"))?;
            self.program.op[at] = Op::Num(to as i64);
        }
        Ok(())
    }
}
