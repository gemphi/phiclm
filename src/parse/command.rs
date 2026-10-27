//! The command grammar: blocks `$( … $)` or `{ … }`, the structured
//! commands, the labelled ones (`CASE k:`, `DEFAULT:`, `L:`), and the three
//! that start with an expression — an assignment, a routine call, a label
//! (`bcplfe.b:1955–2062`).

use super::Parser;
use crate::token::Token;
use crate::tree::{Command, Element, Expression};

impl Parser {
    pub(crate) fn command(&mut self) -> Result<Command, String> {
        self.seps();
        let mut c = self.bare()?;
        // The postfixed loops: `c REPEAT`, `c REPEATWHILE e`, `c REPEATUNTIL e`.
        loop {
            match self.peek() {
                Some(Token::Key("repeat")) => {
                    self.at += 1;
                    c = Command::Repeat(Box::new(c));
                }
                Some(Token::Key("repeatwhile")) => {
                    self.at += 1;
                    c = Command::RepeatWhile(Box::new(c), self.expression()?);
                }
                Some(Token::Key("repeatuntil")) => {
                    self.at += 1;
                    c = Command::RepeatUntil(Box::new(c), self.expression()?);
                }
                _ => return Ok(c),
            }
        }
    }

    /// The command after a label — BCPL's `rbcom`: it may be empty, when the
    /// label sits at the end of its block.
    fn labelled(&mut self) -> Result<Command, String> {
        self.seps();
        match self.peek() {
            None | Some(Token::Pun("$)")) | Some(Token::Pun("}")) => Ok(Command::Block(Vec::new())),
            _ => self.command(),
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
                    match self.peek() {
                        Some(Token::Key("let")) => {
                            self.at += 1;
                            let mut run = self.declaration()?;
                            while self.eat(&Token::Key("and")) {
                                run.extend(self.declaration()?);
                            }
                            body.push(Element::Let(run));
                        }
                        Some(Token::Key(which @ ("manifest" | "static" | "global"))) => {
                            let which: &'static str = *which;
                            self.at += 1;
                            body.push(Element::Let(vec![self.table(which)?]));
                        }
                        _ => body.push(Element::Do(self.command()?)),
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
                // `OR` is `ELSE` (`declsyswords`, `bcplfe.b:1082`).
                match self.eat(&Token::Key("else")) || self.eat(&Token::Key("or")) {
                    true => Ok(Command::Test(q, Box::new(yes), Box::new(self.command()?))),
                    false => Err(format!("ELSE missing, found {:?}", self.peek())),
                }
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
            Some(Token::Key("switchon")) => {
                self.at += 1;
                let e = self.expression()?;
                self.expect(&Token::Key("into"))?;
                self.seps();
                match self.peek() {
                    Some(Token::Pun("$(")) | Some(Token::Pun("{")) => Ok(Command::Switchon(e, Box::new(self.command()?))),
                    other => Err(format!("SWITCHON … INTO needs a block, found {other:?}")),
                }
            }
            Some(Token::Key("case")) => {
                self.at += 1;
                let k = self.expression()?;
                self.expect(&Token::Pun(":")).map_err(|_| "Bad CASE label".to_string())?;
                Ok(Command::Case(k, Box::new(self.labelled()?)))
            }
            Some(Token::Key("default")) => {
                self.at += 1;
                self.expect(&Token::Pun(":")).map_err(|_| "Bad DEFAULT label".to_string())?;
                Ok(Command::Default(Box::new(self.labelled()?)))
            }
            Some(Token::Key("endcase")) => {
                self.at += 1;
                Ok(Command::Endcase)
            }
            Some(Token::Key("goto")) => {
                self.at += 1;
                Ok(Command::Goto(self.expression()?))
            }
            _ => {
                // An expression opens: `lhs := e` or `a, b := x, y` (left to
                // right, `bcplfe.b:3540`), a routine call, or `L:`.
                let mut lhs = self.expressions()?;
                match (self.eat(&Token::Pun(":=")), lhs.len()) {
                    (true, 1) => Ok(Command::Assign(lhs.remove(0), self.expression()?)),
                    (true, n) => {
                        let rhs = self.expressions()?;
                        match rhs.len() == n {
                            true => Ok(Command::Block(lhs.into_iter().zip(rhs).map(|(l, r)| Element::Do(Command::Assign(l, r))).collect())),
                            false => Err(format!("{n} places but {} values in an assignment", rhs.len())),
                        }
                    }
                    (false, 1) => match lhs.remove(0) {
                        Expression::Name(label) if self.eat(&Token::Pun(":")) => Ok(Command::Label(label, Box::new(self.labelled()?))),
                        e @ Expression::Call(..) => Ok(Command::Do(e)),
                        e => Err(format!("a command, not a value: {e:?}")),
                    },
                    (false, _) => Err("a list of values is not a command".into()),
                }
            }
        }
    }
}
