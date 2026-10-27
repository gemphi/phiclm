//! Numbers: `#` in binary, octal and hex (or a floating operator after it),
//! decimals with underlines, and the floating constants.

use super::{Lexer, Token, FLOATING};

impl Lexer {
    /// Digits of `radix`, underlines ignored (`readnumber`, `bcplfe.b:1331`).
    pub(super) fn digits(&mut self, radix: u32) -> Result<i64, String> {
        let start = self.i;
        while self.at(0).is_some_and(|c| c == '_' || c.is_digit(radix)) {
            self.i += 1;
        }
        let text: String = self.b[start..self.i].iter().filter(|c| **c != '_').collect();
        match text.is_empty() {
            true => Err("Bad number".into()),
            false => u64::from_str_radix(&text, radix).map(|n| n as i64).map_err(|e| format!("number {text}: {e}")),
        }
    }

    /// `#` opens a binary, octal or hex number, or a floating operator.
    pub(super) fn hash(&mut self) -> Result<(), String> {
        self.i += 1;
        let n = match self.at(0) {
            Some('0'..='7') => self.digits(8)?,
            Some('b' | 'B') => {
                self.i += 1;
                self.digits(2)?
            }
            Some('o' | 'O') => {
                self.i += 1;
                self.digits(8)?
            }
            Some('x' | 'X') => {
                self.i += 1;
                self.digits(16)?
            }
            Some('(') => {
                // `E#(obj, args…)`: a method call, a bracket like a call's.
                self.i += 1;
                self.parens += 1;
                self.push(Token::Pun("#("));
                return Ok(());
            }
            _ => {
                let rest: String = self.b[self.i..(self.i + 3).min(self.b.len())].iter().collect();
                if rest.to_lowercase() == "abs" {
                    self.i += 3;
                    self.push(Token::Pun("#abs"));
                    return Ok(());
                }
                return match FLOATING.iter().find(|(op, _)| rest.starts_with(op)) {
                    Some((op, f)) => {
                        self.i += op.chars().count();
                        self.push(Token::Pun(f));
                        Ok(())
                    }
                    None => Err("'#' out of context".into()),
                };
            }
        };
        self.push(Token::Num(n));
        Ok(())
    }

    /// A decimal integer, or a floating constant when a `.` or an exponent
    /// appears (`readdecimal`, `bcplfe.b:1259`).
    pub(super) fn decimal(&mut self) -> Result<(), String> {
        let mut text = String::new();
        let mut floating = false;
        while let Some(c) = self.at(0).filter(|c| c.is_ascii_digit() || *c == '_' || *c == '.') {
            match c {
                '_' => {}
                '.' if floating => return Err("Bad floating point constant".into()),
                '.' => {
                    floating = true;
                    text.push('.');
                }
                d => text.push(d),
            }
            self.i += 1;
        }
        if matches!(self.at(0), Some('e' | 'E')) {
            floating = true;
            self.i += 1;
            text.push('e');
            if self.at(0) == Some('-') {
                self.i += 1;
                text.push('-');
            }
            while let Some(c) = self.at(0).filter(|c| c.is_ascii_digit() || *c == '_') {
                if c != '_' {
                    text.push(c);
                }
                self.i += 1;
            }
            if text.ends_with(['e', '-']) {
                text.push('0');
            }
        }
        let t = match floating {
            true => Token::Fnum(text.parse().map_err(|e| format!("number {text}: {e}"))?),
            false => Token::Num(text.parse().map_err(|e| format!("number {text}: {e}"))?),
        };
        self.push(t);
        Ok(())
    }
}
