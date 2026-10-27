//! The native globals' slots: one name, one routine, one control transfer
//! where a routine moves the machine itself (coroutines, `longjump`,
//! `stop`), which the run loop does instead of a native call.

/// The native globals, in slot order: the five the `com/` sources lean on
/// most first (their slots are fixed), then the rest of BLIB's writers,
/// characters, strings, store, input and arithmetic, the coroutines and
/// `level`/`longjump`, and the streams.
pub const NATIVE: &[&str] = &[
    "writef", "writes", "newline", "writen", "wrch", "writed", "writez", "writeu", "writehex", "writeoct", "writebin",
    "capitalch", "compch", "compstring", "getvec", "freevec", "writet", "codewrch", "rdch", "unrdch", "readn",
    "randno", "setseed", "muldiv", "packstring", "unpackstring", "createco", "deleteco", "callco", "resumeco",
    "cowait", "level", "longjump", "stop", "abort", "findinput", "findoutput", "selectinput", "selectoutput",
    "endread", "endwrite", "input", "output",
];

/// A native routine, by its slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Native {
    Writef,
    Writes,
    Newline,
    Writen,
    Wrch,
    Writed,
    Writez,
    Writeu,
    Writehex,
    Writeoct,
    Writebin,
    Capitalch,
    Compch,
    Compstring,
    Getvec,
    Freevec,
    Writet,
    Codewrch,
    Rdch,
    Unrdch,
    Readn,
    Randno,
    Setseed,
    Muldiv,
    Packstring,
    Unpackstring,
    Createco,
    Deleteco,
    Callco,
    Resumeco,
    Cowait,
    Level,
    Longjump,
    Stop,
    Abort,
    Findinput,
    Findoutput,
    Selectinput,
    Selectoutput,
    Endread,
    Endwrite,
    Input,
    Output,
}

pub(super) const ORDER: [Native; 43] = [
    Native::Writef, Native::Writes, Native::Newline, Native::Writen, Native::Wrch, Native::Writed, Native::Writez,
    Native::Writeu, Native::Writehex, Native::Writeoct, Native::Writebin, Native::Capitalch, Native::Compch,
    Native::Compstring, Native::Getvec, Native::Freevec, Native::Writet, Native::Codewrch, Native::Rdch,
    Native::Unrdch, Native::Readn, Native::Randno, Native::Setseed, Native::Muldiv, Native::Packstring,
    Native::Unpackstring, Native::Createco, Native::Deleteco, Native::Callco, Native::Resumeco, Native::Cowait,
    Native::Level, Native::Longjump, Native::Stop, Native::Abort, Native::Findinput, Native::Findoutput,
    Native::Selectinput, Native::Selectoutput, Native::Endread, Native::Endwrite, Native::Input, Native::Output,
];

/// The routine at native entry `f` (negative).
pub(super) fn which(f: i64) -> Result<Native, String> {
    ORDER.get((-f - 1) as usize).copied().ok_or_else(|| format!("no native {}", -f - 1))
}

/// A routine that moves the machine itself rather than computing a value:
/// the run loop performs it (`machine/co.rs`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Control {
    Createco,
    Deleteco,
    Callco,
    Resumeco,
    Cowait,
    Level,
    Longjump,
    Stop,
    Abort,
}

/// The control transfer at native entry `f`, if it is one.
pub(crate) fn control(f: i64) -> Option<Control> {
    match which(f).ok()? {
        Native::Createco => Some(Control::Createco),
        Native::Deleteco => Some(Control::Deleteco),
        Native::Callco => Some(Control::Callco),
        Native::Resumeco => Some(Control::Resumeco),
        Native::Cowait => Some(Control::Cowait),
        Native::Level => Some(Control::Level),
        Native::Longjump => Some(Control::Longjump),
        Native::Stop => Some(Control::Stop),
        Native::Abort => Some(Control::Abort),
        _ => None,
    }
}
