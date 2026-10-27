//! `trn` — the tree translated to O-code, keeping BCPL's own name for the
//! pass. One walk, labels fixed up in place, the frame laid out as the
//! machine expects: slot 0 the saved P, slot 1 the saved PC, parameters
//! from slot 2, locals and vectors after. `STATIC` cells take global-vector
//! slots no name owns, so they live as long as the program. Every name is
//! looked up innermost first, whatever it names — a local shadows a
//! `libhdr` manifest as BCPL's scopes do.

use crate::ocode::{Op, Program};
use crate::tree::Declaration;
use std::collections::{HashMap, HashSet};

mod declare;
mod express;
mod field;
mod fold;
mod labels;
mod loops;
mod order;
mod routine;
mod switch;

/// What a name means where it is used.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Binding {
    /// A frame slot.
    Local(usize),
    /// A manifest constant.
    Const(i64),
    /// A global-vector cell: a global, a function's entry, a STATIC.
    Cell(usize),
    /// A label of the current routine: its value is its code address.
    Label,
}

struct Trn {
    program: Program,
    /// The top level's constants: MANIFESTs and constant LETs.
    constant: HashMap<String, i64>,
    /// The names declared in the routine being translated, innermost last.
    scope: Vec<(String, Binding)>,
    /// Next free frame slot — also the distance a new frame sits up.
    free: usize,
    /// Enclosing loops: the BREAK jumps and the LOOP jumps, each patched
    /// when the construct closes (BREAK to after it, LOOP to its test or
    /// its increment — the construct knows which).
    looping: Vec<(Vec<usize>, Vec<usize>)>,
    /// Enclosing VALOFs: the jumps to each one's end, patched when it closes.
    valof: Vec<Vec<usize>>,
    /// The next global-vector slot no name owns yet.
    next: usize,
    /// Enclosing SWITCHONs, innermost last.
    switching: Vec<Switch>,
    /// The current routine's label names, where each is set, the GOTOs
    /// waiting for them, and the places their values are loaded.
    labels: HashSet<String>,
    label: HashMap<String, usize>,
    goto: Vec<(usize, String)>,
    label_value: Vec<(usize, String)>,
    /// Functions declared in blocks, translated once their routine ends:
    /// each one's slot, the static names in scope where it was declared,
    /// and itself.
    pending: Vec<(usize, Vec<(String, Binding)>, Declaration)>,
}

/// One SWITCHON being translated: its CASEs `(k, pc)`, its DEFAULT, and the
/// ENDCASE jumps to patch to its end (`bcplfe.b:2706-2740`).
#[derive(Default)]
struct Switch {
    case: Vec<(i64, usize)>,
    default: Option<usize>,
    out: Vec<usize>,
}

/// Translates a parsed program. Natives are declared first, so `writef`
/// and its kin are global names like any other (the global-vector trick),
/// then `result2`, then the program's own.
pub fn translate(tree: &[Declaration]) -> Result<Program, String> {
    use crate::global::{NATIVE, RESULT2, USER};
    let mut t = Trn {
        program: Program::default(),
        constant: HashMap::new(),
        scope: Vec::new(),
        free: 2,
        looping: Vec::new(),
        valof: Vec::new(),
        next: USER,
        switching: Vec::new(),
        labels: HashSet::new(),
        label: HashMap::new(),
        goto: Vec::new(),
        label_value: Vec::new(),
        pending: Vec::new(),
    };
    // A guard at pc 0, so no function's entry is 0 and an unset global
    // vector slot (0) can never be entered.
    t.program.op.push(Op::Finish);
    for (i, name) in NATIVE.iter().enumerate() {
        t.program.global.insert(name.to_string(), i);
    }
    t.program.global.insert("result2".into(), RESULT2);
    // Pass 1, in written order: every function gets its slot, so forward
    // calls resolve; every constant is folded before any body reads it.
    for d in tree {
        match d {
            Declaration::Function(name, ..) | Declaration::Routine(name, ..) => {
                if !t.program.global.contains_key(name) {
                    let slot = t.slot()?;
                    t.program.global.insert(name.clone(), slot);
                }
            }
            Declaration::Variable(name, e) => {
                // The outer level has no frame: its values are constants.
                let k = t.fold(e).ok_or_else(|| format!("top-level LET {name} must be constant: the outer level has no frame"))?;
                t.constant.insert(name.clone(), k);
            }
            Declaration::Manifest(rows) => t.rows(rows, "MANIFEST", |t, name, k| {
                t.constant.insert(name.into(), k);
                Ok(())
            })?,
            Declaration::Global(rows) => t.rows(rows, "GLOBAL", |t, name, k| {
                let slot = t.global(name, k)?;
                t.program.global.insert(name.into(), slot);
                Ok(())
            })?,
            Declaration::Static(rows) => t.rows(rows, "STATIC", |t, name, k| {
                let slot = t.cell(k)?;
                t.program.global.insert(name.into(), slot);
                Ok(())
            })?,
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

    /// What `name` means here: the innermost declaration, then the top
    /// level's constants, then the global vector, then the routine's labels.
    fn lookup(&self, name: &str) -> Option<Binding> {
        match self.scope.iter().rev().find(|(n, _)| n == name) {
            Some((_, b)) => Some(*b),
            None => match (self.constant.get(name), self.program.global.get(name)) {
                (Some(k), _) => Some(Binding::Const(*k)),
                (None, Some(slot)) => Some(Binding::Cell(*slot)),
                (None, None) => self.labels.contains(name).then_some(Binding::Label),
            },
        }
    }
}
