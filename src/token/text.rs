//! Strings and characters, with every `*` escape `rdstrch` knows.

use super::{Lexer, Token};

impl Lexer {
    /// One character of a string or character constant, its `*` escapes
    /// read (`rdstrch`, `bcplfe.b:1357`); `None` for a `*`…`*` gap.
    pub(super) fn strch(&mut self) -> Result<Option<i64>, String> {
        let c = self.at(0).ok_or("Bad string or character constant")?;
        self.i += 1;
        match c {
            '\n' => return Err("Unescaped newline character".into()),
            '*' => {}
            c => return Ok(Some(c as i64)),
        }
        let k = self.at(0).ok_or("Bad string or character constant")?;
        self.i += 1;
        Ok(Some(match k.to_ascii_uppercase() {
            '*' | '\'' | '"' => k as i64,
            'T' => 9,
            'S' => 32,
            'N' => 10,
            'E' => 27,
            'B' => 8,
            'P' => 12,
            'C' => 13,
            'X' => self.fixed(16, 2)?,
            '#' => match self.at(0) {
                Some('#') => {
                    self.i += 1;
                    self.fixed(16, 8)?
                }
                Some('u' | 'U' | 'g' | 'G') => {
                    // The encoding switch: a character is a word here, so
                    // every code is kept whole and the switch is moot.
                    self.i += 1;
                    return Ok(None);
                }
                _ => self.fixed(16, 4)?,
            },
            w if w.is_whitespace() || w == '/' => {
                // `*` white space (and `//` comments) `*`: a gap in the text.
                self.i -= 1;
                loop {
                    match (self.at(0), self.at(1)) {
                        (Some('*'), _) => {
                            self.i += 1;
                            return Ok(None);
                        }
                        (Some('/'), Some('/')) => {
                            while self.at(0).is_some_and(|c| c != '\n') {
                                self.i += 1;
                            }
                        }
                        (Some(c), _) if c.is_whitespace() => self.i += 1,
                        _ => return Err("Bad string or character constant".into()),
                    }
                }
            }
            other => return Err(format!("Bad string or character constant, *{other}")),
        }))
    }

    /// Exactly `n` digits of `radix`.
    pub(super) fn fixed(&mut self, radix: u32, n: usize) -> Result<i64, String> {
        let text: String = self.b.get(self.i..self.i + n).map(|s| s.iter().collect()).unwrap_or_default();
        self.i += n;
        i64::from_str_radix(&text, radix).map_err(|_| format!("Bad escape digits {text:?}"))
    }

    pub(super) fn string(&mut self) -> Result<(), String> {
        self.i += 1;
        let mut s = String::new();
        loop {
            match self.at(0) {
                None => return Err("Bad string constant: no closing '\"'".into()),
                Some('"') => break,
                _ => {
                    if let Some(c) = self.strch()? {
                        s.extend(char::from_u32(c as u32));
                    }
                }
            }
        }
        self.i += 1;
        self.push(Token::Str(s));
        Ok(())
    }

    pub(super) fn character(&mut self) -> Result<(), String> {
        self.i += 1;
        let mut c = self.strch()?;
        while c.is_none() {
            c = self.strch()?;
        }
        match self.at(0) {
            Some('\'') => self.i += 1,
            _ => return Err("Bad character constant".into()),
        }
        self.push(Token::Chr(c.unwrap_or(0)));
        Ok(())
    }
}
