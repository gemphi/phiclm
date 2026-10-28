//! The algebra of relations, as *Principia Mathematica* has it (Russell
//! and Whitehead, ∗30–∗37, ∗90), over sets of pairs — every operation a
//! set operation on what was told, none of them an inference engine.
//!
//! | Principia | here |
//! |---|---|
//! | D‘R, Ɑ‘R, C‘R — domain, converse domain, field | [`domain`], [`range`], [`field`] |
//! | Cnv‘R — the converse | [`super::converse`] |
//! | R ∣ S — the relative product | [`super::compose`] |
//! | R ∪ S, R ∩ S, R − S — union, meet, difference | [`super::union`], [`super::meet`], [`difference`] |
//! | R ⊆ S — inclusion | [`included`] |
//! | R‘y, R→‘y — referents of `y`; R‘x — relata of `x` | [`referents`], [`relata`] |
//! | R“α — the R-image of a class | [`image`] |
//! | α ⥏ R, R ↾ β — domain- and range-restriction | [`restrict_domain`], [`restrict_range`] |
//! | α ↑ β — the Cartesian product | [`product`] |
//! | I ↾ α — identity on a class | [`identity`] |
//! | R², Rⁿ — powers | [`power`] |
//! | R∗ — the ancestral (reflexive-transitive closure, ∗90) | [`ancestral`] |
//! | R_po — the proper ancestral (transitive closure) | [`closure`] |

use super::{compose, union, Relation};
use std::collections::BTreeSet;

/// A class: a set of words.
pub type Class = BTreeSet<String>;

/// D‘R: everything that stands in R to something.
pub fn domain(r: &Relation) -> Class {
    r.iter().map(|(x, _)| x.clone()).collect()
}

/// Ɑ‘R: everything something stands in R to.
pub fn range(r: &Relation) -> Class {
    r.iter().map(|(_, y)| y.clone()).collect()
}

/// C‘R: the field, domain and range together.
pub fn field(r: &Relation) -> Class {
    domain(r).union(&range(r)).cloned().collect()
}

/// R − S.
pub fn difference(r: &Relation, s: &Relation) -> Relation {
    r.difference(s).cloned().collect()
}

/// R ⊆ S.
pub fn included(r: &Relation, s: &Relation) -> bool {
    r.is_subset(s)
}

/// R‘x read forwards: everything `x` stands in R to.
pub fn relata(r: &Relation, x: &str) -> Class {
    r.iter().filter(|(a, _)| a == x).map(|(_, b)| b.clone()).collect()
}

/// Everything that stands in R to `y`.
pub fn referents(r: &Relation, y: &str) -> Class {
    r.iter().filter(|(_, b)| b == y).map(|(a, _)| a.clone()).collect()
}

/// R“α: everything some member of `a` stands in R to.
pub fn image(r: &Relation, a: &Class) -> Class {
    r.iter().filter(|(x, _)| a.contains(x)).map(|(_, y)| y.clone()).collect()
}

/// α ⥏ R: R with its domain limited to `a`.
pub fn restrict_domain(r: &Relation, a: &Class) -> Relation {
    r.iter().filter(|(x, _)| a.contains(x)).cloned().collect()
}

/// R ↾ β: R with its range limited to `b`.
pub fn restrict_range(r: &Relation, b: &Class) -> Relation {
    r.iter().filter(|(_, y)| b.contains(y)).cloned().collect()
}

/// α ↑ β: every pair from `a` to `b`.
pub fn product(a: &Class, b: &Class) -> Relation {
    a.iter().flat_map(|x| b.iter().map(move |y| (x.clone(), y.clone()))).collect()
}

/// I ↾ α: each member of `a` to itself.
pub fn identity(a: &Class) -> Relation {
    a.iter().map(|x| (x.clone(), x.clone())).collect()
}

/// Rⁿ: R composed with itself `n` times; R⁰ is the identity on its field.
pub fn power(r: &Relation, n: usize) -> Relation {
    (0..n).fold(identity(&field(r)), |acc, _| compose(&acc, r))
}

/// R_po: the transitive closure — `x` to everything reachable from it in
/// one or more steps. Grows until nothing new is reached.
pub fn closure(r: &Relation) -> Relation {
    let mut out = r.clone();
    loop {
        let step = compose(&out, r);
        let grown = union(&out, &step);
        if grown.len() == out.len() {
            return out;
        }
        out = grown;
    }
}

/// R∗: the ancestral — the closure, with every member of the field to itself.
pub fn ancestral(r: &Relation) -> Relation {
    union(&closure(r), &identity(&field(r)))
}

/// Whether R is transitive, symmetric, reflexive on its field: the three
/// questions ∗34 asks of a relation before calling it an order.
pub fn is_transitive(r: &Relation) -> bool {
    included(&compose(r, r), r)
}
pub fn is_symmetric(r: &Relation) -> bool {
    included(&super::converse(r), r)
}
pub fn is_reflexive(r: &Relation) -> bool {
    included(&identity(&field(r)), r)
}
