//! The labels a routine sets, found before its code is laid down.

use crate::tree::{Command, Declaration, Element, Expression};
use std::collections::HashSet;

/// The labels a routine's body sets, so a label's name means its address
/// before and after it is set (`decllabels`, `bcplfe.b:2847`).
pub(super) fn command_labels(c: &Command, out: &mut HashSet<String>) {
    match c {
        Command::Label(name, body) => {
            out.insert(name.clone());
            command_labels(body, out);
        }
        Command::Block(body) => {
            for e in body {
                match e {
                    Element::Do(c) => command_labels(c, out),
                    Element::Let(run) => {
                        for d in run {
                            if let Declaration::Variable(_, e) = d {
                                expression_labels(e, out);
                            }
                        }
                    }
                }
            }
        }
        Command::Assign(a, b) => {
            expression_labels(a, out);
            expression_labels(b, out);
        }
        Command::Do(e) | Command::Resultis(e) | Command::Goto(e) => expression_labels(e, out),
        Command::If(q, c) | Command::Unless(q, c) | Command::While(q, c) | Command::Until(q, c) | Command::Switchon(q, c) | Command::Case(q, c) => {
            expression_labels(q, out);
            command_labels(c, out);
        }
        Command::RepeatWhile(c, q) | Command::RepeatUntil(c, q) => {
            command_labels(c, out);
            expression_labels(q, out);
        }
        Command::Repeat(c) | Command::Default(c) | Command::For(.., c) => command_labels(c, out),
        Command::Test(q, a, b) => {
            expression_labels(q, out);
            command_labels(a, out);
            command_labels(b, out);
        }
        Command::Return | Command::Finish | Command::Break | Command::Loop | Command::Endcase => {}
    }
}

pub(super) fn expression_labels(e: &Expression, out: &mut HashSet<String>) {
    match e {
        Expression::Valof(c) => command_labels(c, out),
        Expression::Monad(_, a) | Expression::Vec(a) => expression_labels(a, out),
        Expression::Dyad(_, a, b) => {
            expression_labels(a, out);
            expression_labels(b, out);
        }
        Expression::Cond(a, b, c) | Expression::Slct(a, b, c) => {
            for x in [a, b, c] {
                expression_labels(x, out);
            }
        }
        Expression::Call(f, args) => {
            expression_labels(f, out);
            for a in args {
                expression_labels(a, out);
            }
        }
        Expression::Table(items) => {
            for a in items {
                expression_labels(a, out);
            }
        }
        Expression::Num(_) | Expression::Fnum(_) | Expression::Name(_) | Expression::Str(_) | Expression::True | Expression::False => {}
    }
}
