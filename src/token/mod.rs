//! The lexer: BCPL's closed vocabulary of structure over an open vocabulary
//! of names (docs/169 §2). `KEYWORDS` is the one copy in the workspace —
//! `phient::code::bcpl::role` reads it from here. Read against `lex()` in
//! `bcplfe.b:616–985`: the `#` numbers and floating operators, the `\/ /\ \=`
//! spellings, nested comments, the string escapes, and the `$<tag $>tag`
//! conditional sections.

use std::collections::HashSet;

mod number;
mod section;
mod skip;
mod text;

#[cfg(test)]
mod tests;

/// BCPL's reserved words (`bcplfe.b` `declsyswords`, lower-cased), plus the
/// two library routines frequent enough to read as structure.
pub const KEYWORDS: &[&str] = &[
    "and", "abs", "be", "bitsperbcplword", "break", "by", "case", "do", "default", "eq", "eqv", "else", "endcase",
    "false", "finish", "fix", "float", "for", "goto", "ge", "gr", "global", "get", "if", "into", "let", "lv", "le",
    "ls", "logor", "logand", "loop", "lshift", "manifest", "mod", "ne", "needs", "neqv", "not", "of", "or", "resultis",
    "return", "rem", "rshift", "rv", "repeat", "repeatwhile", "repeatuntil", "section", "slct", "static", "switchon",
    "to", "test", "true", "then", "table", "unless", "until", "vec", "valof", "while", "xor", "writef", "writes",
];

/// One token. Keywords are matched case-blind (BCPL is upper-case by
/// tradition, modern sources are mixed); names keep their spelling.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Num(i64),
    /// A floating constant (`1.5`, `2e-3`): `s_fnum`.
    Fnum(f64),
    Name(String),
    Str(String),
    Chr(i64),
    Key(&'static str),
    Pun(&'static str),
    /// A newline outside brackets, `;`, or the `.` that ends a section:
    /// where one command may end.
    Sep,
}

const PUNCT: &[&str] = &[
    "$(", "$)", ":=", "~=", "<=", ">=", "<<", ">>", "->", "{", "}", "(", ")", "[", "]", ",", "!", "@", "+", "-", "*",
    "/", "=", "<", ">", "&", "|", "~", "?", ":", "%",
];

/// The floating operators `#op` spells (`bcplfe.b:744–757`), by the
/// integer operator after the `#`.
const FLOATING: &[(&str, &str)] = &[
    ("~=", "#~="), ("\\=", "#~="), ("<=", "#<="), (">=", "#>="), ("+", "#+"), ("-", "#-"), ("*", "#*"), ("/", "#/"),
    ("=", "#="), ("<", "#<"), (">", "#>"),
];

struct Lexer {
    b: Vec<char>,
    i: usize,
    out: Vec<Token>,
    parens: i32,
    /// The conditional-compilation tags set by `$$tag`.
    tags: HashSet<String>,
    /// While skipping a `$<tag … $>tag` section: its tag.
    skip: Option<String>,
    /// The open sections' tags, innermost last (`""` untagged).
    sections: Vec<String>,
}

/// Lexes a whole source. Newlines inside `( )` are suppressed, as the real
/// lexer suppresses the inserted semicolon there, so calls span lines.
pub fn lex(src: &str) -> Result<Vec<Token>, String> {
    let mut l = Lexer { b: src.chars().collect(), i: 0, out: Vec::new(), parens: 0, tags: HashSet::new(), skip: None, sections: Vec::new() };
    while l.i < l.b.len() {
        l.step()?;
    }
    Ok(l.out)
}


impl Lexer {
    fn at(&self, k: usize) -> Option<char> {
        self.b.get(self.i + k).copied()
    }

    fn push(&mut self, t: Token) {
        if self.skip.is_none() {
            self.out.push(t);
        }
    }

    fn step(&mut self) -> Result<(), String> {
        let c = self.b[self.i];
        match (c, self.at(1)) {
            ('\n', _) => {
                if self.parens == 0 {
                    self.push(Token::Sep);
                }
                self.i += 1;
            }
            (c, _) if c.is_whitespace() => self.i += 1,
            ('/', Some('/')) => {
                while self.i < self.b.len() && self.b[self.i] != '\n' {
                    self.i += 1;
                }
            }
            ('/', Some('*')) => self.comment()?,
            ('/', Some('\\')) => self.spell(2, Token::Pun("&")),
            ('\\', Some('/')) => self.spell(2, Token::Pun("|")),
            ('\\', Some('=')) => self.spell(2, Token::Pun("~=")),
            ('\\', _) => self.spell(1, Token::Pun("~")),
            (':', Some(':')) => self.spell(2, Token::Key("of")),
            ('.', _) => self.spell(1, Token::Sep),
            ('"', _) => self.string()?,
            ('\'', _) => self.character()?,
            ('#', _) => self.hash()?,
            ('$', Some(k @ ('<' | '>' | '$' | '~'))) => self.condition(k),
            ('$', Some(k @ ('(' | ')'))) => self.section(k == '(')?,
            (c, _) if c.is_ascii_digit() => self.decimal()?,
            (c, _) if c.is_alphabetic() || c == '_' => self.word(),
            _ => self.punct()?,
        }
        Ok(())
    }

    fn spell(&mut self, len: usize, t: Token) {
        self.i += len;
        self.push(t);
    }

    /// A run of name characters: a tag, or a word.
    fn tag(&mut self) -> String {
        let start = self.i;
        while self.i < self.b.len() && (self.b[self.i].is_alphanumeric() || self.b[self.i] == '_') {
            self.i += 1;
        }
        self.b[start..self.i].iter().collect()
    }

    fn word(&mut self) {
        let word = self.tag();
        let low = word.to_lowercase();
        match KEYWORDS.iter().find(|k| **k == low) {
            Some(k) => self.push(Token::Key(k)),
            None => self.push(Token::Name(word)),
        }
    }

    fn punct(&mut self) -> Result<(), String> {
        let c = self.b[self.i];
        let two: String = self.b[self.i..(self.i + 2).min(self.b.len())].iter().collect();
        match (PUNCT.iter().find(|p| p.len() == 2 && **p == two), PUNCT.iter().find(|p| p.len() == 1 && p.starts_with(c))) {
            (Some(p), _) => self.spell(2, Token::Pun(p)),
            (None, Some(p)) => {
                match *p {
                    "(" => self.parens += 1,
                    ")" => self.parens -= 1,
                    "{" => self.brace(true),
                    "}" => self.brace(false),
                    _ => {}
                }
                self.spell(1, Token::Pun(p));
            }
            (None, None) if c == ';' => self.spell(1, Token::Sep),
            (None, None) => return Err(format!("unlexable character {c:?}")),
        }
        Ok(())
    }
}
