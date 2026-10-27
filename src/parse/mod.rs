//! Tokens → the tree. Three small grammars — program, expression, command
//! — one file each (`expression.rs`, `command.rs`; the program is here).

mod command;
mod expression;

use crate::token::{lex, Token};
use crate::tree::{Declaration, Expression, Rows};

pub struct Parser {
    pub(crate) token: Vec<Token>,
    pub(crate) at: usize,
}

/// Parses a whole source: `SECTION` and `NEEDS` read and skipped, `GET
/// "libhdr"` as the header's manifests, then the top-level declarations.
pub fn program(src: &str) -> Result<Vec<Declaration>, String> {
    let mut p = Parser { token: lex(src)?, at: 0 };
    let mut out = Vec::new();
    loop {
        p.seps();
        match p.peek() {
            None => return Ok(out),
            Some(Token::Key("section")) | Some(Token::Key("needs")) => {
                p.at += 1;
                p.seps();
                if let Some(Token::Str(_)) = p.peek() {
                    p.at += 1;
                }
            }
            Some(Token::Key("get")) => {
                p.at += 1;
                p.seps();
                match p.peek().cloned() {
                    Some(Token::Str(file)) => {
                        p.at += 1;
                        if file.trim_end_matches(".h").ends_with("libhdr") {
                            let rows = crate::global::LIBHDR.iter().map(|(n, k)| (n.to_string(), Some(Expression::Num(*k)))).collect();
                            out.push(Declaration::Manifest(rows));
                        }
                    }
                    other => return Err(format!("GET needs a file name, found {other:?}")),
                }
            }
            Some(Token::Key("let")) | Some(Token::Key("and")) => {
                p.at += 1;
                out.extend(p.declaration()?);
            }
            Some(Token::Key("manifest")) => {
                p.at += 1;
                out.push(p.table("manifest")?);
            }
            Some(Token::Key("global")) => {
                p.at += 1;
                out.push(p.table("global")?);
            }
            Some(Token::Key("static")) => {
                p.at += 1;
                out.push(p.table("static")?);
            }
            Some(t) => return Err(format!("expected LET at top level, found {t:?}")),
        }
    }
}

impl Parser {
    pub(crate) fn peek(&self) -> Option<&Token> {
        self.token.get(self.at)
    }

    /// Skips separators (newlines, semicolons).
    pub(crate) fn seps(&mut self) {
        while matches!(self.peek(), Some(Token::Sep)) {
            self.at += 1;
        }
    }

    /// Takes exactly `t` next (separators skipped first).
    pub(crate) fn expect(&mut self, t: &Token) -> Result<(), String> {
        self.seps();
        if self.peek() == Some(t) {
            self.at += 1;
            Ok(())
        } else {
            Err(format!("expected {t:?}, found {:?}", self.peek()))
        }
    }

    /// Takes `t` if it is next (separators skipped when it is).
    pub(crate) fn eat(&mut self, t: &Token) -> bool {
        let keep = self.at;
        self.seps();
        if self.peek() == Some(t) {
            self.at += 1;
            true
        } else {
            self.at = keep;
            false
        }
    }

    pub(crate) fn name(&mut self) -> Result<String, String> {
        self.seps();
        match self.peek().cloned() {
            Some(Token::Name(n)) => {
                self.at += 1;
                Ok(n)
            }
            other => Err(format!("expected a name, found {other:?}")),
        }
    }

    /// A `MANIFEST`/`GLOBAL`/`STATIC` block: `name = k` or `name: k`,
    /// separated; `k` is a constant expression, folded by `trn`. A bare
    /// `name` is the previous value plus one (`bcplfe.b:2524`).
    pub(crate) fn table(&mut self, which: &str) -> Result<Declaration, String> {
        self.seps();
        let close = if self.eat(&Token::Pun("$(")) { Token::Pun("$)") } else { self.expect(&Token::Pun("{"))?; Token::Pun("}") };
        let mut rows: Rows = Vec::new();
        loop {
            self.seps();
            if self.peek() == Some(&close) {
                self.at += 1;
                break;
            }
            let name = self.name()?;
            let value = match self.eat(&Token::Pun("=")) || self.eat(&Token::Pun(":")) {
                true => Some(self.expression()?),
                false => None,
            };
            rows.push((name, value));
        }
        Ok(match which {
            "manifest" => Declaration::Manifest(rows),
            "static" => Declaration::Static(rows),
            _ => Declaration::Global(rows),
        })
    }

    /// One `LET` item (the `LET`/`AND` keyword already taken): a function,
    /// a routine, or `a, b, … = x, y, …` — one variable per name.
    pub(crate) fn declaration(&mut self) -> Result<Vec<Declaration>, String> {
        let name = self.name()?;
        if self.eat(&Token::Pun("(")) {
            let mut params = Vec::new();
            if !self.eat(&Token::Pun(")")) {
                loop {
                    params.push(self.name()?);
                    if !self.eat(&Token::Pun(",")) {
                        break;
                    }
                }
                self.expect(&Token::Pun(")"))?;
            }
            self.seps();
            return match self.eat(&Token::Key("be")) {
                true => Ok(vec![Declaration::Routine(name, params, self.command()?)]),
                false => {
                    self.expect(&Token::Pun("="))?;
                    Ok(vec![Declaration::Function(name, params, self.expression()?)])
                }
            };
        }
        let mut names = vec![name];
        while self.eat(&Token::Pun(",")) {
            names.push(self.name()?);
        }
        self.expect(&Token::Pun("="))?;
        let values = self.expressions()?;
        match values.len() == names.len() {
            true => Ok(names.into_iter().zip(values).map(|(n, e)| Declaration::Variable(n, e)).collect()),
            false => Err(format!("LET: {} names but {} values", names.len(), values.len())),
        }
    }
}
