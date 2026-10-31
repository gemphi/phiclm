//! Execution is the judge (docs/169 §4): the loop a prose model can never
//! have. A source is compiled and run by a deterministic machine that
//! answers in microseconds, offline; what it did — compiled or not, what
//! it wrote, what it returned — is the verdict, and the verdict is told as
//! rows so every student can learn from it.
//!
//! The judge answers under two declared caps, [`STEP_CAP`] and
//! [`SECONDS_CAP`], because a judged source is generated source: it may not
//! stop, and it may not be a program at all. Every outcome comes back as a
//! [`Verdict`] carrying all five fields — compiled, error, out, result,
//! steps — and the cap it reached, if it reached one. The judge returns a
//! `Verdict` rather than panicking on any source text, truncated,
//! unbalanced and random bytes included.

use crate::machine::Machine;
use crate::ocode::Program;
use phiecs::Row;
use std::time::{Duration, Instant};

/// Which cap a run reached, at the judge's path.
pub use crate::machine::Capped as Cap;

/// The declared step cap: 1,000,000 machine steps.
pub const STEP_CAP: usize = 1_000_000;

/// The declared wall-clock cap: 10 seconds.
pub const SECONDS_CAP: u64 = 10;

/// The two caps a run is held to. `seconds` is `None` where no clock is
/// watched — the old step-only entry points, whose behaviour is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caps {
    pub steps: usize,
    pub seconds: Option<u64>,
}

impl Caps {
    /// The declared pair: [`STEP_CAP`] steps and [`SECONDS_CAP`] seconds.
    pub fn declared() -> Caps {
        Caps { steps: STEP_CAP, seconds: Some(SECONDS_CAP) }
    }

    /// A step cap alone, with no clock watched.
    pub fn steps(limit: usize) -> Caps {
        Caps { steps: limit, seconds: None }
    }

    fn deadline(&self, from: Instant) -> Option<Instant> {
        self.seconds.map(|s| from + Duration::from_secs(s))
    }
}

impl Default for Caps {
    fn default() -> Caps {
        Caps::declared()
    }
}

/// What happened when the source met the machine.
#[derive(Debug)]
pub struct Verdict {
    pub compiled: bool,
    pub error: Option<String>,
    pub out: String,
    pub result: i64,
    pub steps: usize,
    /// The cap the run reached, when it reached one.
    pub capped: Option<Cap>,
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
    judge_under(source, input, files, Caps::steps(limit))
}

/// The capped entry point: compiles and runs `start()` under the declared
/// [`Caps::declared`] pair — 1,000,000 steps and 10 seconds.
pub fn judge_capped(source: &str) -> Verdict {
    judge_under(source, "", &[], Caps::declared())
}

/// As [`judge_files`], under a declared pair of caps rather than a step
/// limit alone. A run stopped by either cap comes back compiled, with the
/// error the machine gave and `capped` naming the cap.
pub fn judge_under(source: &str, input: &str, files: &[(&str, &str)], caps: Caps) -> Verdict {
    let chars = |t: &str| t.chars().map(|c| c as i64).collect::<Vec<i64>>();
    let program = match compile(source) {
        Ok(p) => p,
        Err(e) => {
            return Verdict {
                compiled: false,
                error: Some(e),
                out: String::new(),
                result: 0,
                steps: 0,
                capped: None,
                written: Vec::new(),
            }
        }
    };
    let mut m = Machine::load(&program);
    m.input = chars(input);
    m.files = files.iter().map(|(n, t)| (n.to_string(), chars(t))).collect();
    let run = m.run_until(&program, caps.steps, caps.deadline(Instant::now()));
    let (out, written) = m.outputs();
    let (steps, capped) = (m.steps, m.capped);
    match run {
        Ok(result) => Verdict { compiled: true, error: None, out, result, steps, capped, written },
        Err(e) => {
            Verdict { compiled: true, error: Some(e), out, result: 0, steps, capped, written }
        }
    }
}

impl Verdict {
    /// Was the source supported by the machine? False when it did not
    /// compile, when it faulted, or when it reached a cap (R8.13). The
    /// classification is the detector's to make; this is the fact it reads.
    pub fn supported(&self) -> bool {
        self.compiled && self.error.is_none() && self.capped.is_none()
    }

    /// The verdict as rows, for any `World` or `Tally` to learn from.
    pub fn rows(&self, name: &str) -> Vec<Row> {
        let mut out = vec![
            Row::new(name, "compiled", if self.compiled { "yes" } else { "no" }),
            Row::new(name, "returned", &self.result.to_string()),
        ];
        if let Some(e) = &self.error {
            out.push(Row::new(name, "failed", e));
        }
        if let Some(cap) = self.capped {
            out.push(Row::new(name, "capped", cap.name()));
        }
        for line in self.out.lines().filter(|l| !l.is_empty()) {
            out.push(Row::new(name, "wrote", line));
        }
        out
    }
}

#[cfg(test)]
mod judged;
#[cfg(test)]
mod source;
