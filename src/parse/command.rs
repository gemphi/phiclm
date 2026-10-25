//! The command grammar: blocks `$( … $)` or `{ … }`, the structured
//! commands, and the two commands that start with an expression — an
//! assignment and a routine call.

use super::Parser;
use crate::token::Token;
use crate::tree::{Command, Element, Expression};

impl Parser {
    pub(crate) fn command(&mut self) -> Result<Command, String> {
        self.seps();
        let mut c = self.bare()?;
        // The postfixed loops: `c REPEAT`, `c REPEATWHILE e`, `c REPEATUNTIL e`.
        loop {
            if self.peek() == Some(&Token::Key("repeat")) {
                self.at += 1;
                c = Command::Repeat(Box::new(c));
            } else if self.peek() == Some(&Token::Key("repeatwhile")) {
                self.at += 1;
                c = Command::RepeatWhile(Box::new(c), self.expression()?);
            } else if self.peek() == Some(&Token::Key("repeatuntil")) {
                self.at += 1;
                c = Command::RepeatUntil(Box::new(c), self.expression()?);
            } else {
                return Ok(c);
            }
        }
    }

    fn bare(&mut self) -> Result<Command, String> {
        match self.peek().cloned() {
            Some(Token::Pun("$(")) | Some(Token::Pun("{")) => {
                let close = if self.peek() == Some(&Token::Pun("$(")) { Token::Pun("$)") } else { Token::Pun("}") };
                self.at += 1;
                let mut body = Vec::new();
                loop {
                    self.seps();
                    if self.peek() == Some(&close) {
                        self.at += 1;
                        return Ok(Command::Block(body));
                    }
                    if self.peek() == Some(&Token::Key("let")) {
                        self.at += 1;
                        let mut run = vec![self.declaration()?];
                        while self.eat(&Token::Key("and")) {
                            run.push(self.declaration()?);
                        }
                        body.push(Element::Let(run));
                    } else {
                        body.push(Element::Do(self.command()?));
                    }
                }
            }
            Some(Token::Key("if")) => {
                self.at += 1;
                let q = self.expression()?;
                let _ = self.eat(&Token::Key("do")) || self.eat(&Token::Key("then"));
                Ok(Command::If(q, Box::new(self.command()?)))
            }
            Some(Token::Key("unless")) => {
                self.at += 1;
                let q = self.expression()?;
                let _ = self.eat(&Token::Key("do")) || self.eat(&Token::Key("then"));
                Ok(Command::Unless(q, Box::new(self.command()?)))
            }
            Some(Token::Key("test")) => {
                self.at += 1;
                let q = self.expression()?;
                let _ = self.eat(&Token::Key("then")) || self.eat(&Token::Key("do"));
                let yes = self.command()?;
                self.seps();
                self.expect(&Token::Key("else"))?;
                Ok(Command::Test(q, Box::new(yes), Box::new(self.command()?)))
            }
            Some(Token::Key("while")) => {
                self.at += 1;
                let q = self.expression()?;
                let _ = self.eat(&Token::Key("do"));
                Ok(Command::While(q, Box::new(self.command()?)))
            }
            Some(Token::Key("until")) => {
                self.at += 1;
                let q = self.expression()?;
                let _ = self.eat(&Token::Key("do"));
                Ok(Command::Until(q, Box::new(self.command()?)))
            }
            Some(Token::Key("for")) => {
                self.at += 1;
                let var = self.name()?;
                self.expect(&Token::Pun("="))?;
                let from = self.expression()?;
                self.expect(&Token::Key("to"))?;
                let to = self.expression()?;
                let by = if self.eat(&Token::Key("by")) { Some(self.expression()?) } else { None };
                let _ = self.eat(&Token::Key("do"));
                Ok(Command::For(var, from, to, by, Box::new(self.command()?)))
            }
            Some(Token::Key("resultis")) => {
                self.at += 1;
                Ok(Command::Resultis(self.expression()?))
            }
            Some(Token::Key("return")) => {
                self.at += 1;
                Ok(Command::Return)
            }
            Some(Token::Key("finish")) => {
                self.at += 1;
                Ok(Command::Finish)
            }
            Some(Token::Key("break")) => {
                self.at += 1;
                Ok(Command::Break)
            }
            Some(Token::Key("loop")) => {
                self.at += 1;
                Ok(Command::Loop)
            }
            _ => {
                // An expression opens: `lhs := e`, or a routine call.
                let e = self.expression()?;
                if self.eat(&Token::Pun(":=")) {
                    Ok(Command::Assign(e, self.expression()?))
                } else if matches!(e, Expression::Call(..)) {
                    Ok(Command::Do(e))
                } else {
                    Err(format!("a command, not a value: {e:?}"))
                }
            }
        }
    }
}
