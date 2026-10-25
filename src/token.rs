//! The lexer: BCPL's closed vocabulary of structure over an open vocabulary
//! of names (docs/169 §2). `KEYWORDS` is the one copy in the workspace —
//! `phient::code::bcpl::role` reads it from here.

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
    Name(String),
    Str(String),
    Chr(i64),
    Key(&'static str),
    Pun(&'static str),
    /// A newline outside brackets, or `;`: where one command may end.
    Sep,
}

const PUNCT: &[&str] = &[
    "$(", "$)", ":=", "~=", "<=", ">=", "<<", ">>", "->", "{", "}", "(", ")", ",", "!", "@", "+", "-", "*", "/",
    "=", "<", ">", "&", "|", "~", "?", ":",
];

/// The string escapes: `*n *t *s ** *" *'`.
fn escape(c: char) -> char {
    match c.to_ascii_lowercase() {
        'n' => '\n',
        't' => '\t',
        's' => ' ',
        other => other,
    }
}

/// Lexes a whole source. Newlines inside `( )` are suppressed, as the real
/// lexer suppresses the inserted semicolon there, so calls span lines.
pub fn lex(src: &str) -> Result<Vec<Token>, String> {
    let b: Vec<char> = src.chars().collect();
    let (mut i, mut out, mut parens) = (0usize, Vec::new(), 0i32);
    while i < b.len() {
        let c = b[i];
        if c == '\n' {
            if parens == 0 {
                out.push(Token::Sep);
            }
            i += 1;
        } else if c.is_whitespace() {
            i += 1;
        } else if c == '/' && b.get(i + 1) == Some(&'/') {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && b.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                i += 1;
            }
            i += 2;
        } else if c == '"' {
            let mut s = String::new();
            i += 1;
            while i < b.len() && b[i] != '"' {
                if b[i] == '*' && i + 1 < b.len() {
                    s.push(escape(b[i + 1]));
                    i += 2;
                } else {
                    s.push(b[i]);
                    i += 1;
                }
            }
            i += 1;
            out.push(Token::Str(s));
        } else if c == '\'' {
            i += 1;
            let ch = if b[i] == '*' {
                i += 1;
                escape(b[i])
            } else {
                b[i]
            };
            i += 2;
            out.push(Token::Chr(ch as i64));
        } else if c == '#' {
            i += 1;
            let radix = match b.get(i).map(|c| c.to_ascii_lowercase()) {
                Some('x') => {
                    i += 1;
                    16
                }
                Some('b') => {
                    i += 1;
                    2
                }
                _ => 8,
            };
            let start = i;
            while i < b.len() && b[i].is_ascii_alphanumeric() {
                i += 1;
            }
            let text: String = b[start..i].iter().collect();
            out.push(Token::Num(i64::from_str_radix(&text, radix).map_err(|e| format!("number #{text}: {e}"))?));
        } else if c.is_ascii_digit() {
            let start = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            let text: String = b[start..i].iter().collect();
            out.push(Token::Num(text.parse().map_err(|e| format!("number {text}: {e}"))?));
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                i += 1;
            }
            let word: String = b[start..i].iter().collect();
            let low = word.to_lowercase();
            match KEYWORDS.iter().find(|k| **k == low) {
                Some(k) => out.push(Token::Key(k)),
                None => out.push(Token::Name(word)),
            }
        } else if c == '$' && b.get(i + 1) == Some(&'(') {
            i += 2;
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                i += 1; // the section tag
            }
            out.push(Token::Pun("$("));
        } else if c == '$' && b.get(i + 1) == Some(&')') {
            i += 2;
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                i += 1;
            }
            out.push(Token::Pun("$)"));
        } else {
            let two: String = b[i..(i + 2).min(b.len())].iter().collect();
            if let Some(p) = PUNCT.iter().find(|p| p.len() == 2 && **p == two) {
                i += 2;
                if *p == "(" { /* unreachable */ }
                out.push(Token::Pun(p));
            } else if let Some(p) = PUNCT.iter().find(|p| p.len() == 1 && p.chars().next() == Some(c)) {
                i += 1;
                parens += match *p {
                    "(" => 1,
                    ")" => -1,
                    _ => 0,
                };
                if *p == ";" { /* unreachable */ }
                out.push(Token::Pun(p));
            } else if c == ';' {
                i += 1;
                out.push(Token::Sep);
            } else {
                return Err(format!("unlexable character {c:?}"));
            }
        }
    }
    Ok(out)
}
