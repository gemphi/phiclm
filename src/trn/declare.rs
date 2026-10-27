//! Declarations in a block: a `LET … AND …` run, declared together.

use super::{Binding, Trn};
use crate::ocode::Op;
use crate::tree::Declaration;

impl Trn {
    /// One `LET … AND …` run, declared together (`bcplfe.b:2480`): every
    /// value is computed before any of the new names is in scope, so
    /// `LET a, b = b, a` reads the outer `a` and `b`.
    pub(super) fn declare(&mut self, run: &[Declaration]) -> Result<(), String> {
        // A function's name is in scope at once, for every body in the run:
        // mutual recursion. Its body sees the run's static names, never a
        // frame's (BCPL has no dynamic free variables).
        let mut functions = Vec::new();
        for d in run {
            if let Declaration::Function(name, ..) | Declaration::Routine(name, ..) = d {
                let slot = self.slot()?;
                self.scope.push((name.clone(), Binding::Cell(slot)));
                functions.push((slot, d.clone()));
            }
        }
        let base: Vec<(String, Binding)> = self.scope.iter().filter(|(_, b)| !matches!(b, Binding::Local(_))).cloned().collect();
        for (slot, d) in functions {
            self.pending.push((slot, base.clone(), d));
        }
        let mut bound = Vec::new();
        for d in run {
            match d {
                Declaration::Function(..) | Declaration::Routine(..) => {}
                Declaration::Variable(name, e) => {
                    self.expression(e)?;
                    let at = self.free;
                    self.free += 1;
                    self.emit(Op::Sp(at));
                    bound.push((name.clone(), Binding::Local(at)));
                }
                Declaration::Manifest(rows) => self.rows(rows, "MANIFEST", |t, name, k| {
                    t.scope.push((name.into(), Binding::Const(k)));
                    Ok(())
                })?,
                Declaration::Static(rows) => self.rows(rows, "STATIC", |t, name, k| {
                    let slot = t.cell(k)?;
                    t.scope.push((name.into(), Binding::Cell(slot)));
                    Ok(())
                })?,
                Declaration::Global(rows) => self.rows(rows, "GLOBAL", |t, name, k| {
                    let slot = t.global(name, k)?;
                    t.scope.push((name.into(), Binding::Cell(slot)));
                    Ok(())
                })?,
            }
        }
        self.scope.extend(bound);
        Ok(())
    }
}
