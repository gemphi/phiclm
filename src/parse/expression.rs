//! The expression grammar, as BCPL's own `rexp(n)` climbs it
//! (`bcplfe.b:1814–1910`): a primary or a monad, then every infix operator
//! whose priority beats `n`. The priorities, lowest first: `-> ,` (0),
//! `EQV NEQV` (1), `|` (2), `&` (3), the relations and the shifts (4),
//! `+ -` (5), `* / REM` (6), and `! % OF` (8); a call `f(…)` and a subscript
//! `v[i]` bind always. The monads take their operand at their own height:
//! `! @ FLOAT FIX` at 7, `- + ABS` at 5, `~` at 3 — so `~a = b` is
//! `~(a = b)`, and `a < b < c` is `a < b & b < c`. A newline ends an
//! expression unless an operator ends the line.

use super::Parser;
use crate::token::Token;
use crate::tree::Expression;

/// What may follow an operand.
#[derive(Clone, Copy)]
enum Infix {
    Call,
    Method,
    Index,
    Cond,
    Relation(&'static str),
    Shift(&'static str),
    Plain(u8, &'static str),
}

/// The infix reading of a token, spelled canonically.
fn infix(t: &Token) -> Option<Infix> {
    Some(match t {
        Token::Pun("(") => Infix::Call,
        Token::Pun("#(") => Infix::Method,
        Token::Pun("[") => Infix::Index,
        Token::Pun("->") => Infix::Cond,
        Token::Pun("!") => Infix::Plain(8, "!"),
        Token::Pun("%") => Infix::Plain(8, "%"),
        Token::Key("of") => Infix::Plain(8, "of"),
        Token::Pun("*") => Infix::Plain(6, "*"),
        Token::Pun("/") => Infix::Plain(6, "/"),
        Token::Key("rem") | Token::Key("mod") => Infix::Plain(6, "rem"),
        Token::Pun("#*") => Infix::Plain(6, "#*"),
        Token::Pun("#/") => Infix::Plain(6, "#/"),
        Token::Pun("+") => Infix::Plain(5, "+"),
        Token::Pun("-") => Infix::Plain(5, "-"),
        Token::Pun("#+") => Infix::Plain(5, "#+"),
        Token::Pun("#-") => Infix::Plain(5, "#-"),
        Token::Pun("=") | Token::Key("eq") => Infix::Relation("="),
        Token::Pun("~=") | Token::Key("ne") => Infix::Relation("~="),
        Token::Pun("<") | Token::Key("ls") => Infix::Relation("<"),
        Token::Pun(">") | Token::Key("gr") => Infix::Relation(">"),
        Token::Pun("<=") | Token::Key("le") => Infix::Relation("<="),
        Token::Pun(">=") | Token::Key("ge") => Infix::Relation(">="),
        Token::Pun(op @ ("#=" | "#~=" | "#<" | "#>" | "#<=" | "#>=")) => Infix::Relation(*op),
        Token::Pun("<<") | Token::Key("lshift") => Infix::Shift("<<"),
        Token::Pun(">>") | Token::Key("rshift") => Infix::Shift(">>"),
        Token::Pun("&") | Token::Key("logand") => Infix::Plain(3, "&"),
        Token::Pun("|") | Token::Key("logor") => Infix::Plain(2, "|"),
        Token::Key("eqv") => Infix::Plain(1, "eqv"),
        Token::Key("neqv") | Token::Key("xor") => Infix::Plain(1, "neqv"),
        _ => return None,
    })
}

impl Parser {
    /// A full expression: `rexp(0)`.
    pub(crate) fn expression(&mut self) -> Result<Expression, String> {
        self.rexp(0)
    }

    /// A comma-separated list of full expressions (`rexplist`).
    pub(crate) fn expressions(&mut self) -> Result<Vec<Expression>, String> {
        let mut list = vec![self.expression()?];
        while self.eat(&Token::Pun(",")) {
            list.push(self.expression()?);
        }
        Ok(list)
    }

    /// An operand, then every infix operator that binds tighter than `n`.
    /// Separators are not skipped here: a newline before an operator ends
    /// the expression, as BCPL's `nlpending` does.
    pub(crate) fn rexp(&mut self, n: u8) -> Result<Expression, String> {
        let mut a = self.rbexp()?;
        loop {
            let next = self.peek().and_then(infix);
            match next {
                Some(Infix::Call) => {
                    self.at += 1;
                    let mut args = Vec::new();
                    self.seps();
                    if self.peek() != Some(&Token::Pun(")")) {
                        args = self.expressions()?;
                    }
                    self.expect(&Token::Pun(")"))?;
                    a = Expression::Call(Box::new(a), args);
                }
                Some(Infix::Method) => {
                    // `E#(obj, args…)` is `((!obj)!E)(obj, args…)` (`bcplfe.b:1830`).
                    self.at += 1;
                    let args = self.expressions()?;
                    self.expect(&Token::Pun(")"))?;
                    let table = Expression::Monad("!", Box::new(args[0].clone()));
                    a = Expression::Call(Box::new(Expression::Dyad("!", Box::new(table), Box::new(a))), args);
                }
                Some(Infix::Index) => {
                    self.at += 1;
                    let i = self.expression()?;
                    self.expect(&Token::Pun("]"))?;
                    a = Expression::Dyad("!", Box::new(a), Box::new(i));
                }
                Some(Infix::Cond) if n < 1 => {
                    self.at += 1;
                    let yes = self.expression()?;
                    self.expect(&Token::Pun(","))?;
                    let no = self.expression()?;
                    a = Expression::Cond(Box::new(a), Box::new(yes), Box::new(no));
                }
                Some(Infix::Relation(op)) if n < 4 => {
                    self.at += 1;
                    let mut b = self.rexp(4)?;
                    a = Expression::Dyad(op, Box::new(a), Box::new(b.clone()));
                    // `a < b < c` is `a < b & b < c` (`bcplfe.b:1876`).
                    while let Some(Infix::Relation(op)) = self.peek().and_then(infix) {
                        self.at += 1;
                        let c = b;
                        b = self.rexp(4)?;
                        a = Expression::Dyad("&", Box::new(a), Box::new(Expression::Dyad(op, Box::new(c), Box::new(b.clone()))));
                    }
                }
                Some(Infix::Shift(op)) if n < 4 => {
                    self.at += 1;
                    a = Expression::Dyad(op, Box::new(a), Box::new(self.rexp(4)?));
                }
                Some(Infix::Plain(p, op)) if n < p => {
                    self.at += 1;
                    a = Expression::Dyad(op, Box::new(a), Box::new(self.rexp(p)?));
                }
                _ => return Ok(a),
            }
        }
    }

    fn monad(&mut self, op: &'static str, at: u8) -> Result<Expression, String> {
        Ok(Expression::Monad(op, Box::new(self.rexp(at)?)))
    }

    /// A primary or a monad (`rbexp`, `bcplfe.b:1700–1812`). An operand may
    /// open on the next line, after an operator that ended the last.
    fn rbexp(&mut self) -> Result<Expression, String> {
        self.seps();
        let t = self.peek().cloned().ok_or("expected an expression, found the end")?;
        self.at += 1;
        Ok(match t {
            Token::Num(n) | Token::Chr(n) => Expression::Num(n),
            Token::Fnum(f) => Expression::Fnum(f),
            Token::Str(s) => Expression::Str(s),
            Token::Name(n) => Expression::Name(n),
            Token::Key("true") => Expression::True,
            Token::Key("false") => Expression::False,
            Token::Key(k @ ("writef" | "writes")) => Expression::Name(k.into()),
            Token::Key("bitsperbcplword") => Expression::Num(i64::BITS as i64),
            Token::Key("valof") => Expression::Valof(Box::new(self.command()?)),
            Token::Key("vec") => Expression::Vec(Box::new(self.expression()?)),
            Token::Key("table") => Expression::Table(self.expressions()?),
            Token::Key("slct") => {
                // `SLCT offset`, `SLCT sh:offset`, or `SLCT len:sh:offset`.
                let mut part = vec![self.rexp(9)?];
                while part.len() < 3 && self.eat(&Token::Pun(":")) {
                    part.push(self.rexp(9)?);
                }
                while part.len() < 3 {
                    part.insert(0, Expression::Num(0));
                }
                let [len, sh, offset]: [Expression; 3] = part.try_into().map_err(|_| "SLCT has three parts")?;
                Expression::Slct(Box::new(len), Box::new(sh), Box::new(offset))
            }
            Token::Pun("(") => {
                let inner = self.expression()?;
                self.expect(&Token::Pun(")"))?;
                inner
            }
            Token::Pun("?") => Expression::Num(0),
            Token::Pun("!") | Token::Key("rv") => self.monad("!", 7)?,
            Token::Pun("@") | Token::Key("lv") => self.monad("@", 7)?,
            Token::Key("float") => self.monad("float", 7)?,
            Token::Key("fix") => self.monad("fix", 7)?,
            Token::Pun("+") | Token::Pun("#+") => self.rexp(5)?,
            Token::Pun("-") => match self.rexp(5)? {
                // `-5` is the number -5 (`bcplfe.b:1795`).
                Expression::Num(k) => Expression::Num(k.wrapping_neg()),
                a => Expression::Monad("-", Box::new(a)),
            },
            Token::Pun("#-") => self.monad("#-", 5)?,
            Token::Key("abs") => self.monad("abs", 5)?,
            Token::Pun("#abs") => self.monad("#abs", 5)?,
            Token::Pun("~") | Token::Key("not") => self.monad("~", 3)?,
            other => return Err(format!("expected an expression, found {other:?}")),
        })
    }
}
