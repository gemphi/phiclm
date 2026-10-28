//! A small Lisp over the sets (docs/165): McCarthy's machine on Russell's
//! algebra on Wittgenstein's counts. An S-expression is a list; a row is
//! `(kind about other)`; a relation is a set of pairs; the core primitives
//! are the whole of [`set`], lists, numbers, text and the [`judge`](crate::judge),
//! and a [`Host`] adds a world's own — phiano's adds `held`, `ask` and
//! `tell` on its personas and `next` and `says` on a story's scopes.
//! Nothing here infers: the machine computes over what was told, and only a
//! host's `tell` changes a world. It was phiano's `sys::lisp`
//! (docs/phiclm/lisp).
//!
//! ```text
//! (meet (held bertrand-russell is-a) (held whitehead is-a))
//! (count (closure (next "scope:text/emma/ch01")))
//! (define mortal (compose (held russell is-a) (held whitehead is-a)))
//! (tell claude mortal is-a)
//! ```

mod eval;
mod host;
mod prims;
pub mod set;
#[cfg(test)]
mod tests;

pub use eval::{Env, Machine};
pub use host::Host;
pub use prims::{arg, as_class, as_num, as_relation, as_text, NAMES};

use set::{Class, Relation};
use std::fmt;

/// What the machine passes around.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Num(f64),
    Str(String),
    Sym(String),
    List(Vec<Value>),
    Relation(Relation),
    Class(Class),
    /// `(lambda (params) body)` with the frames it closed over.
    Lambda(Vec<String>, Box<Value>, Env),
    Prim(&'static str),
}

impl Value {
    pub fn truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }
    /// A symbol or string as text; a number as its digits.
    pub fn text(&self) -> Option<String> {
        match self {
            Value::Sym(s) | Value::Str(s) => Some(s.clone()),
            Value::Num(n) => Some(n.to_string()),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "()"),
            Value::Bool(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Value::Num(n) => write!(f, "{n}"),
            Value::Str(s) => write!(f, "{s:?}"),
            Value::Sym(s) => write!(f, "{s}"),
            Value::List(v) => write!(f, "({})", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")),
            Value::Relation(p) => {
                let shown: Vec<String> = p.iter().take(8).map(|(a, b)| format!("({a} {b})")).collect();
                write!(f, "#pairs[{}]{{{}{}}}", p.len(), shown.join(" "), if p.len() > 8 { " …" } else { "" })
            }
            Value::Class(c) => {
                let shown: Vec<&str> = c.iter().take(12).map(String::as_str).collect();
                write!(f, "#class[{}]{{{}{}}}", c.len(), shown.join(" "), if c.len() > 12 { " …" } else { "" })
            }
            Value::Lambda(p, _, _) => write!(f, "#lambda({})", p.join(" ")),
            Value::Prim(n) => write!(f, "#prim:{n}"),
        }
    }
}

/// Read every S-expression in `src`.
pub fn read(src: &str) -> Result<Vec<Value>, String> {
    let toks = tokens(src);
    let mut i = 0;
    let mut out = Vec::new();
    while i < toks.len() {
        out.push(parse(&toks, &mut i)?);
    }
    Ok(out)
}

fn tokens(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ';' => while chars.next_back_or_skip_line() {},
            '(' | ')' | '\'' => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                out.push(c.to_string());
            }
            '"' => {
                let mut s = String::from("\"");
                for d in chars.by_ref() {
                    s.push(d);
                    if d == '"' {
                        break;
                    }
                }
                out.push(s);
            }
            c if c.is_whitespace() => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

trait SkipLine {
    fn next_back_or_skip_line(&mut self) -> bool;
}
impl<I: Iterator<Item = char>> SkipLine for std::iter::Peekable<I> {
    /// Consume to the end of the line (a `;` comment); false when done.
    fn next_back_or_skip_line(&mut self) -> bool {
        matches!(self.next(), Some(c) if c != '\n')
    }
}

fn parse(toks: &[String], i: &mut usize) -> Result<Value, String> {
    let t = toks.get(*i).ok_or("unexpected end")?;
    *i += 1;
    match t.as_str() {
        "(" => {
            let mut v = Vec::new();
            while toks.get(*i).map(String::as_str) != Some(")") {
                if *i >= toks.len() {
                    return Err("missing )".into());
                }
                v.push(parse(toks, i)?);
            }
            *i += 1;
            Ok(if v.is_empty() { Value::Nil } else { Value::List(v) })
        }
        ")" => Err("unexpected )".into()),
        "'" => Ok(Value::List(vec![Value::Sym("quote".into()), parse(toks, i)?])),
        "#t" => Ok(Value::Bool(true)),
        "#f" => Ok(Value::Bool(false)),
        s if s.starts_with('"') => Ok(Value::Str(s.trim_matches('"').to_string())),
        s => Ok(s.parse::<f64>().map(Value::Num).unwrap_or_else(|_| Value::Sym(s.to_string()))),
    }
}
