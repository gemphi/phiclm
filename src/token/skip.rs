//! What the lexer passes over: comments, nested as the real lexer nests
//! them, and the sections a `$<tag` condition switches off.

use super::Lexer;

impl Lexer {
    /// `/* … */`, nesting as the real lexer nests (`bcplfe.b:800`).
    pub(super) fn comment(&mut self) -> Result<(), String> {
        self.i += 2;
        let mut depth = 1;
        while depth > 0 {
            match (self.at(0), self.at(1)) {
                (None, _) => return Err("Missing '*/'".into()),
                (Some('*'), Some('/')) => {
                    depth -= 1;
                    self.i += 2;
                }
                (Some('/'), Some('*')) => {
                    depth += 1;
                    self.i += 2;
                }
                _ => self.i += 1,
            }
        }
        Ok(())
    }

    /// `$$tag` flips a tag; `$<tag` skips to `$>tag` unless the tag is set,
    /// `$~tag` unless it is not (`bcplfe.b:664–700`).
    pub(super) fn condition(&mut self, k: char) {
        self.i += 2;
        let tag = self.tag().to_lowercase();
        match (k, self.skip.is_some()) {
            ('>', _) => {
                if self.skip.as_deref() == Some(tag.as_str()) {
                    self.skip = None;
                }
            }
            (_, true) => {}
            ('$', false) => {
                if !self.tags.remove(&tag) {
                    self.tags.insert(tag);
                }
            }
            ('<', false) => {
                if !self.tags.contains(&tag) {
                    self.skip = Some(tag);
                }
            }
            _ => {
                if self.tags.contains(&tag) {
                    self.skip = Some(tag);
                }
            }
        }
    }
}
