use super::set::*;
use super::{Host, Machine, Value};

fn pairs(v: &[(&str, &str)]) -> Relation {
    v.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
}

#[test]
fn reads_and_evaluates() {
    let mut m: Machine<()> = Machine::new();
    assert_eq!(m.run("(+ 1 2 (* 2 3))").unwrap(), Value::Num(9.0));
    assert_eq!(m.run("(car '(a b))").unwrap(), Value::Sym("a".into()));
    assert_eq!(m.run("(define sq (lambda (x) (* x x))) (sq 7)").unwrap(), Value::Num(49.0));
    assert_eq!(m.run("(let ((x 2)) (if (< x 3) \"small\" \"big\"))").unwrap(), Value::Str("small".into()));
    assert_eq!(m.run("(length (list 1 2 3)) ; a comment\n").unwrap(), Value::Num(3.0));
}

#[test]
fn principia_on_a_chain() {
    let r = pairs(&[("a", "b"), ("b", "c"), ("c", "d")]);
    assert_eq!(domain(&r).len(), 3);
    assert_eq!(range(&r).len(), 3);
    assert_eq!(field(&r).len(), 4);
    assert_eq!(relata(&r, "a"), ["b".to_string()].into_iter().collect());
    assert_eq!(referents(&r, "d"), ["c".to_string()].into_iter().collect());
    assert_eq!(image(&r, &["a".to_string(), "b".to_string()].into_iter().collect()).len(), 2);
    assert_eq!(power(&r, 2), pairs(&[("a", "c"), ("b", "d")]));
    assert_eq!(closure(&r).len(), 6, "a→b,c,d; b→c,d; c→d");
    assert_eq!(ancestral(&r).len(), 10, "plus the four identities");
    assert!(is_transitive(&closure(&r)) && !is_transitive(&r));
    assert!(is_reflexive(&ancestral(&r)) && !is_symmetric(&r));
    let only_a: Class = ["a".to_string()].into_iter().collect();
    assert_eq!(restrict_domain(&r, &only_a), pairs(&[("a", "b")]));
    assert_eq!(restrict_range(&r, &only_a), Relation::new());
    assert_eq!(product(&only_a, &only_a), identity(&only_a));
    assert_eq!(difference(&r, &pairs(&[("a", "b")])).len(), 2);
    assert!(included(&pairs(&[("a", "b")]), &r));
}

/// A host's primitives are bound after the core's and called by name.
struct Twice;

impl Host for Twice {
    fn names(&self) -> &'static [&'static str] {
        &["twice"]
    }
    fn call(&mut self, name: &'static str, args: Vec<Value>) -> Result<Value, String> {
        match name {
            "twice" => Ok(Value::Num(2.0 * super::as_num(super::arg(&args, 0, name)?)?)),
            other => Err(format!("unknown primitive {other}")),
        }
    }
}

#[test]
fn a_host_adds_its_primitives() {
    let mut m = Machine::with_host(Twice);
    assert_eq!(m.run("(twice (+ 1 2))").unwrap(), Value::Num(6.0));
    assert_eq!(m.run("(count (union (pairs (a b)) (pairs (b c))))").unwrap(), Value::Num(2.0), "the core is still there");
    let mut bare: Machine<()> = Machine::new();
    // Without the host, `twice` is unbound: a word, and a word is not a function.
    assert!(bare.run("(twice 3)").unwrap_err().contains("not a function"));
}

struct Shadow;

impl Host for Shadow {
    fn names(&self) -> &'static [&'static str] {
        &["union"]
    }
    fn call(&mut self, _: &'static str, _: Vec<Value>) -> Result<Value, String> {
        Ok(Value::Nil)
    }
}

#[test]
#[should_panic(expected = "repeats a core primitive")]
fn a_host_may_not_shadow_the_core() {
    let _ = Machine::with_host(Shadow);
}

#[test]
fn the_judge_runs_bcpl() {
    let mut m: Machine<()> = Machine::new();
    let rows = m.run("(judge \"LET start() = VALOF RESULTIS 6 * 7\")").unwrap();
    let Value::List(rows) = rows else { panic!("rows expected") };
    let row = |kind: &str| rows.iter().find_map(|r| match r {
        Value::List(v) if v[1] == Value::Str(kind.into()) => Some(v[2].clone()),
        _ => None,
    });
    assert_eq!(row("compiled"), Some(Value::Str("yes".into())));
    assert_eq!(row("returned"), Some(Value::Str("42".into())));
}
