//! Constant folding: `evalconst`, by the machine's own arithmetic.

use super::{express, Binding, Trn};
use crate::tree::Expression;

impl Trn {
    /// A constant expression's value (`evalconst`, `bcplfe.b:3355–3450`):
    /// numbers, manifest names, selectors, and every operator over them,
    /// computed by the machine's own arithmetic.
    pub(super) fn fold(&self, e: &Expression) -> Option<i64> {
        use crate::machine::{binary, unary};
        match e {
            Expression::Num(n) => Some(*n),
            Expression::Fnum(f) => Some(f.to_bits() as i64),
            Expression::True => Some(-1),
            Expression::False => Some(0),
            Expression::Name(n) => match self.lookup(n) {
                Some(Binding::Const(k)) => Some(k),
                _ => None,
            },
            Expression::Monad(op, a) => Some(unary(express::monad(op).ok()?, self.fold(a)?)),
            Expression::Dyad("!" | "%" | "of", ..) => None,
            Expression::Dyad(op, a, b) => binary(express::dyad(op).ok()?, self.fold(a)?, self.fold(b)?).ok(),
            Expression::Cond(q, yes, no) => match self.fold(q)? {
                0 => self.fold(no),
                _ => self.fold(yes),
            },
            Expression::Slct(len, sh, offset) => {
                let (len, sh, offset) = (self.fold(len)?, self.fold(sh)?, self.fold(offset)?);
                match (0..=255).contains(&len) && (0..=255).contains(&sh) && (0..=0xFFFF).contains(&offset) {
                    true => Some(len << 24 | sh << 16 | offset),
                    false => None,
                }
            }
            _ => None,
        }
    }
}
