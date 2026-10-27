//! The machine: BCPL's model kept whole — ONE word type, a linear store,
//! frames IN the store at P (so `@` of a local is a real address and `!`
//! reaches anything), a global vector at the bottom, strings and `TABLE`s
//! loaded after it, the stack region above, and `getvec`'s heap taken from
//! the top of the store down. Expression evaluation uses a Rust-side stack,
//! as INTCODE's interpreter kept registers outside the store.

use crate::global::NATIVE;
use crate::ocode::Program;

mod arith;
mod call;
mod co;
mod context;
mod heap;
mod run;

use context::Co;

pub(crate) use arith::{binary, unary};

/// Words of store, and where its regions sit.
pub const MEMORY: usize = 1 << 16;
pub const GLOBAL: usize = 400;

/// A return address meaning "the machine was entered here".
const OUTSIDE: i64 = -9_000_000;
/// A return address meaning "a coroutine's body was entered here".
const OUTSIDE_CO: i64 = -9_000_001;

/// One stream (`blib.b`'s stream control block, as the judge needs it):
/// an input's characters and how far they are read, or an output's text.
#[derive(Debug, Default)]
pub(crate) struct Stream {
    pub name: String,
    pub chars: Vec<i64>,
    pub read: usize,
    pub text: String,
    pub output: bool,
    pub open: bool,
}

/// The lowest the heap may reach, and the room a frame keeps below it.
const FLOOR: i64 = (MEMORY / 2) as i64;
const ROOM: i64 = 256;

pub struct Machine {
    pub mem: Vec<i64>,
    pub out: String,
    pub steps: usize,
    /// Each string's address in the store.
    string: Vec<i64>,
    /// Each `TABLE`'s address in the store.
    table: Vec<i64>,
    /// The bottom of the heap: `getvec` takes words below it.
    pub heap: i64,
    /// The blocks `getvec` handed out, newest last, each with whether it is
    /// still in use: the heap is a stack, so a freed block's words return
    /// once every block newer than it is freed too.
    block: Vec<(i64, bool)>,
    /// What `rdch` reads, and how far it has read.
    pub input: Vec<i64>,
    pub(crate) read: usize,
    /// `randno`'s seed (BLIB's `randseed`).
    pub(crate) seed: i64,
    /// The coroutines, the main program first, and the one running.
    co: Vec<Co>,
    current: i64,
    /// The streams: 1 is the judge's input, 2 its output, then the ones the
    /// program opened. `input`/`read` and `out` are the current two's.
    pub(crate) streams: Vec<Stream>,
    pub(crate) cis: usize,
    pub(crate) cos: usize,
    /// The named texts `findinput` can open.
    pub files: Vec<(String, Vec<i64>)>,
}

impl Machine {
    /// Loads a program: entries into the global vector, natives as negative
    /// entries, strings as `[len, chars…]` after the globals.
    pub fn load(program: &Program) -> Self {
        let mut mem = vec![0i64; MEMORY];
        for (i, _) in NATIVE.iter().enumerate() {
            mem[i] = -1 - i as i64;
        }
        for (slot, entry) in &program.entry {
            mem[*slot] = *entry as i64;
        }
        let mut at = GLOBAL;
        let mut string = Vec::new();
        for s in &program.string {
            string.push(at as i64);
            mem[at] = s.chars().count() as i64;
            for (i, c) in s.chars().enumerate() {
                mem[at + 1 + i] = c as i64;
            }
            at += s.chars().count() + 1;
        }
        let mut table = Vec::new();
        for t in &program.table {
            table.push(at as i64);
            mem[at..at + t.len()].copy_from_slice(t);
            at += t.len();
        }
        for (slot, k) in &program.init {
            mem[*slot] = *k;
        }
        let mut m = Machine {
            mem,
            out: String::new(),
            steps: 0,
            string,
            table,
            heap: MEMORY as i64,
            block: Vec::new(),
            input: Vec::new(),
            read: 0,
            seed: 0,
            co: Vec::new(),
            current: 0,
            streams: vec![
                Stream { name: "*".into(), open: true, ..Stream::default() },
                Stream { name: "*".into(), output: true, open: true, ..Stream::default() },
            ],
            cis: 1,
            cos: 2,
            files: Vec::new(),
        };
        m.mem[GLOBAL - 1] = (at + 8) as i64; // where the stack region starts
        m
    }

    /// After a run: what the program wrote to the judge's output, and each
    /// stream it opened with `findoutput`, by name.
    pub fn outputs(&mut self) -> (String, Vec<(String, String)>) {
        self.streams[self.cos - 1].text = std::mem::take(&mut self.out);
        self.cos = 2;
        let stdout = std::mem::take(&mut self.streams[1].text);
        let written = self.streams.iter().skip(2).filter(|s| s.output).map(|s| (s.name.clone(), s.text.clone())).collect();
        (stdout, written)
    }

    fn word(&self, at: i64) -> Result<i64, String> {
        self.mem.get(at as usize).copied().ok_or_else(|| format!("read outside the store: {at}"))
    }

    fn set(&mut self, at: i64, v: i64) -> Result<(), String> {
        match self.mem.get_mut(at as usize) {
            Some(w) => {
                *w = v;
                Ok(())
            }
            None => Err(format!("write outside the store: {at}")),
        }
    }
}
