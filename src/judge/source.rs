//! The source generator the judge is tested against: well-formed BCPL
//! through truncated, unbalanced and random byte sequences, plus programs
//! that will not stop. Drawn from [`phistd::gen`], so a failing case
//! replays from `(seed, case)` alone.

use phistd::gen::{Gen, Seeded};

/// Where a drawn source came from, so a failure says which arm broke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A whole program that compiles and stops.
    Fine,
    /// A whole program cut at a drawn character.
    Truncated,
    /// A whole program with one bracket dropped or doubled.
    Unbalanced,
    /// Characters drawn from the language's own alphabet and beyond it.
    Bytes,
    /// A program whose `start()` loops forever in a bounded store: the step
    /// cap is the only thing that stops it.
    Endless,
    /// A program whose recursion never bottoms out. It runs its frames into
    /// the heap first, so the store fault reaches it before either cap does
    /// — which is why the two are separate arms.
    Runaway,
}

/// One drawn source text and the arm it was drawn from.
#[derive(Clone, Debug)]
pub struct Source {
    pub kind: Kind,
    pub text: String,
}

/// The well-formed arm: five shapes, each with a drawn number in it, so the
/// arm is a family rather than one program seen 64 times.
fn fine(g: &mut Seeded) -> String {
    let n = g.below(12) as i64;
    let m = g.below(90) as i64 + 1;
    match g.below(5) {
        0 => format!("LET start() = VALOF $( RESULTIS {n} + {m} $)"),
        1 => format!(
            "LET start() = VALOF
             $( LET s = 0
                FOR i = 1 TO {m} DO s := s + i
                RESULTIS s
             $)"
        ),
        2 => format!(
            "LET fact(n) = n = 0 -> 1, n * fact(n - 1)
             LET start() = VALOF $( RESULTIS fact({n}) $)"
        ),
        3 => format!(
            "LET start() = VALOF
             $( writef(\"%n*n\", {m})
                RESULTIS {n}
             $)"
        ),
        _ => format!(
            "LET start() = VALOF
             $( LET v = VEC {m}
                FOR i = 0 TO {m} DO v!i := i
                RESULTIS v!{n}
             $)"
        ),
    }
}

/// The alphabet the random arm draws from: the language's punctuation and
/// section brackets, letters and digits, the quote marks that open a string
/// the text may never close, and bytes outside ASCII.
const ALPHABET: &[char] = &[
    '$', '(', ')', '!', '%', '@', '*', '+', '-', '/', ':', ';', ',', '.', '=', '<', '>', '#', '"',
    '\'', '\\', '\n', ' ', '\t', 'L', 'E', 'T', 'a', 'n', 'x', 'V', 'A', 'O', 'F', 'R', 'S', 'I',
    '0', '1', '7', '9', '_', '|', '&', '~', '[', ']', '{', '}', '?', 'é', '§', '\u{0}', '\u{7f}',
    '\u{fffd}',
];

/// The endless arm: two ways to loop forever without growing the store, so
/// the step cap is what stops them.
fn endless(g: &mut Seeded) -> String {
    match g.below(2) {
        0 => "LET start() = VALOF $( LET n = 0\n n := n + 1\n IF n < 0 RESULTIS n $) REPEAT"
            .to_string(),
        _ => "LET start() = VALOF $( LET n = 0\n WHILE TRUE DO n := n + 1\n RESULTIS n $)"
            .to_string(),
    }
}

impl Gen for Source {
    fn draw(g: &mut Seeded) -> Source {
        // Every arm is drawn, and the endless one often enough that the cap
        // is exercised in a 64-case run.
        let kind = *g.pick(&[
            Kind::Fine,
            Kind::Fine,
            Kind::Truncated,
            Kind::Unbalanced,
            Kind::Bytes,
            Kind::Endless,
            Kind::Runaway,
        ]);
        let text = match kind {
            Kind::Fine => fine(g),
            Kind::Truncated => {
                let whole = fine(g);
                let chars: Vec<char> = whole.chars().collect();
                let cut = g.below(chars.len() as u64 + 1) as usize;
                chars[..cut].iter().collect()
            }
            Kind::Unbalanced => {
                let whole = fine(g);
                let brackets: Vec<usize> = whole
                    .char_indices()
                    .filter(|(_, c)| matches!(c, '(' | ')' | '$'))
                    .map(|(i, _)| i)
                    .collect();
                match brackets.is_empty() {
                    true => whole,
                    false => {
                        let at = *g.pick(&brackets);
                        let c = whole[at..].chars().next().unwrap_or('$');
                        let mut out = whole.clone();
                        match g.below(2) == 0 {
                            true => {
                                out.replace_range(at..at + c.len_utf8(), "");
                            }
                            false => out.insert(at, c),
                        }
                        out
                    }
                }
            }
            Kind::Bytes => {
                let n = g.below(240) as usize;
                (0..n).map(|_| *g.pick(ALPHABET)).collect()
            }
            Kind::Endless => endless(g),
            Kind::Runaway => format!(
                "LET down(n) = n < 0 -> down(n), down(n - {})
                 LET start() = VALOF $( RESULTIS down(1) $)",
                g.below(2)
            ),
        };
        Source { kind, text }
    }
}
