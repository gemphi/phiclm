//! Section brackets: `$(tag … $)tag` and the braces. A tagged close shuts
//! every section opened since its tag, as BCPL's lexer does, so the parser
//! sees one `$)` per section it closes; an untagged close, or `}`, shuts one.

use super::{Lexer, Token};

impl Lexer {
    /// `$(tag` or `$)tag` (the `$` and bracket not yet taken).
    pub(super) fn section(&mut self, open: bool) -> Result<(), String> {
        self.i += 2;
        let tag = self.tag();
        if self.skip.is_some() {
            return Ok(());
        }
        match (open, tag.is_empty()) {
            (true, _) => {
                self.sections.push(tag);
                self.push(Token::Pun("$("));
            }
            (false, true) => {
                self.sections.pop();
                self.push(Token::Pun("$)"));
            }
            (false, false) => {
                let at = self.sections.iter().rposition(|t| *t == tag).ok_or_else(|| format!("$){tag} closes no open $({tag}"))?;
                for _ in at..self.sections.len() {
                    self.push(Token::Pun("$)"));
                }
                self.sections.truncate(at);
            }
        }
        Ok(())
    }

    /// `{` opens an untagged section, `}` closes the innermost.
    pub(super) fn brace(&mut self, open: bool) {
        if self.skip.is_some() {
            return;
        }
        match open {
            true => self.sections.push(String::new()),
            false => {
                self.sections.pop();
            }
        }
    }
}
