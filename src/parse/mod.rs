//! Tokens → the tree. Three small grammars — program, expression, command
//! — one file each (`expression.rs`, `command.rs`; the program is here).

mod command;
mod expression;

use crate::token::{lex, Token};
use crate::tree::Declaration;

pub struct Parser {
    pub(crate) token: Vec<Token>,
    pub(crate) at: usize,
}

/// Parses a whole source: `SECTION`/`GET` tolerated and skipped, then the
/// top-level `LET` declarations.
pub fn program(src: &str) -> Result<Vec<Declaration>, String> {
    let mut p = Parser { token: lex(src)?, at: 0 };
    let mut out = Vec::new();
    loop {
        p.seps();
        match p.peek() {
            None => return Ok(out),
            Some(Token::Key("section")) | Some(Token::Key("get")) => {
                p.at += 1;
                p.seps();
                if let Some(Token::Str(_)) = p.peek() {
                    p.at += 1;
                }
            }
            Some(Token::Key("let")) | Some(Token::Key("and")) => {
                p.at += 1;
                out.push(p.declaration()?);
            }
            Some(Token::Key("manifest")) => {
                p.at += 1;
                out.push(p.table("manifest")?);
            }
            Some(Token::Key("global")) => {
                p.at += 1;
                out.push(p.table("global")?);
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

    /// A `MANIFEST`/`GLOBAL` block: `name = k` or `name: slot`, separated.
    pub(crate) fn table(&mut self, which: &str) -> Result<Declaration, String> {
        self.seps();
        let close = if self.eat(&Token::Pun("$(")) { Token::Pun("$)") } else { self.expect(&Token::Pun("{"))?; Token::Pun("}") };
        let mut rows = Vec::new();
        loop {
            self.seps();
            if self.peek() == Some(&close) {
                self.at += 1;
                break;
            }
            let name = self.name()?;
            if !self.eat(&Token::Pun("=")) {
                self.expect(&Token::Pun(":"))?;
            }
            self.seps();
            match self.peek().cloned() {
                Some(Token::Num(k)) => {
                    self.at += 1;
                    rows.push((name, k));
                }
                other => return Err(format!("{which} needs a constant, found {other:?}")),
            }
        }
        Ok(match which {
            "manifest" => Declaration::Manifest(rows),
            _ => Declaration::Global(rows.into_iter().map(|(n, k)| (n, k as usize)).collect()),
        })
    }

    /// One `LET` item (the `LET`/`AND` keyword already taken).
    pub(crate) fn declaration(&mut self) -> Result<Declaration, String> {
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
            if self.eat(&Token::Key("be")) {
                Ok(Declaration::Routine(name, params, self.command()?))
            } else {
                self.expect(&Token::Pun("="))?;
                Ok(Declaration::Function(name, params, self.expression()?))
            }
        } else {
            self.expect(&Token::Pun("="))?;
            Ok(Declaration::Variable(name, self.expression()?))
        }
    }
}
