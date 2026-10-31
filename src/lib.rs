//! # phiclm — Phi Compiler Language Model: BCPL's architecture as one of ours
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and
//! composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon.
//!
//! CPL is the Combined Programming Language (Cambridge + London, 1963,
//! Strachey); Richards cut it to the bootstrappable core in 1967 — Basic
//! CPL, BCPL — which begat B, which begat C. phiclm continues the line:
//! the same architecture, a crate of this workspace, so the power is
//! extracted and callable from every other crate (docs/178).
//!
//! | BCPL piece | here | what |
//! |---|---|---|
//! | `g/bcplfecg.h` | [`manifest`] | the one numbering; every token, tree node and op on it by `kind()` |
//! | `declsyswords` + lexer | [`token`] | the closed vocabulary of structure; the ONE copy of `KEYWORDS` |
//! | syntax analyser | [`parse`] | tokens → [`tree`] by BCPL's own `rexp(n)` priorities |
//! | `trn` | [`trn`] | the tree → [`ocode`], BCPL's tiny stack intermediate (55 ops) |
//! | `cinterp`/`cintsys` | [`machine`] | one word type, a linear store, P-frames, `!` `%` and `@`, floats in words, a heap |
//! | BLIB + the global vector | [`global`] | `writef` with BLIB's widths, the `write*` family, characters, strings, `getvec`, `rdch`/`readn`, `muldiv`, as negative entries |
//! | — (docs/169 §4) | [`judge`] | execution as the judge: compile → run under [`judge::Caps`] → [`judge::Verdict`] → rows |
//! | — (docs/169 §3) | [`tell`] | the compiled O-code told as rows the student reads |
//!
//! Not ported (the README says why): the CG stage to Cintcode, `sys` and
//! real files.
//!
//! ```
//! let v = phiclm::judge::judge(
//!     "LET fact(n) = n = 0 -> 1, n * fact(n - 1)
//!      LET start() = VALOF
//!      $( writef(\"fact(%n) = %n*n\", 5, fact(5))
//!         RESULTIS fact(6)
//!      $)",
//!     100_000,
//! );
//! assert_eq!(v.result, 720);
//! assert_eq!(v.out, "fact(5) = 120\n");
//! ```
//!
//! Generated source — which may not stop, and may not be a program at all —
//! goes to [`judge::judge_capped`], which holds a run to the declared
//! [`judge::STEP_CAP`] and [`judge::SECONDS_CAP`] and names the cap it
//! reached:
//!
//! ```
//! let v = phiclm::judge_capped("LET start() = VALOF $( LET n = 0\n WHILE TRUE DO n := n + 1\n RESULTIS n $)");
//! assert_eq!(v.capped, Some(phiclm::Cap::Steps));
//! assert_eq!(v.steps, phiclm::STEP_CAP);
//! assert!(!v.supported());
//! ```

pub mod global;
pub mod judge;
pub mod lisp;
pub mod machine;
pub mod ocode;
pub mod manifest;
pub mod parse;
pub mod tell;
pub mod token;
pub mod tree;
pub mod trn;

pub use judge::{compile, judge, judge_files, judge_with, Verdict};
pub use judge::{judge_capped, judge_under, Cap, Caps, SECONDS_CAP, STEP_CAP};

#[cfg(test)]
mod tests;
