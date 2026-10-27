//! Field selectors: `SLCT len:sh:offset` unpacked, `f OF p` read
//! (`bcplfe.b:3105`) and `f OF p := e` written into the old word (`:3498`).

use super::Trn;
use crate::ocode::Op;
use crate::tree::Expression;

impl Trn {
    /// The selector `f` unpacked: `(len, sh, offset)`.
    pub(super) fn selector(&self, f: &Expression) -> Result<(i64, i64, i64), String> {
        let s = self.fold(f).ok_or_else(|| format!("OF needs a constant selector, found {f:?}"))?;
        Ok((s >> 24, (s >> 16) & 255, s & 0xFFFF))
    }

    /// The word `p!offset`, loaded.
    pub(super) fn word_of(&mut self, p: &Expression, offset: i64) -> Result<(), String> {
        self.expression(p)?;
        if offset != 0 {
            self.emit(Op::Num(offset));
            self.emit(Op::Add);
        }
        self.emit(Op::Rv);
        Ok(())
    }

    /// `f OF p` (`bcplfe.b:3105`): the word, shifted down, masked to `len`
    /// bits (all of the rest when `len` is 0).
    pub(super) fn field(&mut self, f: &Expression, p: &Expression) -> Result<(), String> {
        let (len, sh, offset) = self.selector(f)?;
        self.word_of(p, offset)?;
        if sh != 0 {
            self.emit(Op::Num(sh));
            self.emit(Op::Rsh);
        }
        if len > 0 && len + sh < 64 {
            self.emit(Op::Num((1 << len) - 1));
            self.emit(Op::And);
        }
        Ok(())
    }

    /// `f OF p := e` (`bcplfe.b:3498`): the new bits laid into the old word,
    /// the rest of the word kept — `((e << sh) NEQV old) & mask NEQV old`.
    pub(super) fn store_field(&mut self, f: &Expression, p: &Expression, e: &Expression) -> Result<(), String> {
        let (len, sh, offset) = self.selector(f)?;
        let mask = match len > 0 {
            true => ((1i64 << len) - 1) << sh,
            false => -1i64 << sh,
        };
        self.expression(e)?;
        if sh != 0 {
            self.emit(Op::Num(sh));
            self.emit(Op::Lsh);
        }
        if mask != -1 {
            self.word_of(p, offset)?;
            self.emit(Op::Neqv);
            self.emit(Op::Num(mask));
            self.emit(Op::And);
            self.word_of(p, offset)?;
            self.emit(Op::Neqv);
        }
        self.expression(p)?;
        if offset != 0 {
            self.emit(Op::Num(offset));
            self.emit(Op::Add);
        }
        self.emit(Op::Stind);
        Ok(())
    }
}
