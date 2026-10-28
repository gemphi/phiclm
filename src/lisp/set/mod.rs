//! `lisp::set` — the algebra the Lisp computes over (docs/162): a relation
//! is a set of `(about, other)` pairs in Russell and Whitehead's sense, a
//! class a set of words, and every operation is a set operation on what was
//! told — none of them an inference. It was phiano's `sys::set`; phiano
//! keeps the parts that read a persona (`held`, `think`, `render`).

use std::collections::BTreeSet;

mod algebra;
pub use algebra::*;

/// A relation: the set of `(about, other)` pairs.
pub type Relation = BTreeSet<(String, String)>;

/// What either holds.
pub fn union(a: &Relation, b: &Relation) -> Relation {
    a.union(b).cloned().collect()
}

/// What both hold — the agreement.
pub fn meet(a: &Relation, b: &Relation) -> Relation {
    a.intersection(b).cloned().collect()
}

/// `R ∘ S`: `(x, z)` for every `(x, y)` in `R` and `(y, z)` in `S`.
pub fn compose(r: &Relation, s: &Relation) -> Relation {
    r.iter().flat_map(|(x, y)| s.iter().filter(move |(y2, _)| y2 == y).map(move |(_, z)| (x.clone(), z.clone()))).collect()
}

/// `R˘`: every pair turned round.
pub fn converse(r: &Relation) -> Relation {
    r.iter().map(|(x, y)| (y.clone(), x.clone())).collect()
}
