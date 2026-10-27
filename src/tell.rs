//! The compiler readable by its student (docs/169 §3): the compiled O-code
//! told as rows, so what the compiler DID — not just what the source said —
//! goes through the same `Row` every other knowing goes through.

use crate::ocode::{Op, Program};
use phiecs::Row;

impl Program {
    /// One row per operation, owned by the function it belongs to:
    /// `(fn, mnemonic, operand)`.
    pub fn rows(&self) -> Vec<Row> {
        let name: std::collections::HashMap<usize, &str> =
            self.global.iter().map(|(n, s)| (*s, n.as_str())).collect();
        let mut sorted: Vec<(usize, usize)> = self.entry.clone();
        sorted.sort_by_key(|(_, pc)| *pc);
        let owner = |pc: usize| {
            sorted
                .iter()
                .rev()
                .find(|(_, entry)| *entry <= pc)
                .and_then(|(slot, _)| name.get(slot).copied())
                .unwrap_or("?")
        };
        self.op
            .iter()
            .enumerate()
            .map(|(pc, op)| {
                let operand = match op {
                    Op::Num(n) => n.to_string(),
                    Op::Str(i) => self.string.get(*i).cloned().unwrap_or_default(),
                    Op::Table(i) => self.table.get(*i).map(|t| t.len().to_string()).unwrap_or_default(),
                    Op::Lp(k) | Op::Sp(k) | Op::Llp(k) | Op::Lg(k) | Op::Sg(k) => k.to_string(),
                    Op::Jump(t) | Op::Jf(t) | Op::Jt(t) => t.to_string(),
                    Op::Call { args, .. } => args.to_string(),
                    _ => String::new(),
                };

               // this is so cool
                Row::new(owner(pc), op.name(), &operand)
            })
            .collect()
    }
}
