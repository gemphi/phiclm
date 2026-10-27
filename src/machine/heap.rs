//! `getvec`'s heap: words taken from the top of the store down, given back
//! as a stack gives them back.

use super::{Machine, FLOOR, MEMORY};

impl Machine {
    /// `getvec(upb)`: words `0..=upb` from the top of the store, or 0 when
    /// the heap would meet the stack's half.
    pub(crate) fn getvec(&mut self, upb: i64) -> i64 {
        match upb >= 0 && self.heap - upb - 1 >= FLOOR {
            true => {
                self.heap -= upb + 1;
                self.block.push((self.heap, true));
                self.heap
            }
            false => 0,
        }
    }

    /// `freevec(v)`: the block is given back; the heap rises past every
    /// freed block at its bottom.
    pub(crate) fn freevec(&mut self, v: i64) {
        if let Some(b) = self.block.iter_mut().find(|(at, live)| *at == v && *live) {
            b.1 = false;
        }
        while let Some((_, false)) = self.block.last() {
            self.block.pop();
        }
        self.heap = self.block.last().map_or(MEMORY as i64, |(at, _)| *at);
    }
}
