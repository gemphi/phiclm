//! Execution is the judge (docs/169 §4): the loop a prose model can never
//! have. A source is compiled and run by a deterministic machine that
//! answers in microseconds, offline; what it did — compiled or not, what
//! it wrote, what it returned — is the verdict, and the verdict is told as
//! rows so every student can learn from it.

use crate::machine::Machine;
use crate::ocode::Program;
use phiecs::Row;

/// What happened when the source met the machine.
#[derive(Debug)]
pub struct Verdict {
    pub compiled: bool,
    pub error: Option<String>,
    pub out: String,
    pub result: i64,
    pub steps: usize,
    /// What the program wrote to each stream it opened with `findoutput`.
    pub written: Vec<(String, String)>,
}

/// Compiles a source to O-code.
pub fn compile(source: &str) -> Result<Program, String> {
    crate::trn::translate(&crate::parse::program(source)?)
}

/// Compiles and runs `start()`, within `limit` steps. Never panics on bad
/// input — generated code must be judgeable.
pub fn judge(source: &str, limit: usize) -> Verdict {
    judge_with(source, "", limit)
}

/// As [`judge`], with `input` for `rdch` and `readn` to read.
pub fn judge_with(source: &str, input: &str, limit: usize) -> Verdict {
    judge_files(source, input, &[], limit)
}

/// As [`judge_with`], with named texts `findinput` can open; what the
/// program writes to `findoutput` streams comes back in `written`.
pub fn judge_files(source: &str, input: &str, files: &[(&str, &str)], limit: usize) -> Verdict {
    let chars = |t: &str| t.chars().map(|c| c as i64).collect::<Vec<i64>>();
    let program = match compile(source) {
        Ok(p) => p,
        Err(e) => return Verdict { compiled: false, error: Some(e), out: String::new(), result: 0, steps: 0, written: Vec::new() },
    };
    let mut m = Machine::load(&program);
    m.input = chars(input);
    m.files = files.iter().map(|(n, t)| (n.to_string(), chars(t))).collect();
    let run = m.run(&program, limit);
    let (out, written) = m.outputs();
    match run {
        Ok(result) => Verdict { compiled: true, error: None, out, result, steps: m.steps, written },
        Err(e) => Verdict { compiled: true, error: Some(e), out, result: 0, steps: m.steps, written },
    }
}

impl Verdict {
    /// The verdict as rows, for any `World` or `Tally` to learn from.
    pub fn rows(&self, name: &str) -> Vec<Row> {
        let mut out = vec![
            Row::new(name, "compiled", if self.compiled { "yes" } else { "no" }),
            Row::new(name, "returned", &self.result.to_string()),
        ];
        if let Some(e) = &self.error {
            out.push(Row::new(name, "failed", e));
        }
        for line in self.out.lines().filter(|l| !l.is_empty()) {
            out.push(Row::new(name, "wrote", line));
        }
        out
    }
}
