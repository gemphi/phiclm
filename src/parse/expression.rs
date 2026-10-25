//! The expression grammar, by precedence (Crick 2004 App. D, compressed):
//! `-> ,` lowest, then `|`, `&`, `~`, the relations, the shifts, `+ -`,
//! `* / rem`, the monads, and the primaries with their postfixes — a call
//! `f(…)` and the subscript `a!i`, which binds tightest of all.

use super::Parser;
use crate::token::Token;
use crate::tree::Expression;

impl Parser {
    /// A full expression: the conditional `q -> a, b` sits lowest.
    pub(crate) fn expression(&mut self) -> Result<Expression, String> {
        let q = self.ors()?;
        if self.eat(&Token::Pun("->")) {
            let yes = self.expression()?;
            self.expect(&Token::Pun(","))?;
            let no = self.expression()?;
            return Ok(Expression::Cond(Box::new(q), Box::new(yes), Box::new(no)));
        }
        Ok(q)
    }

    fn dyadic(&mut self, ops: &[(&Token, &'static str)], below: fn(&mut Self) -> Result<Expression, String>) -> Result<Expression, String> {
        let mut left = below(self)?;
        'grow: loop {
            for (t, name) in ops {
                if self.peek() == Some(*t) {
                    self.at += 1;
                    let right = below(self)?;
                    left = Expression::Dyad(name, Box::new(left), Box::new(right));
                    continue 'grow;
                }
            }
            return Ok(left);
        }
    }

    fn ors(&mut self) -> Result<Expression, String> {
        self.dyadic(&[(&Token::Pun("|"), "|"), (&Token::Key("logor"), "|")], Self::ands)
    }

    fn ands(&mut self) -> Result<Expression, String> {
        self.dyadic(&[(&Token::Pun("&"), "&"), (&Token::Key("logand"), "&")], Self::relation)
    }

    fn relation(&mut self) -> Result<Expression, String> {
        self.dyadic(
            &[
                (&Token::Pun("="), "="), (&Token::Pun("~="), "~="), (&Token::Key("eq"), "="), (&Token::Key("ne"), "~="),
                (&Token::Pun("<="), "<="), (&Token::Pun(">="), ">="), (&Token::Pun("<"), "<"), (&Token::Pun(">"), ">"),
                (&Token::Key("le"), "<="), (&Token::Key("ge"), ">="), (&Token::Key("ls"), "<"), (&Token::Key("gr"), ">"),
            ],
            Self::shift,
        )
    }

    fn shift(&mut self) -> Result<Expression, String> {
        self.dyadic(&[(&Token::Pun("<<"), "<<"), (&Token::Pun(">>"), ">>"), (&Token::Key("lshift"), "<<"), (&Token::Key("rshift"), ">>")], Self::sum)
    }

    fn sum(&mut self) -> Result<Expression, String> {
        self.dyadic(&[(&Token::Pun("+"), "+"), (&Token::Pun("-"), "-")], Self::product)
    }

    fn product(&mut self) -> Result<Expression, String> {
        self.dyadic(&[(&Token::Pun("*"), "*"), (&Token::Pun("/"), "/"), (&Token::Key("rem"), "rem"), (&Token::Key("mod"), "rem")], Self::monad)
    }

    fn monad(&mut self) -> Result<Expression, String> {
        for (t, name) in [
            (Token::Pun("-"), "-"), (Token::Pun("!"), "!"), (Token::Key("rv"), "!"),
            (Token::Pun("@"), "@"), (Token::Key("lv"), "@"), (Token::Pun("~"), "~"),
            (Token::Key("not"), "~"), (Token::Key("abs"), "abs"),
        ] {
            if self.peek() == Some(&t) {
                self.at += 1;
                return Ok(Expression::Monad(name, Box::new(self.monad()?)));
            }
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expression, String> {
        let mut e = match self.peek().cloned() {
            Some(Token::Num(n)) => {
                self.at += 1;
                Expression::Num(n)
            }
            Some(Token::Chr(n)) => {
                self.at += 1;
                Expression::Num(n)
            }
            Some(Token::Str(s)) => {
                self.at += 1;
                Expression::Str(s)
            }
            Some(Token::Name(n)) => {
                self.at += 1;
                Expression::Name(n)
            }
            Some(Token::Key("true")) => {
                self.at += 1;
                Expression::True
            }
            Some(Token::Key("false")) => {
                self.at += 1;
                Expression::False
            }
            Some(Token::Key("writef")) => {
                self.at += 1;
                Expression::Name("writef".into())
            }
            Some(Token::Key("writes")) => {
                self.at += 1;
                Expression::Name("writes".into())
            }
            Some(Token::Key("valof")) => {
                self.at += 1;
                Expression::Valof(Box::new(self.command()?))
            }
            Some(Token::Key("vec")) => {
                self.at += 1;
                self.seps();
                match self.peek().cloned() {
                    Some(Token::Num(k)) => {
                        self.at += 1;
                        Expression::Vec(k)
                    }
                    other => return Err(format!("VEC needs a constant size, found {other:?}")),
                }
            }
            Some(Token::Pun("(")) => {
                self.at += 1;
                let inner = self.expression()?;
                self.expect(&Token::Pun(")"))?;
                inner
            }
            Some(Token::Pun("?")) => {
                self.at += 1;
                Expression::Num(0)
            }
            other => return Err(format!("expected an expression, found {other:?}")),
        };
        // Postfixes bind tightest: a call, and the subscript `a!i`.
        loop {
            if self.peek() == Some(&Token::Pun("(")) {
                self.at += 1;
                let mut args = Vec::new();
                if self.peek() != Some(&Token::Pun(")")) {
                    loop {
                        args.push(self.expression()?);
                        if !self.eat(&Token::Pun(",")) {
                            break;
                        }
                    }
                }
                self.expect(&Token::Pun(")"))?;
                e = Expression::Call(Box::new(e), args);
            } else if self.peek() == Some(&Token::Pun("!")) {
                self.at += 1;
                let i = self.monad()?;
                e = Expression::Dyad("!", Box::new(e), Box::new(i));
            } else {
                return Ok(e);
            }
        }
    }
}
